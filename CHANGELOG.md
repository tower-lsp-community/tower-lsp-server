# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.22.0] - 2025-04-07

### Changed

* Fixed the return type for the `workspace/symbol` handler to match the specification (#49)
* Updated the crate to edition 2024 and bump MSRV to 1.85 accordingly

### Fixed

* Percent encode path in `UriExt::from_file_path` (#53)
* Consistent windows driver conversion in `UriExt` (#56)

## [0.21.1] - 2025-04-07

### Added

* Export `UriExt` to reduce development friction (#38)

### Fixed

* Do no panic when recieving a response whose request has been cancelled (#37)
* Allow `null` param in JSON-RPC response when it should be None (#41)

## [0.21.0] - 2025-03-16

### Added

* Add notebook support (#15).
  * Implement `notebookDocument/didOpen` server request.
  * Implement `notebookDocument/didChange` server request.
  * Implement `notebookDocument/didSave` server request.
  * Implement `notebookDocument/didClose` server request.

### Changed

* Update `lsp-types` from `0.94.1` to `0.97` (#6).
* Bump minimum supported Rust version from `1.64.0` to `1.77.0`.
* The `LanguageServer` trait now uses `impl Trait` in Trait feature (#21).
  * `LanguageServer` is not longer dyn-compatible/object-safe
  * You should be able to migrate from the old trait signature by just dropping the `#[async_trait]` macro at the top of the implementation.
* Correctly use `lsp_types::LSPAny` in public API instead of `serde_json::Value` (#10).

### Fixed

* Use `stream_select` to avoid hanging behaviour (#9)
* Close transport 1s after exit notification instead of waiting for transport channel to close (#29)

### Removed

* `async_trait` is no longer exported as it is no longer needed for the `LanguageServer` trait.

## [0.20.0] - 2023-08-10

> Versions 0.20 and below correspond to the original tower-lsp project.

### Added

* Add support for pull-based diagnostics from LSP 3.17.0 (PR #396).
  * Implement `textDocument/diagnostic` server request.
  * Implement `workspace/diagnostic` server request.
  * Implement `workspace/diagnostic/refresh` client request.
* Implement `std::str::FromStr` for `jsonrpc::{Request,Response}` (PR #379).
* Implement `From<jsonrpc::ErrorCode>` for `i64` (PR #379).
* Document supported LSP features in [FEATURES.md](./FEATURES.md) matrix (PR
  #383).

### Changed

* Bump minimum supported Rust version from `1.52.0` to `1.64.0` (PR #377, PR #395).
* Update `lsp-types` from `0.94` to `0.94.1` (PR #396).
* Update `syn` from `1` to `2` (PR #390).
* Update dev-dependency `async-tungstenite` from `0.18` to `0.22` (PR #395).
* Update dev-dependency `ws_stream_tungstenite` from `0.9` to `0.10` (PR #395).
* Optimize JSON-RPC deserialization types.
  * Change `jsonrpc::Error::message` field to `Cow<'static, str>` (PR #378).
  * Mark several methods on `jsonrpc::Error` as `const fn` (PR #378).
  * Mark all methods on `jsonrpc::ErrorCode` as `const fn` (PR #378).
  * Avoid heap allocation in `version` field deserialization (PR #379).

### Fixed

* Fix broken Markdown in doc comment for `LanguageServer::completion()` (PR #396).

## [0.19.0] - 2023-02-28

### Added

* Add `LspService::inner()` method (PR #344).
* Add missing `window/showDocument` client request from LSP 3.16.0 (PR #375).
* Add partial support for Language Server Protocol 3.17.0 (PR #375):
  * Implement `textDocument/prepareTypeHierarchy` server request.
  * Implement `typeHierarchy/supertypes` server request.
  * Implement `typeHierarchy/subtypes` server request.
  * Implement `textDocument/inlineValue` server request.
  * Implement `textDocument/inlayHint` server request.
  * Implement `inlayHint/resolve` server request.
  * Implement `workspaceSymbol/resolve` server request.
  * Implement `workspace/inlineValue/refresh` client request.
  * Implement `workspace/inlayHint/refresh` client request.

### Changed

* Address Clippy lints (PR #369).
* Update `edition` from `2018` to `2021` (PR #370).
* Update `lsp-types` from `0.93` to `0.94` (PR #367).
* Reorder `LanguageServer` trait methods to match the LSP 3.17.0 spec document
  (PR #375).
* Reorder `Client` inherent methods to better match the LSP 3.17.0 spec document
  (PR #375).

### Fixed

* Fix doc links for `textDocument/colorPresentation` request (PR #371).
* Fix doc links for `textDocument/willSaveWaitUntil` request (PR #371).
* Fix doc links for `workspace/didChangeWatchedFiles` request (PR #371).
* Improve documentation for `LanguageServer` and `Client` methods (PR #375).

## [0.18.0] - 2023-01-14

### Changed

* Switch from `log` facade to `tracing` (PR #332).
* Change `$/cancelRequest` log message from `warn` to `debug` (PR #353).
* Update `auto_impl` from `0.5` to `1.0` (PR #343).
* Update `httparse` from `1.3.5` to `1.8` (PR #363)
* Update `memchr` from `2.4.1` to `2.5` (PR #363).
* Relax `tower` version requirement from `0.3.11` to `0.3` (PR #363).
* Update dev-dependency `async-tungstenite` from `0.16` to `0.18` (PR #363).
* Update dev-dependency `ws_stream_tungstenite` from `0.7` to `0.9` (PR #363).

### Fixed

* Improve client connection behavior in `tcp` example (PR #336).
* Tweak grammar in `initialized` and `textDocument/codeAction` doc comments (PR #361).

## [0.17.0] - 2022-04-15

### Added

* Support proposed LSP features with the `proposed` feature flag (PR #330).

### Changed

* Update `lsp-types` from `0.92` to `0.93` (PR #333).

## [0.16.0] - 2022-03-10

### Added

* Support defining custom JSON-RPC requests on `LspService` (PR #313).
* Add compatibility with WASM (PR #309).
* Support alternative async runtimes other than `tokio` when enabling the
  `runtime-agnostic` feature (PR #309).
* Implement `Service<Request, Response = Option<Response>>` for `Client` (PR
  #313).
* Expose `concurrency_level` setting on `Server`, allowing adjustment from the
  default value of 4.
* Add `Request::build()` interface for creating custom requests.
* Add convenient `From` implementations for `jsonrpc::Id`.
* Add `.result()`/`.error()` and `.is_ok()`/`.is_error()` methods to
  `jsonrpc::Response`.

### Changed

* `LspService` now implements `Service<Request, Response = Option<Response>>` .
* `LspService::new()` now returns a `ClientSocket` instead of a `MessageStream`.
* `Server::new()` now requires a third `ClientSocket` argument instead of using
  `.interleave()`.
* Rename `Client::send_custom_{request,notification}` to
  `Client::send_{request,notification}`.
* Rename `jsonrpc::Response::{ok, error}` to
  `jsonrpc::Response::{from_ok, from_error}`.

### Fixed

* Close `Client` channel properly on `exit` notification (PR #309).
* Fix `Server` occasionally stalling by processing client responses separately
  from client-to-server requests (PR #313).
* Return error code `-32600` (invalid request) if incoming data is valid JSON,
  but isn't a JSON-RPC request or response (PR #313).

### Removed

* Remove `.interleave()` method from `Server` (PR #313).
* Remove `jsonrpc::{ClientRequest, Incoming, Outgoing, ServerRequest}` (PR
  #313).
* Remove `MessageStream` (PR #313).

## [0.15.1] - 2022-02-14

### Fixed

* Fix semver incompatibility in release of `tower-lsp-macros` (PR #306).
  * Re-released `tower-lsp-macros` 0.4.2 -> 0.5.0.
  * Re-released `tower-lsp` 0.15.0 -> 0.15.1.
* Update `tokio-util` from `0.6.5` to `0.7.0` (PR #303).

## [0.15.0] - 2022-02-10 [YANKED]

### Changed

* Bump minimum supported Rust version from `1.45.0` to `1.52.0` (PR #300).
* Update `lsp-types` from `0.89` to `0.92` (PR #300).
* Update `auto_impl` from `0.4` to `0.5` (PR #298).
* Update `dashmap` from `4.0.2` to `5.0.0` (PR #298).
* Update `nom` from `6.1.2` to `7.1.0` (PR #298).

### Fixed

* Support `null` and negative integer values as request IDs (PR #285).

## [0.14.1] - 2021-05-21

### Fixed

* Fix regression in server-side `$/cancelRequest` support (PR #280).

## [0.14.0] - 2021-05-20

### Added

* Add support for Language Server Protocol 3.16.0 (PR #270):
  * Implement `workspace/willCreateFiles` server request.
  * Implement `workspace/willRenameFiles` server request.
  * Implement `workspace/willDeleteFiles` server request.
  * Implement `workspace/didCreateFiles` server notification.
  * Implement `workspace/didRenameFiles` server notification.
  * Implement `workspace/didDeleteFiles` server notification.
  * Implement call hierarchy server requests.
  * Implement semantic tokens server requests.
  * Implement `workspace/codeLens/refresh` client request.
  * Implement `workspace/semanticTokens/refresh` client request.
  * Implement `textDocument/linkedEditingRange` server request.
  * Implement `textDocument/moniker` request.
  * Implement `codeAction/resolve` request.
* Add support for custom server-to-client requests (PR #275).

### Changed

* Bump minimum supported Rust version from `1.41.0` to `1.45.0` (PR #264).
* Update `lsp-types` from `0.82` to `0.89` (PR #264).
* Update `tokio` from `0.2` to `1.6` (PR #264, PR #268).
* Update `tokio-util` from `0.3` to `0.6.5` (PR #264).
* Update `bytes` from `0.5` to `1.0.1` (PR #264).
* Update `dashmap` from `3.5.1` to `4.0.2` (PR #264).
* Update `nom` from `5.1` to `6.1.2` (PR #264).
* Eliminate looping, message reparsing in codec using SIMD accelerated
  `take_until` combinator (PR #274).

## Fixed

* Fix race when sending requests to the client (PR #245).
* Permit `window/showMessageRequest` while server is uninitialized (PR #288).
* Fix client request futures hanging by fixing `serde` overlap (PR #269).
* Correctly handle incoming zero-length messages (PR #271).
* Clean up documentation, fix broken intra-doc and external doc links.

## [0.13.3] - 2020-09-19

### Changed

* Increase `lsp-types` semantic version range to `>=0.79, <0.82`. This is safe
  because the upstream changes only concern proposed LSP features, which this
  library does not currently support.

## [0.13.2] - 2020-09-03

### Changed

* Increase `lsp-types` semantic version range to `>=0.79, <0.81`. This is safe
  because the upstream changes only concern proposed LSP features, which this
  library does not currently support.

## [0.13.1] - 2020-08-21

### Changed

* Improve `std::fmt::Debug` implementation for `Client` (PR #216).
* Several API documentation improvements.

### Fixed

* Fix infinite loop upon encountering invalid UTF-8 characters in an incoming
  message (PR #215).

## [0.13.0] - 2020-08-20

### Changed

* Improve log message quality and reduce noise.
* Return responses in corresponding request order to make things easier on the
  client side (PR #212).
* Change remaining `Client` notification methods to `async fn` (PR #213).
* Bump minimum supported Rust version to 1.41.0 (PR #213).

### Fixed

* Report missing params as "invalid params" instead of "parse error" (PR #211).

## [0.12.1] - 2020-08-11

### Fixed

* Reject multiple `initialize` requests sent in quick succession (PR #208).
* Fix bug deserializing `jsonrpc` field from `serde_json::Value` (PR #209).

## [0.12.0] - 2020-08-09

### Added

* Add private subcrate `tower-lsp-macros` for internal use only (PR #202).
* Implement cancellation support via `$/cancelRequest` (PR #202).
* Officially support serving over TCP (PR #198).

### Changed

* Update `lsp-types` crate from 0.74 to 0.79.
* Have language servers store `Client` directly as struct field (PR #199).
* Replace `jsonrpc-core` with minimal JSON-RPC implementation (PR #202).
* Redefine `LspService` as `Service<Incoming, Response = Option<Outgoing>>`.
* Implement `FusedStream` for `MessageStream`.

### Fixed

* Fix typo which caused `workspace/didChangeConfiguration` to break (PR #195).
* Implement proper parse error recovery in LSP codec (PR #201).
* Refuse to accept further requests after `shutdown` has been called once.

### Removed

* Remove dependency on `jsonrpc-core`, as `tower-lsp` no longer relies on it.
* Remove `LspService::with_handler()` constructor (PR #202).

## [0.11.0] - 2020-04-30

### Changed

* Update `lsp-types` crate from 0.73 to 0.74 (PR #178). This introduces breaking
  changes to the following `LanguageServer` trait method signatures:
  * `hover()`
  * `signatureHelp()`
  * `goto_declaration()`
  * `goto_definition()`
  * `goto_type_definition()`
  * `goto_implementation()`
  * `document_highlight()`
* Make `LanguageServer::initialize()` handler `async fn` (PR #182).
* Accept `stdin` and `stdout` handles that are not `Send + 'static`. This
  permits the use of `std::io::Cursor` or `Vec<u8>` as mock stdio sources for
  tests, and passing in `&mut` handles is now supported as well (PR #184).

### Fixed

* Fix broken bidirectional request/response routing (PR #184). The original
  implementation introduced in [0.9.0](#090---2020-03-04) would deadlock under
  certain conditions.

## [0.10.1] - 2020-04-29

### Changed

* Implement `Clone` for `Client` so it can be safely passed to functions
  expecting `'static` values.
* Mark `MessageStream` as `#[must_use]`.

## [0.10.0] - 2020-03-19

### Added

* Implement support for the following client-to-server messages:
  * `textDocument/willSaveWaitUntil`
  * `textDocument/selectionRange`
* Re-export useful `jsonrpc-core` types in a new `jsonrpc` module (PR #169).

### Changed

* Update `lsp-types` crate from 0.70 to 0.73 (PR #162).

### Fixed

* Fix JSON-RPC delegate for `textDocument/foldingRange` (PR #167).

## [0.9.1] - 2020-03-07

### Added

* Implement support for the following client-to-server messages:
  * `textDocument/documentColor`
  * `textDocument/colorPresentation`
  * `textDocument/rangeFormatting`
  * `textDocument/onTypeFormatting`
  * `textDocument/foldingRange`

### Changed

* Server will accept the `initialize` request from the client only once and will
  respond with JSON-RPC error code `-32600` if sent again (PR #160).

### Fixed

* Fix broken links and improve documentation (PRs #152 #157 #158).

## [0.9.0] - 2020-03-04

### Added

* Add `info!()` message when server initializes to be consistent with the
  existing `info!()` message that is emitted when the server exits.
* Implement support for the following client-to-server messages:
  * `textDocument/references`
  * `textDocument/documentLink`
  * `documentLink/resolve`
  * `textDocument/rename`
  * `textDocument/prepareRename`
* Implement support for the following server-to-client messages:
  * `window/showMessageRequest`
  * `workspace/workspaceFolders`
  * `workspace/configuration`

### Changed

* Improve LSP message encoding efficiency (PR #126).
* Reduce chattiness of `trace!()` logs (PR #130).
* Change all notification trait methods to `async fn` (PR #131).
* Implement proper server-to-client request routing (PRs #134 #135).
* Rename `Printer` to `Client`.
* Change `Client::apply_edit()` to return `Result<ApplyWorkspaceEditResponse>`.
* Change `Client::register_capability()` to return `Result<()>`.
* Change `Client::unregister_capability()` to return `Result<()>`.

### Removed

* Remove redundant serialization steps from `Client` (PR #129).

## [0.8.0] - 2020-02-28

### Added

* Implement support for the following client-to-server messages:
  * `textDocument/willSave`
  * `completionItem/resolve`
  * `textDocument/documentSymbol`
  * `textDocument/codeAction`
  * `textDocument/codeLens`
  * `codeLens/resolve`
  * `textDocument/formatting`

### Changed

* `LspService::call()` stops serving requests after `exit` notification,
  meaning there is no longer a need for `ExitReceiver::run_until_exit` and the
  `Server::serve()` async method can now be awaited directly (PR #117).
* Return `Option<String>` as service response type (PR #116).
* Disable unused `nom` features for a hopefully lighter build (PR #112).
* Link to current version of LSP specification in doc comments (PR #122).

### Fixed

* Correctly handle backpressure using `Service::poll_ready()` (PR #117).

### Removed

* Remove `ExitReceiver` type and `LspService::close_handle()` method (PR #117).

## [0.7.0] - 2020-02-24

### Added

* Add default implementations to all non-required `LanguageServer` methods.
* Add support for emitting custom notifications to clients (PR #99).
* Implement support for the following client-to-server messages:
  * `textDocument/signatureHelp`
  * `textDocument/implementation`

### Changed

* Bump minimum supported Rust version to 1.39.0.
* Convert to `std::future` and async/await (PR #101).
* Update `futures` crate from 0.1.28 to 0.3.
* Update `lsp-types` crate from 0.68.0 to 0.70.
* Update `tokio` crate from 0.1.12 to 0.2.
* Update `tower-service` crate from 0.2.0 to 0.3.

### Fixed

* Fix some incorrect links in doc comments.

## [0.6.0] - 2020-01-07

### Added

* Implement support for the following client-to-server messages:
  * `textDocument/declaration`
  * `textDocument/definition`
  * `textDocument/typeDefinition`

### Changed

* Update `lsp-types` crate from 0.63.1 to 0.68.0.

## [0.5.0] - 2019-12-12

### Added

* Add support for Language Server Protocol 3.15.

### Changed

* Update `lsp-types` crate from 0.61.0 to 0.63.1.

## [0.4.1] - 2019-12-09

### Changed

* Update `jsonrpc-core` crate from 14.0 to 14.0.5.
* Update `jsonrpc-derive` crate from 14.0 to 14.0.5.
* Update `log` crate from 0.4.7 to 0.4.8.
* Update `serde` crate from 1.0.99 to 1.0.103.
* Update `tokio-executor` crate from 0.1.8 to 0.1.9.
* Update `env_logger` crate from 0.7.0 to 0.7.1.

### Fixed

* Correctly handle LSP requests containing incomplete UTF-8 (PR #66).

## [0.4.0] - 2019-10-02

### Added

* Implement support for `textDocument/completion` request.

### Changed

* Expose `Printer` in `LanguageServer::initialize()`.
* Update `env_logger` crate from 0.6.2 to 0.7.0.
* Update `lsp-types` crate from 0.60.0 to 0.61.0.

### Fixed

* Allow `window/logMessage`, `window/showMessage`, and `telemetry/event`
  server-to-client notifications in `initialize` request (PR #48).
* Update links to the LSP specification website to point to the new URL.

## [0.3.1] - 2019-09-08

### Changed

* Use more descriptive message in not initialized JSON-RPC error.
* Initialize example server with available features so it can be used as a
  working mock language server.

### Fixed

* Allow JSON data for `telemetry/event` notification to be null.

## [0.3.0] - 2019-09-05

### Added

* Add support for decoding the optional `Content-Type` field in messages.
* Implement support for the following client-to-server messages:
  * `workspace/didChangeWorkspaceFolders`
  * `workspace/didChangeConfiguration`
  * `workspace/didChangeWatchedFiles`
  * `workspace/symbol`
  * `workspace/executeCommand`
* Implement support for the following server-to-client messages:
  * `telemetry/event`
  * `client/registerCapability`
  * `client/unregisterCapability`
  * `workspace/applyEdit`

### Changed

* Bump minimum Rust version to 1.34.0.
* Rename `highlight()` to `document_highlight()` to better match the
  specification.
* Make all notification methods into provided methods (PR #34).
* Change `LspService` request type from `String` to `Incoming` (PR #28).
* Update `Server` to spawn services with `Incoming` request type.
* Use `env_logger` to print log messages in examples.

### Fixed

* Fix broken doc link to `textDocument/didChange` in `LanguageServer` trait.

## [0.2.0] - 2019-09-03

### Added

* Add `ExitedError` for when calling `LspService` after it has already exited.

### Changed

* Language server now returns server error code `-32002` if any method is called
  before `initialize` request is received, [as per the spec][init].
* `LspService` sets `Service::Error` to `ExitedError`.
* `Server` can now accept any service where `Service::Error` is convertible to
  `Box<dyn Error + Send + Sync>`. This enables compatibility with most Tower
  middleware.
* Retain error or success from future in `ExitReceiver::run_until_exit()`.
* Remove `'static` bounds on some `Server` and `ExitReceiver` methods.

[init]: https://microsoft.github.io/language-server-protocol/specifications/specification-3-14/#initialize

## [0.1.0] - 2019-09-02

### Added

* Initial crate release.
* Implement support for the following message types:
  * `initialize`
  * `initialized`
  * `shutdown`
  * `exit`
  * `window/showMessage`
  * `window/logMessage`
  * `textDocument/publishDiagnostics`
  * `textDocument/didOpen`
  * `textDocument/didChange`
  * `textDocument/didSave`
  * `textDocument/didClose`
  * `textDocument/hover`
  * `textDocument/documentHighlight`

[Unreleased]: https://github.com/ebkalderon/tower-lsp/compare/v0.20.0...HEAD
[0.20.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.19.0...v0.20.0
[0.19.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.18.0...v0.19.0
[0.18.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.17.0...v0.18.0
[0.17.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.16.0...v0.17.0
[0.16.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.15.1...v0.16.0
[0.15.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.15.0...v0.15.1
[0.15.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.14.1...v0.15.0
[0.14.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.14.0...v0.14.1
[0.14.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.13.3...v0.14.0
[0.13.3]: https://github.com/ebkalderon/tower-lsp/compare/v0.13.2...v0.13.3
[0.13.2]: https://github.com/ebkalderon/tower-lsp/compare/v0.13.1...v0.13.2
[0.13.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.12.1...v0.13.0
[0.12.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.9.1...v0.10.0
[0.9.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/ebkalderon/tower-lsp/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ebkalderon/tower-lsp/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ebkalderon/tower-lsp/releases/tag/v0.1.0
