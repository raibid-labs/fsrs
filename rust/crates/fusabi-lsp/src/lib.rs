use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct FusabiLanguageServer {
    client: Client,
}

impl FusabiLanguageServer {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FusabiLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "fusabi-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Fusabi LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }
}
