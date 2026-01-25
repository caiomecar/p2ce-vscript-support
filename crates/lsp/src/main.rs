use dashmap::DashMap;
use sq_3_parser::{Parse, SyntaxError};
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, Position, Range,
    ServerCapabilities, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct Backend {
    client: Client,
    documents: DashMap<Url, String>,
}

impl Backend {
    pub fn new(client: Client) -> Backend {
        Backend {
            client,
            documents: DashMap::new(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncKind::INCREMENTAL.into()),
                ..Default::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        let parse = Parse::new(&text);
        self.client
            .publish_diagnostics(
                uri.clone(),
                syntax_errors_to_diagnostic(&text, parse.errors()),
                None,
            )
            .await;
        self.documents.insert(uri, text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let mut text = self.documents.get_mut(&uri).expect("Document not found");
        for change in params.content_changes {
            let range = change.range.expect("incremental change");
            let start = position_to_byte_offset(&text, range.start);
            let end = position_to_byte_offset(&text, range.end);
            text.replace_range(start..end, &change.text);
        }
        let parse = Parse::new(&text);

        self.client
            .publish_diagnostics(
                uri,
                syntax_errors_to_diagnostic(&text, parse.errors()),
                None,
            )
            .await;
        // eprintln!("{:#?}", parse.into_syntax());
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.remove(&uri);
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn syntax_errors_to_diagnostic(text: &str, errors: &[SyntaxError]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for error in errors {
        let range = error.range();
        let start = byte_offset_to_position(text, range.start().into());
        let end = byte_offset_to_position(text, range.end().into());

        diagnostics.push(Diagnostic {
            range: Range::new(start, end),
            message: error.message().to_string(),
            ..Default::default()
        })
    }

    diagnostics
}

fn position_to_byte_offset(text: &str, pos: Position) -> usize {
    let mut line = 0u32;
    let mut col_utf16 = 0u32;
    let mut byte_offset = 0usize;

    for ch in text.chars() {
        if line == pos.line && col_utf16 >= pos.character {
            break;
        }

        if ch == '\n' {
            line += 1;
            col_utf16 = 0;
        } else {
            col_utf16 += ch.len_utf16() as u32;
        }

        byte_offset += ch.len_utf8();
    }

    byte_offset
}

fn byte_offset_to_position(text: &str, offset: usize) -> Position {
    let mut line = 0;
    let mut col = 0;

    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    Position::new(line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        byte_offset_to_position("", 0);
        position_to_byte_offset("", Position::new(0, 0));
    }
}
