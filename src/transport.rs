//! Generic server for multiplexing bidirectional streams through a transport.

#[cfg(feature = "runtime-agnostic")]
use async_codec_lite::{FramedRead, FramedWrite};
#[cfg(feature = "runtime-agnostic")]
use futures::io::{AsyncRead, AsyncWrite};

#[cfg(feature = "runtime-tokio")]
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "runtime-tokio")]
use tokio_util::codec::{FramedRead, FramedWrite};

use futures::channel::mpsc;
use futures::{
    FutureExt, Sink, SinkExt, Stream, StreamExt, TryFutureExt, future, join, stream, stream_select,
};
use tower::Service;
use tracing::error;

use crate::codec::{LanguageServerCodec, ParseError};
use crate::jsonrpc::{Error, Id, Message, Request, Response};
use crate::service::{ClientSocket, RequestStream, ResponseSink};

const DEFAULT_MAX_CONCURRENCY: usize = 4;
const MESSAGE_QUEUE_SIZE: usize = 100;

/// Trait implemented by client loopback sockets.
///
/// This socket handles the server-to-client half of the bidirectional communication stream.
pub trait Loopback {
    /// Yields a stream of pending server-to-client requests.
    type RequestStream: Stream<Item = Request> + Unpin;
    /// Routes client-to-server responses back to the server.
    type ResponseSink: Sink<Response> + Unpin;

    /// Splits this socket into two halves capable of operating independently.
    ///
    /// The two halves returned implement the [`Stream`] and [`Sink`] traits, respectively.
    fn split(self) -> (Self::RequestStream, Self::ResponseSink);
}

impl Loopback for ClientSocket {
    type RequestStream = RequestStream;
    type ResponseSink = ResponseSink;

    #[inline]
    fn split(self) -> (Self::RequestStream, Self::ResponseSink) {
        self.split()
    }
}

/// Server for processing requests and responses on standard I/O or TCP.
#[derive(Debug)]
pub struct Server<I, O, L = ClientSocket> {
    stdin: I,
    stdout: O,
    loopback: L,
    max_concurrency: usize,
}

impl<I, O, L> Server<I, O, L>
where
    I: AsyncRead + Unpin,
    O: AsyncWrite,
    L: Loopback,
    <L::ResponseSink as Sink<Response>>::Error: std::error::Error,
{
    /// Creates a new `Server` with the given `stdin` and `stdout` handles.
    pub const fn new(stdin: I, stdout: O, socket: L) -> Self {
        Self {
            stdin,
            stdout,
            loopback: socket,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
        }
    }

    /// Sets the server concurrency limit to `max`.
    ///
    /// This setting specifies how many incoming requests may be processed concurrently. Setting
    /// this value to `1` forces all requests to be processed sequentially, thereby implicitly
    /// disabling support for the [`$/cancelRequest`] notification.
    ///
    /// [`$/cancelRequest`]: https://microsoft.github.io/language-server-protocol/specification#cancelRequest
    ///
    /// If not explicitly specified, `max` defaults to 4.
    ///
    /// # Preference over standard `tower` middleware
    ///
    /// The [`ConcurrencyLimit`] and [`Buffer`] middlewares provided by `tower` rely on
    /// [`tokio::spawn`] in common usage, while this library aims to be executor agnostic and to
    /// support exotic targets currently incompatible with `tokio`, such as WASM. As such, `Server`
    /// includes its own concurrency facilities that don't require a global executor to be present.
    ///
    /// [`ConcurrencyLimit`]: https://docs.rs/tower/latest/tower/limit/concurrency/struct.ConcurrencyLimit.html
    /// [`Buffer`]: https://docs.rs/tower/latest/tower/buffer/index.html
    /// [`tokio::spawn`]: https://docs.rs/tokio/latest/tokio/fn.spawn.html
    #[must_use]
    pub const fn concurrency_level(mut self, max: usize) -> Self {
        self.max_concurrency = max;
        self
    }

    /// Spawns the service with messages read through `stdin` and responses written to `stdout`.
    pub async fn serve<T>(self, mut service: T)
    where
        T: Service<Request, Response = Option<Response>> + Send + 'static,
        T::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        T::Future: Send,
    {
        let (client_requests, mut client_responses) = self.loopback.split();
        let (client_requests, client_abort) = stream::abortable(client_requests);
        let (mut responses_tx, responses_rx) = mpsc::channel(0);
        let (mut server_tasks_tx, server_tasks_rx) = mpsc::channel(MESSAGE_QUEUE_SIZE);

        let mut framed_stdin = FramedRead::new(self.stdin, LanguageServerCodec::default());
        let framed_stdout = FramedWrite::new(self.stdout, LanguageServerCodec::default());

        let process_server_tasks = server_tasks_rx
            .buffer_unordered(self.max_concurrency)
            .filter_map(future::ready)
            .map(|res| Ok(Message::Response(res)))
            .forward(responses_tx.clone());

        let print_output = stream_select!(responses_rx, client_requests.map(Message::Request))
            .map(Ok)
            .forward(framed_stdout);

        let read_input = async {
            while let Some(msg) = framed_stdin.next().await {
                match msg {
                    Ok(Message::Request(req)) => {
                        if let Err(err) = future::poll_fn(|cx| service.poll_ready(cx)).await {
                            error!("{}", display_sources(err.into().as_ref()));
                            return;
                        }

                        // per specifications, the server must exit immediately after the exit notification
                        // some clients will not close stdin and thus keep framed_stdin waiting forever
                        // we break early here so that control can be yielded back immediately
                        let will_exit = req.method() == "exit";

                        let fut = service.call(req).unwrap_or_else(|err| {
                            error!("{}", display_sources(err.into().as_ref()));
                            None
                        });

                        let _ = server_tasks_tx.send(fut).await;

                        if will_exit {
                            break;
                        }
                    }
                    Ok(Message::Response(res)) => {
                        if let Err(err) = client_responses.send(res).await {
                            error!("{}", display_sources(&err));
                            return;
                        }
                    }
                    Err(err) => {
                        error!("failed to decode message: {}", err);
                        let res = Response::from_error(Id::Null, to_jsonrpc_error(err));
                        let _ = responses_tx.send(Message::Response(res)).await;
                    }
                }
            }

            server_tasks_tx.disconnect();
            responses_tx.disconnect();
            client_abort.abort();
        };

        join!(
            process_server_tasks.map(|_| ()),
            print_output.map(|_| ()),
            read_input
        );
    }
}

fn display_sources(error: &dyn std::error::Error) -> String {
    error.source().map_or_else(
        || error.to_string(),
        |source| format!("{}: {}", error, display_sources(source)),
    )
}

#[cfg(feature = "runtime-tokio")]
fn to_jsonrpc_error(err: ParseError) -> Error {
    match err {
        ParseError::Body(err) if err.is_data() => Error::invalid_request(),
        _ => Error::parse_error(),
    }
}

#[cfg(feature = "runtime-agnostic")]
fn to_jsonrpc_error(err: impl std::error::Error) -> Error {
    match err.source().and_then(|e| e.downcast_ref()) {
        Some(ParseError::Body(err)) if err.is_data() => Error::invalid_request(),
        _ => Error::parse_error(),
    }
}

#[cfg(test)]
mod tests {
    use std::task::{Context, Poll};

    #[cfg(feature = "runtime-agnostic")]
    use futures::io::Cursor;
    #[cfg(feature = "runtime-tokio")]
    use std::io::Cursor;

    use futures::future::Ready;
    use futures::{future, sink, stream};

    use super::*;

    const REQUEST: &str = r#"{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}"#;
    const RESPONSE: &str = r#"{"jsonrpc":"2.0","result":{"capabilities":{}},"id":1}"#;

    #[derive(Debug)]
    struct MockService;

    impl Service<Request> for MockService {
        type Response = Option<Response>;
        type Error = String;
        type Future = Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _: Request) -> Self::Future {
            let response = serde_json::from_str(RESPONSE).unwrap();
            future::ok(Some(response))
        }
    }

    struct MockLoopback(Vec<Request>);

    impl Loopback for MockLoopback {
        type RequestStream = stream::Iter<std::vec::IntoIter<Request>>;
        type ResponseSink = sink::Drain<Response>;

        fn split(self) -> (Self::RequestStream, Self::ResponseSink) {
            (stream::iter(self.0), sink::drain())
        }
    }

    fn mock_request() -> Vec<u8> {
        format!("Content-Length: {}\r\n\r\n{}", REQUEST.len(), REQUEST).into_bytes()
    }

    fn mock_response() -> Vec<u8> {
        format!("Content-Length: {}\r\n\r\n{}", RESPONSE.len(), RESPONSE).into_bytes()
    }

    fn mock_stdio() -> (Cursor<Vec<u8>>, Vec<u8>) {
        (Cursor::new(mock_request()), Vec::new())
    }

    // Simulates a still-live stdin that the client didn't drop.
    struct DetachedCursor(Vec<u8>);

    #[cfg(feature = "runtime-tokio")]
    impl AsyncRead for DetachedCursor {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            if self.0.is_empty() {
                return Poll::Pending;
            }

            buf.put_slice(&self.0);
            self.0.clear();
            Poll::Ready(Ok(()))
        }
    }

    #[cfg(feature = "runtime-agnostic")]
    impl AsyncRead for DetachedCursor {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            if self.0.is_empty() {
                return Poll::Pending;
            }

            let len = std::cmp::min(buf.len(), self.0.len());
            let after = self.0.split_off(len);
            buf[..len].copy_from_slice(&self.0);
            self.0 = after;
            Poll::Ready(Ok(len))
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn serves_on_stdio() {
        let (mut stdin, mut stdout) = mock_stdio();
        Server::new(&mut stdin, &mut stdout, MockLoopback(vec![]))
            .serve(MockService)
            .await;

        assert_eq!(stdin.position(), 80);
        assert_eq!(stdout, mock_response());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interleaves_messages() {
        let socket = MockLoopback(vec![serde_json::from_str(REQUEST).unwrap()]);

        let (mut stdin, mut stdout) = mock_stdio();
        Server::new(&mut stdin, &mut stdout, socket)
            .serve(MockService)
            .await;

        assert_eq!(stdin.position(), 80);
        let output: Vec<_> = mock_request().into_iter().chain(mock_response()).collect();
        assert_eq!(stdout, output);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn handles_invalid_json() {
        let invalid = r#"{"jsonrpc":"2.0","method":"#;
        let message = format!("Content-Length: {}\r\n\r\n{}", invalid.len(), invalid).into_bytes();
        let (mut stdin, mut stdout) = (Cursor::new(message), Vec::new());

        Server::new(&mut stdin, &mut stdout, MockLoopback(vec![]))
            .serve(MockService)
            .await;

        assert_eq!(stdin.position(), 48);
        let err = r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#;
        let output = format!("Content-Length: {}\r\n\r\n{}", err.len(), err).into_bytes();
        assert_eq!(stdout, output);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stops_promptly_after_exit_notification() {
        let req = r#"{"jsonrpc":"2.0","method":"exit"}"#;
        let message = format!("Content-Length: {}\r\n\r\n{}", req.len(), req).into_bytes();
        let (mut stdin, mut stdout) = (DetachedCursor(message), Vec::new());

        assert!(
            tokio::time::timeout(
                std::time::Duration::from_secs(1),
                Server::new(&mut stdin, &mut stdout, MockLoopback(vec![])).serve(MockService)
            )
            .await
            .is_ok(),
            "waited for more than 1 second for exit"
        );
    }
}
