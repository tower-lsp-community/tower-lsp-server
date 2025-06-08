//! Shows how to send custom notification using `Client::send_notification`.

use serde::{Deserialize, Serialize};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::{Error, Result},
    lsp_types::{notification::Notification, *},
};

#[derive(Debug, Deserialize, Serialize)]
struct CustomNotificationParams {
    title: String,
    message: String,
}

impl CustomNotificationParams {
    fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
        }
    }
}

enum CustomNotification {}

impl Notification for CustomNotification {
    type Params = CustomNotificationParams;

    const METHOD: &'static str = "custom/notification";
}

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["custom.notification".to_string()],
                    ..Default::default()
                }),
                ..ServerCapabilities::default()
            },
            #[cfg(feature = "proposed")]
            offset_encoding: None,
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        if params.command == "custom.notification" {
            let custom_notification = CustomNotificationParams::new("Hello", "Message");
            self.client
                .send_notification::<CustomNotification>(custom_notification)
                .await;

            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Command executed with params: {params:?}"),
                )
                .await;

            Ok(None)
        } else {
            Err(Error::invalid_request())
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
