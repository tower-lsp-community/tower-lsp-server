//! Language Server Protocol (LSP) server abstraction for [Tower].
//!
//! [Tower]: https://github.com/tower-rs/tower
//!
//! # Example
//!
//! ```rust
//! use tower_lsp_server::jsonrpc::Result;
//! use tower_lsp_server::lsp_types::*;
//! use tower_lsp_server::{Client, LanguageServer, LspService, Server};
//!
//! #[derive(Debug)]
//! struct Backend {
//!     client: Client,
//! }
//!
//! impl LanguageServer for Backend {
//!     async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
//!         Ok(InitializeResult {
//!             capabilities: ServerCapabilities {
//!                 hover_provider: Some(HoverProviderCapability::Simple(true)),
//!                 completion_provider: Some(CompletionOptions::default()),
//!                 ..Default::default()
//!             },
//!             ..Default::default()
//!         })
//!     }
//!
//!     async fn initialized(&self, _: InitializedParams) {
//!         self.client
//!             .log_message(MessageType::INFO, "server initialized!")
//!             .await;
//!     }
//!
//!     async fn shutdown(&self) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
//!         Ok(Some(CompletionResponse::Array(vec![
//!             CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
//!             CompletionItem::new_simple("Bye".to_string(), "More detail".to_string())
//!         ])))
//!     }
//!
//!     async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
//!         Ok(Some(Hover {
//!             contents: HoverContents::Scalar(
//!                 MarkedString::String("You're hovering!".to_string())
//!             ),
//!             range: None
//!         }))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//! #   tracing_subscriber::fmt().init();
//! #
//! #   #[cfg(feature = "runtime-agnostic")]
//! #   use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
//! #   use std::io::Cursor;
//!     let stdin = tokio::io::stdin();
//!     let stdout = tokio::io::stdout();
//! #   let message = r#"{"jsonrpc":"2.0","method":"exit"}"#;
//! #   let (stdin, stdout) = (Cursor::new(format!("Content-Length: {}\r\n\r\n{}", message.len(), message).into_bytes()), Cursor::new(Vec::new()));
//! #   #[cfg(feature = "runtime-agnostic")]
//! #   let (stdin, stdout) = (stdin.compat(), stdout.compat_write());
//!
//!     let (service, socket) = LspService::new(|client| Backend { client });
//!     Server::new(stdin, stdout, socket).serve(service).await;
//! }
//! ```

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// A re-export of [`lsp-types`](https://docs.rs/lsp-types) for convenience.
pub use lsp_types;

pub use self::server::LanguageServer;
pub use self::service::progress::{
    Bounded, Cancellable, NotCancellable, OngoingProgress, Progress, Unbounded,
};
pub use self::service::{Client, ClientSocket, ExitedError, LspService, LspServiceBuilder};
pub use self::transport::{Loopback, Server};
pub use self::uri_ext::{uri_from_file_path, UriExt};

pub mod jsonrpc;

mod codec;
mod server;
mod service;
mod transport;
mod uri_ext;
