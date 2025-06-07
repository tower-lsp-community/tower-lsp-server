//! Shows how to serve a language server over standard input/output

use tower_lsp_server::{LspService, Server};

// You can check the language server implementation in `examples/common/lsp.rs`.
// The dummy language server provided is common to all transports examples.
#[path = "common/lsp.rs"]
mod lsp;

#[tokio::main]
async fn main() {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    let (service, socket) = LspService::new(lsp::Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
