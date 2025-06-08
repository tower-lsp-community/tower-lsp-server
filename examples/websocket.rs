//! Shows how to serve a language server of websockets.

use async_tungstenite::{ByteReader, ByteWriter, tokio::accept_async};
use futures::StreamExt;
use tokio::net::TcpListener;
use tower_lsp_server::{LspService, Server};
use tracing::info;

// You can check the language server implementation in `examples/common/lsp.rs`.
// The dummy language server provided is common to all transports examples.
#[path = "common/lsp.rs"]
mod lsp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();

    let listener = TcpListener::bind("127.0.0.1:9257").await?;
    info!("Listening on {}", listener.local_addr()?);
    let (stream, _) = listener.accept().await?;

    let (write, read) = accept_async(stream).await?.split();
    let (write, read) = (ByteWriter::new(write), ByteReader::new(read));

    #[cfg(feature = "runtime-agnostic")]
    let (read, write) = (read.compat(), write.compat_write());

    let (service, socket) = LspService::new(lsp::Backend::new);
    Server::new(read, write, socket).serve(service).await;

    Ok(())
}
