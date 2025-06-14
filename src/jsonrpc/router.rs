//! Lightweight JSON-RPC router service.

use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::future::{self, BoxFuture, FutureExt};
use lsp_types::LSPAny;
use serde::{Serialize, de::DeserializeOwned};
use tower::{Layer, Service, util::BoxService};

use crate::jsonrpc::ErrorCode;

use super::{Error, Id, Request, Response};

/// A modular JSON-RPC 2.0 request router service.
pub struct Router<S, E = Infallible> {
    server: Arc<S>,
    methods: HashMap<&'static str, BoxService<Request, Option<Response>, E>>,
}

impl<S: Send + Sync + 'static, E> Router<S, E> {
    /// Creates a new `Router` with the given shared state.
    pub fn new(server: S) -> Self {
        Self {
            server: Arc::new(server),
            methods: HashMap::new(),
        }
    }

    /// Returns a reference to the inner server.
    pub fn inner(&self) -> &S {
        self.server.as_ref()
    }

    /// Registers a new RPC method which constructs a response with the given `callback`.
    ///
    /// The `layer` argument can be used to inject middleware into the method handler, if desired.
    pub fn method<P, R, F, L>(&mut self, name: &'static str, callback: F, layer: L) -> &mut Self
    where
        P: FromParams,
        R: IntoResponse,
        F: for<'a> Method<&'a S, P, R> + Clone + Send + Sync + 'static,
        L: Layer<MethodHandler<P, R, E>>,
        L::Service: Service<Request, Response = Option<Response>, Error = E> + Send + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        let server = &self.server;
        self.methods.entry(name).or_insert_with(|| {
            let server = server.clone();
            let handler = MethodHandler::new(move |params| {
                let callback = callback.clone();
                let server = server.clone();
                async move { callback.invoke(&*server, params).await }
            });

            BoxService::new(layer.layer(handler))
        });

        self
    }
}

impl<S: Debug, E> Debug for Router<S, E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Router")
            .field("server", &self.server)
            .field("methods", &self.methods.keys())
            .finish()
    }
}

impl<S, E: Send + 'static> Service<Request> for Router<S, E> {
    type Response = Option<Response>;
    type Error = E;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        if let Some(handler) = self.methods.get_mut(req.method()) {
            handler.call(req)
        } else {
            let (method, id, _) = req.into_parts();
            future::ok(id.map(|id| {
                let mut error = Error::method_not_found();
                error.data = Some(LSPAny::from(method));
                Response::from_error(id, error)
            }))
            .boxed()
        }
    }
}

/// Opaque JSON-RPC method handler.
pub struct MethodHandler<P, R, E> {
    f: Box<dyn Fn(P) -> BoxFuture<'static, R> + Send>,
    _marker: PhantomData<E>,
}

impl<P: FromParams, R: IntoResponse, E> MethodHandler<P, R, E> {
    fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(P) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        Self {
            f: Box::new(move |p| handler(p).boxed()),
            _marker: PhantomData,
        }
    }
}

impl<P, R, E> Service<Request> for MethodHandler<P, R, E>
where
    P: FromParams,
    R: IntoResponse,
    E: Send + 'static,
{
    type Response = Option<Response>;
    type Error = E;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let (_, id, params) = req.into_parts();

        match id {
            Some(_) if R::is_notification() => return future::ok(().into_response(id)).boxed(),
            None if !R::is_notification() => return future::ok(None).boxed(),
            _ => {}
        }

        let params = match P::from_params(params) {
            Ok(params) => params,
            Err(err) => return future::ok(id.map(|id| Response::from_error(id, err))).boxed(),
        };

        (self.f)(params)
            .map(move |r| Ok(r.into_response(id)))
            .boxed()
    }
}

/// A trait implemented by all valid JSON-RPC method handlers.
///
/// This trait abstracts over the following classes of functions and/or closures:
///
/// Signature                                            | Description
/// -----------------------------------------------------|---------------------------------
/// `async fn f(&self) -> jsonrpc::Result<R>`            | Request without parameters
/// `async fn f(&self, params: P) -> jsonrpc::Result<R>` | Request with required parameters
/// `async fn f(&self)`                                  | Notification without parameters
/// `async fn f(&self, params: P)`                       | Notification with parameters
pub trait Method<S, P, R>: private::Sealed {
    /// The future response value.
    type Future: Future<Output = R> + Send;

    /// Invokes the method with the given `server` receiver and parameters.
    fn invoke(&self, server: S, params: P) -> Self::Future;
}

/// Support parameter-less JSON-RPC methods.
impl<F, S, R, Fut> Method<S, (), R> for F
where
    F: Fn(S) -> Fut,
    Fut: Future<Output = R> + Send,
{
    type Future = Fut;

    #[inline]
    fn invoke(&self, server: S, (): ()) -> Self::Future {
        self(server)
    }
}

/// Support JSON-RPC methods with `params`.
impl<F, S, P, R, Fut> Method<S, (P,), R> for F
where
    F: Fn(S, P) -> Fut,
    P: DeserializeOwned,
    Fut: Future<Output = R> + Send,
{
    type Future = Fut;

    #[inline]
    fn invoke(&self, server: S, params: (P,)) -> Self::Future {
        self(server, params.0)
    }
}

/// A trait implemented by all JSON-RPC method parameters.
pub trait FromParams: private::Sealed + Send + Sized + 'static {
    /// Attempts to deserialize `Self` from the `params` value extracted from [`Request`].
    ///
    /// # Errors
    ///
    /// - If the given parameters don't match the expected shape
    fn from_params(params: Option<LSPAny>) -> super::Result<Self>;
}

/// Deserialize non-existent JSON-RPC parameters.
impl FromParams for () {
    fn from_params(params: Option<LSPAny>) -> super::Result<Self> {
        match params {
            None
            // See #40: allow lsp clients (e.g. `lsp4j`) to not precisely
            // respect the specification and set `params` to `null` when it
            // should not be present at all.
            | Some(LSPAny::Null) => Ok(()),
            Some(p) => Err(Error::invalid_params(format!("Unexpected params: {p}"))),
        }
    }
}

/// Deserialize required JSON-RPC parameters.
impl<P: DeserializeOwned + Send + 'static> FromParams for (P,) {
    fn from_params(params: Option<LSPAny>) -> super::Result<Self> {
        params.map_or_else(
            || Err(Error::invalid_params("Missing params field")),
            |p| {
                serde_json::from_value(p)
                    .map(|params| (params,))
                    .map_err(|e| Error::invalid_params(e.to_string()))
            },
        )
    }
}

/// A trait implemented by all JSON-RPC response types.
pub trait IntoResponse: private::Sealed + Send + 'static {
    /// Attempts to construct a [`Response`] using `Self` and a corresponding [`Id`].
    fn into_response(self, id: Option<Id>) -> Option<Response>;

    /// Returns `true` if this is a notification response type.
    fn is_notification() -> bool;
}

/// Support JSON-RPC notification methods.
impl IntoResponse for () {
    #[expect(clippy::single_option_map, reason = "we cannot change trait signature")]
    fn into_response(self, id: Option<Id>) -> Option<Response> {
        id.map(|id| Response::from_error(id, Error::invalid_request()))
    }

    #[inline]
    fn is_notification() -> bool {
        true
    }
}

/// Support JSON-RPC request methods.
impl<R: Serialize + Send + 'static> IntoResponse for Result<R, Error> {
    fn into_response(self, id: Option<Id>) -> Option<Response> {
        debug_assert!(id.is_some(), "Requests always contain an `id` field");
        id.map(|id| {
            let result = self.and_then(|r| {
                serde_json::to_value(r).map_err(|e| Error {
                    code: ErrorCode::InternalError,
                    message: e.to_string().into(),
                    data: None,
                })
            });
            Response::from_parts(id, result)
        })
    }

    #[inline]
    fn is_notification() -> bool {
        false
    }
}

mod private {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tower::ServiceExt;
    use tower::layer::layer_fn;

    use super::*;

    #[derive(Deserialize, Serialize)]
    struct Params {
        foo: i32,
        bar: String,
    }

    struct Mock;

    #[expect(clippy::unused_async)]
    impl Mock {
        async fn request(&self) -> Result<LSPAny, Error> {
            Ok(LSPAny::Null)
        }

        async fn request_params(&self, params: Params) -> Result<Params, Error> {
            Ok(params)
        }

        async fn notification(&self) {}

        async fn notification_params(&self, _params: Params) {}
    }

    #[tokio::test(flavor = "current_thread")]
    async fn routes_requests() {
        let mut router: Router<Mock> = Router::new(Mock);
        router
            .method("first", Mock::request, layer_fn(|s| s))
            .method("second", Mock::request_params, layer_fn(|s| s));

        let request = Request::build("first").id(0).finish();
        let response = router.ready().await.unwrap().call(request).await;
        assert_eq!(
            response,
            Ok(Some(Response::from_ok(0.into(), LSPAny::Null)))
        );

        let params = json!({"foo": -123i32, "bar": "hello world"});
        let with_params = Request::build("second")
            .params(params.clone())
            .id(1)
            .finish();
        let response = router.ready().await.unwrap().call(with_params).await;
        assert_eq!(response, Ok(Some(Response::from_ok(1.into(), params))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn routes_notifications() {
        let mut router: Router<Mock> = Router::new(Mock);
        router
            .method("first", Mock::notification, layer_fn(|s| s))
            .method("second", Mock::notification_params, layer_fn(|s| s));

        let request = Request::build("first").finish();
        let response = router.ready().await.unwrap().call(request).await;
        assert_eq!(response, Ok(None));

        let params = json!({"foo": -123i32, "bar": "hello world"});
        let with_params = Request::build("second").params(params).finish();
        let response = router.ready().await.unwrap().call(with_params).await;
        assert_eq!(response, Ok(None));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rejects_request_with_invalid_params() {
        let mut router: Router<Mock> = Router::new(Mock);
        router.method("request", Mock::request_params, layer_fn(|s| s));

        let invalid_params = Request::build("request")
            .params(json!("wrong"))
            .id(0)
            .finish();

        let response = router.ready().await.unwrap().call(invalid_params).await;
        assert_eq!(
            response,
            Ok(Some(Response::from_error(
                0.into(),
                Error::invalid_params("invalid type: string \"wrong\", expected struct Params"),
            )))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ignores_notification_with_invalid_params() {
        let mut router: Router<Mock> = Router::new(Mock);
        router.method("notification", Mock::request_params, layer_fn(|s| s));

        let invalid_params = Request::build("notification")
            .params(json!("wrong"))
            .finish();

        let response = router.ready().await.unwrap().call(invalid_params).await;
        assert_eq!(response, Ok(None));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn handles_incorrect_request_types() {
        let mut router: Router<Mock> = Router::new(Mock);
        router
            .method("first", Mock::request, layer_fn(|s| s))
            .method("second", Mock::notification, layer_fn(|s| s));

        let request = Request::build("first").finish();
        let response = router.ready().await.unwrap().call(request).await;
        assert_eq!(response, Ok(None));

        let request = Request::build("second").id(0).finish();
        let response = router.ready().await.unwrap().call(request).await;
        assert_eq!(
            response,
            Ok(Some(Response::from_error(
                0.into(),
                Error::invalid_request(),
            )))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn responds_to_nonexistent_request() {
        let mut router: Router<Mock> = Router::new(Mock);

        let request = Request::build("foo").id(0).finish();
        let response = router.ready().await.unwrap().call(request).await;
        let mut error = Error::method_not_found();
        error.data = Some("foo".into());
        assert_eq!(response, Ok(Some(Response::from_error(0.into(), error))));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ignores_nonexistent_notification() {
        let mut router: Router<Mock> = Router::new(Mock);

        let request = Request::build("foo").finish();
        let response = router.ready().await.unwrap().call(request).await;
        assert_eq!(response, Ok(None));
    }
}
