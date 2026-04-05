mod completion;
mod conversions;
mod document_symbols;
mod find_references;
mod go_to_definition;
mod hover;
mod rename;
mod semantic_tokens;
mod signature_help;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use ide::{Database, DbConfig, File, FinishedFile, Source, line_index, parse};
use lsp_server::{Connection, Message, Request as ServerRequest, RequestId, Response};
use lsp_types::notification::Notification as _; // for METHOD consts
use lsp_types::request::{
    DocumentSymbolRequest, GotoDefinition, HoverRequest, References, Rename, Request,
    SemanticTokensFullRequest, SignatureHelpRequest,
};
use lsp_types::{
    CompletionOptions, CompletionParams, DiagnosticSeverity, DiagnosticTag, DocumentSymbolParams,
    GotoDefinitionParams, HoverParams, HoverProviderCapability, OneOf, ReferenceParams,
    RenameParams, SemanticTokenModifier, SemanticTokenType, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensServerCapabilities, SignatureHelpOptions, SignatureHelpParams,
};
// for METHOD consts
use lsp_types::{
    Diagnostic,
    DidChangeTextDocumentParams,
    DidOpenTextDocumentParams,
    // core
    InitializeParams,
    PublishDiagnosticsParams,
    ServerCapabilities,
    TextDocumentSyncCapability,
    TextDocumentSyncKind,
    // notifications
    notification::{DidChangeTextDocument, DidOpenTextDocument, PublishDiagnostics},
    request::Completion,
};
use salsa::Setter;

use crate::completion::handle_completions;
use crate::document_symbols::handle_document_symbols;
use crate::find_references::handle_references;
use crate::go_to_definition::handle_go_to_definition;
use crate::hover::handle_hover;
use crate::rename::handle_rename;
use crate::semantic_tokens::handle_semantic_tokens;
use crate::signature_help::handle_signature_help;

fn main() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".to_owned(), "[".to_owned(), "\"".to_owned()]),
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::VARIABLE,
                        SemanticTokenType::FUNCTION,
                        SemanticTokenType::CLASS,
                        SemanticTokenType::PROPERTY,
                        SemanticTokenType::ENUM,
                        SemanticTokenType::ENUM_MEMBER,
                        SemanticTokenType::PARAMETER,
                    ],
                    token_modifiers: vec![SemanticTokenModifier::READONLY],
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..Default::default()
            },
        )),
        document_symbol_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Left(true)),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_owned(), ",".to_owned()]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let init_value = serde_json::to_value(server_capabilities)?;
    let init_params = connection.initialize(init_value)?;

    main_loop(connection, init_params)?;
    io_threads.join()?;
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    let init: InitializeParams = serde_json::from_value(params)?;

    let config = match init.initialization_options {
        Some(serde_json::Value::Object(map)) => {
            let tf2_root_path = map
                .get("tf2Root")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);

            let builtins_path = map
                .get("builtinsPath")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);

            let squirrel_lib_path = map
                .get("squirrelLibPath")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);

            let vscript_lib_path = map
                .get("vscriptLibPath")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);

            DbConfig {
                tf2_root_path,
                builtins_path,
                squirrel_lib_path,
                vscript_lib_path,
            }
        }
        _ => DbConfig::default(),
    };

    let mut db = Database::new(config);

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                let now = Instant::now();
                if let Err(err) = handle_request(&db, &connection, &req) {
                    eprintln!("[lsp] request {} failed: {err}", &req.method);
                }
                eprintln!("Handling request took {:?}", now.elapsed());
            }
            Message::Notification(note) => {
                let now = Instant::now();
                if let Err(err) = handle_notification(&mut db, &connection, &note) {
                    eprintln!("[lsp] notification {} failed: {err}", note.method);
                }
                eprintln!("Handling notification took {:?}", now.elapsed());
            }
            Message::Response(resp) => eprintln!("[lsp] response: {resp:?}"),
        }
    }
    Ok(())
}

fn handle_notification(
    db: &mut Database,
    conn: &Connection,
    note: &lsp_server::Notification,
) -> Result<()> {
    match note.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = serde_json::from_value(note.params.clone())?;
            let Ok(path) = p.text_document.uri.to_file_path() else {
                return Ok(()); // ignore untitled files
            };
            let file = db.open_file_with_text(path, p.text_document.text);
            publish_diagnostics(&db, conn, file)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = serde_json::from_value(note.params.clone())?;
            let Ok(path) = p.text_document.uri.to_file_path() else {
                return Ok(()); // ignore untitled files
            };

            let Some(file) = db.get_file(&path) else {
                return Ok(());
            };

            let mut text = file.text(db).to_string();
            let line_index = line_index(db, file);
            for change in p.content_changes {
                let range = change.range.expect("Incremental changes always have range");
                let text_range = conversions::text_range(line_index, range).unwrap();
                text.replace_range(std::ops::Range::<usize>::from(text_range), &change.text);
            }
            file.set_text(db).to(text);

            publish_diagnostics(db, conn, file)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_request(db: &Database, conn: &Connection, req: &ServerRequest) -> Result<()> {
    match req.method.as_str() {
        Completion::METHOD => {
            let params: CompletionParams = serde_json::from_value(req.params.clone())?;
            let result = handle_completions(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        GotoDefinition::METHOD => {
            let params: GotoDefinitionParams = serde_json::from_value(req.params.clone())?;
            let result = handle_go_to_definition(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        HoverRequest::METHOD => {
            let params: HoverParams = serde_json::from_value(req.params.clone())?;
            let result = handle_hover(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        SemanticTokensFullRequest::METHOD => {
            let params: SemanticTokensParams = serde_json::from_value(req.params.clone())?;
            let result = handle_semantic_tokens(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        DocumentSymbolRequest::METHOD => {
            let params: DocumentSymbolParams = serde_json::from_value(req.params.clone())?;
            let result = handle_document_symbols(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        References::METHOD => {
            let params: ReferenceParams = serde_json::from_value(req.params.clone())?;
            let result = handle_references(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        Rename::METHOD => {
            let params: RenameParams = serde_json::from_value(req.params.clone())?;
            let result = handle_rename(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        SignatureHelpRequest::METHOD => {
            let params: SignatureHelpParams = serde_json::from_value(req.params.clone())?;
            let result = handle_signature_help(db, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        _ => send_err(
            conn,
            req.id.clone(),
            lsp_server::ErrorCode::MethodNotFound,
            "unhandled method",
        )?,
    }
    Ok(())
}

fn publish_diagnostics(db: &Database, conn: &Connection, file: File) -> Result<()> {
    let line_index = line_index(db, file);
    let parse = parse(db, file);
    let finished_file = FinishedFile::new(db, file);

    let diagnostics = parse
        .errors()
        .iter()
        .map(|error| Diagnostic {
            message: error.message().to_owned(),
            range: conversions::range(&line_index, error.range()).unwrap(),
            ..Default::default()
        })
        .chain(finished_file.diagnostics().iter().map(|diagnostic| {
            let (severity, tags) = match diagnostic.severity {
                ide::DiagnosticSeverity::Error => (DiagnosticSeverity::ERROR, None),
                ide::DiagnosticSeverity::Warning => (DiagnosticSeverity::WARNING, None),
                ide::DiagnosticSeverity::Information => (DiagnosticSeverity::INFORMATION, None),
                ide::DiagnosticSeverity::Unnecessary => (
                    DiagnosticSeverity::WARNING,
                    Some(vec![DiagnosticTag::UNNECESSARY]),
                ),
            };
            Diagnostic {
                message: diagnostic.message.to_owned(),
                range: conversions::range(&line_index, diagnostic.range).unwrap(),
                severity: Some(severity),
                tags,
                ..Default::default()
            }
        }))
        .collect();

    let Some(path) = db.get_path(file) else {
        return Ok(());
    };

    let params = PublishDiagnosticsParams {
        uri: conversions::to_uri(&path),
        diagnostics,
        version: None,
    };

    conn.sender
        .send(Message::Notification(lsp_server::Notification::new(
            PublishDiagnostics::METHOD.to_owned(),
            params,
        )))?;

    Ok(())
}

fn send_ok<T: serde::Serialize>(conn: &Connection, id: RequestId, result: &T) -> Result<()> {
    let resp = Response {
        id,
        result: Some(serde_json::to_value(result)?),
        error: None,
    };
    conn.sender.send(Message::Response(resp))?;
    Ok(())
}

fn send_err(
    conn: &Connection,
    id: RequestId,
    code: lsp_server::ErrorCode,
    msg: &str,
) -> Result<()> {
    let resp = Response {
        id,
        result: None,
        error: Some(lsp_server::ResponseError {
            code: code as i32,
            message: msg.into(),
            data: None,
        }),
    };
    conn.sender.send(Message::Response(resp))?;
    Ok(())
}
