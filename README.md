# tower-lsp-server

[![CI][ci-badge]][ci-badge-url]
[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]

*A community fork of [tower-lsp](https://github.com/ebkalderon/tower-lsp)*

[ci-badge]: https://github.com/tower-lsp-community/tower-lsp-server/actions/workflows/rust.yml/badge.svg?branch=main
[ci-badge-url]: https://github.com/tower-lsp-community/tower-lsp-server/actions
[crates-badge]: https://img.shields.io/crates/v/tower-lsp-server.svg
[crates-url]: https://crates.io/crates/tower-lsp-server
[docs-badge]: https://docs.rs/tower-lsp-server/badge.svg
[docs-url]: https://docs.rs/tower-lsp-server

[Language Server Protocol] implementation for Rust based on [Tower].

[language server protocol]: https://microsoft.github.io/language-server-protocol
[tower]: https://github.com/tower-rs/tower

Tower is a simple and composable framework for implementing asynchronous services in Rust. Central to Tower is the [`Service`] trait, which provides the necessary abstractions for defining request/response clients and servers. Examples of protocols implemented using the `Service` trait include [`hyper`] for HTTP and [`tonic`] for gRPC.

[`service`]: https://docs.rs/tower-service/
[`hyper`]: https://docs.rs/hyper/
[`tonic`]: https://docs.rs/tonic/

`tower-lsp-server` provides a simple implementation of the Language Server Protocol (LSP) that makes it easy to write your own language server. It consists of three parts:

- The `LanguageServer` trait which defines the behavior of your language server.
- The asynchronous `LspService` delegate which wraps your language server
  implementation and defines the behavior of the protocol.
- A `Server` which spawns the `LspService` and processes requests and responses
  over `stdio` or TCP.

# Example

```rust
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

See more [examples](https://github.com/tower-lsp-community/tower-lsp-server/tree/main/examples).

# Using runtimes other than tokio

By default, `tower-lsp-server` is configured for use with `tokio`. Using `tower-lsp-server` with other runtimes requires disabling `default-features` and enabling the `runtime-agnostic` feature:

```toml
[dependencies.tower-lsp-server]
version = "*"
default-features = false
features = ["runtime-agnostic"]
```

# Using proposed features

You can use enable proposed features in the [LSP Specification version 3.18](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/) by enabling the `proposed` Cargo crate feature. Note that there are no semver guarantees to the `proposed` features so there may be breaking changes between any type of version in the `proposed` features.

# Projects using `tower-lsp-server`

- [Harper](https://github.com/Automattic/harper)
- [Polarity](https://github.com/polarity-lang/polarity/): both for their language server and their [interactive web demo](https://polarity-lang.github.io).
- [Deno](https://github.com/denoland/deno/tree/main/cli/lsp) (still uses the original project)
- [Turborepo](https://github.com/vercel/turborepo/tree/main/crates/turborepo-lsp) (still uses the original project)

# Ecosystem

- [tower-lsp-boilerplate](https://github.com/IWANABETHATGUY/tower-lsp-boilerplate) - Useful GitHub project template which makes writing new language servers easier. This is made for the original project but should be straight forward to adapt. Issue [#23](https://github.com/tower-lsp-community/tower-lsp-server/issues/23) proposes to host a similar project within the organization

# License

`tower-lsp-server` is free and open source software distributed under the terms of either the [MIT](LICENSE-MIT) or the [Apache 2.0](LICENSE-APACHE) license, at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
