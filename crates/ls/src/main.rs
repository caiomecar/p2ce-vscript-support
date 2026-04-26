mod completion;
mod conversions;
mod document_symbols;
mod find_references;
mod go_to_definition;
mod go_to_type_definition;
mod hover;
mod inlay_hints;
mod prepare_rename;
mod rename;
mod semantic_tokens;
mod signature_help;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use lsp_server::{Connection, Message, Request as ServerRequest, RequestId, Response};
use lsp_types::notification::{DidChangeConfiguration, Notification}; // for METHOD consts
use lsp_types::request::{
    DocumentSymbolRequest, GotoDefinition, GotoTypeDefinition, HoverRequest, InlayHintRequest,
    PrepareRenameRequest, References, Rename, Request, SemanticTokensFullRequest,
    SignatureHelpRequest,
};
use lsp_types::{
    CompletionOptions, DiagnosticSeverity, DiagnosticTag, DidChangeConfigurationParams,
    HoverProviderCapability, OneOf, RenameOptions, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, SignatureHelpOptions, TypeDefinitionProviderCapability,
    WorkDoneProgressOptions,
};
use resolver::{Database, DbConfig, File, FinishedFile, Source, line_index, parse};
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
use crate::go_to_type_definition::handle_go_to_type_definition;
use crate::hover::handle_hover;
use crate::inlay_hints::handle_inlay_hints;
use crate::prepare_rename::handle_prepare_rename;
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
            trigger_characters: Some(vec![
                ".".to_owned(),
                "\"".to_owned(),
                "@".to_owned(),
                "{".to_owned(),
                "|".to_owned(),
                "*".to_owned(),
            ]),
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::VARIABLE,
                        SemanticTokenType::PARAMETER,
                        SemanticTokenType::FUNCTION,
                        SemanticTokenType::CLASS,
                        SemanticTokenType::PROPERTY,
                        SemanticTokenType::ENUM,
                        SemanticTokenType::ENUM_MEMBER,
                    ],
                    token_modifiers: vec![
                        SemanticTokenModifier::READONLY,
                        SemanticTokenModifier::STATIC,
                        SemanticTokenModifier::DEPRECATED,
                    ],
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..Default::default()
            },
        )),
        document_symbol_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_owned(), ",".to_owned()]),
            ..Default::default()
        }),
        inlay_hint_provider: Some(OneOf::Left(true)),
        type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
        ..Default::default()
    };

    let init_value = serde_json::to_value(server_capabilities)?;
    let init_params = connection.initialize(init_value)?;

    main_loop(&connection, init_params)?;
    io_threads.join()?;
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(connection: &Connection, params: serde_json::Value) -> Result<()> {
    let init: InitializeParams = serde_json::from_value(params)?;

    let config = match init.initialization_options {
        Some(serde_json::Value::Object(map)) => {
            let tf2_root_path = map.get("tf2Root").and_then(|v| v.as_str()).and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(v))
                }
            });

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
                let method = req.method.clone();
                if let Err(err) = handle_request(&db, connection, req) {
                    eprintln!("[lsp] request {method} failed: {err}");
                }
                eprintln!("[lsp] request {method} took {:?}", now.elapsed());
            }
            Message::Notification(note) => {
                let now = Instant::now();
                let method = note.method.clone();
                if let Err(err) = handle_notification(&mut db, connection, note) {
                    eprintln!("[lsp] notification {method} failed: {err}");
                }
                eprintln!("[lsp] notification {method} took {:?}", now.elapsed());
            }
            Message::Response(resp) => eprintln!("[lsp] response: {resp:?}"),
        }
    }
    Ok(())
}

fn handle_notification(
    db: &mut Database,
    conn: &Connection,
    note: lsp_server::Notification,
) -> Result<()> {
    match note.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = serde_json::from_value(note.params)?;
            let Ok(path) = p.text_document.uri.to_file_path() else {
                return Ok(()); // ignore untitled files
            };

            let file = db
                .get_file(&path)
                .unwrap_or_else(|| db.open_file_with_text(path, p.text_document.text));

            publish_diagnostics(db, conn, file)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = serde_json::from_value(note.params)?;
            let Ok(path) = p.text_document.uri.to_file_path() else {
                return Ok(()); // ignore untitled files
            };

            let Some(file) = db.get_file(&path) else {
                return Ok(());
            };

            let mut text = file.text(db).clone();
            let line_index = line_index(db, file);
            for change in p.content_changes {
                let range = change.range.expect("Incremental changes always have range");
                let Some(text_range) = conversions::text_range(line_index, range) else {
                    continue;
                };
                text.replace_range(std::ops::Range::<usize>::from(text_range), &change.text);
            }
            file.set_text(db).to(text);

            publish_diagnostics(db, conn, file)?;
        }
        DidChangeConfiguration::METHOD => {
            let p: DidChangeConfigurationParams = serde_json::from_value(note.params)?;
            let settings = p.settings;

            let tf2_root_path = settings
                .get("tf2Root")
                .and_then(|v| v.as_str())
                .and_then(|v| {
                    if v.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(v))
                    }
                });

            db.update_tf2_root(tf2_root_path);

            for (file, _) in &db.all_files() {
                publish_diagnostics(db, conn, *file)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_request(db: &Database, conn: &Connection, req: ServerRequest) -> Result<()> {
    match req.method.as_str() {
        Completion::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_completions(db, params);
            send_ok(conn, req.id, &result)?;
        }
        GotoDefinition::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_go_to_definition(db, params);
            send_ok(conn, req.id, &result)?;
        }
        HoverRequest::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_hover(db, params);
            send_ok(conn, req.id, &result)?;
        }
        SemanticTokensFullRequest::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_semantic_tokens(db, params);
            send_ok(conn, req.id, &result)?;
        }
        DocumentSymbolRequest::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_document_symbols(db, params);
            send_ok(conn, req.id, &result)?;
        }
        References::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_references(db, params);
            send_ok(conn, req.id, &result)?;
        }
        Rename::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_rename(db, params);
            send_ok(conn, req.id, &result)?;
        }
        PrepareRenameRequest::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_prepare_rename(db, params);
            send_ok(conn, req.id, &result)?;
        }
        SignatureHelpRequest::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_signature_help(db, params);
            send_ok(conn, req.id, &result)?;
        }
        InlayHintRequest::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_inlay_hints(db, params);
            send_ok(conn, req.id, &result)?;
        }
        GotoTypeDefinition::METHOD => {
            let params = serde_json::from_value(req.params)?;
            let result = handle_go_to_type_definition(db, params);
            send_ok(conn, req.id, &result)?;
        }
        _ => send_err(
            conn,
            req.id,
            lsp_server::ErrorCode::MethodNotFound,
            "unhandled method",
        )?,
    }
    Ok(())
}

fn publish_diagnostics(db: &Database, conn: &Connection, file: File) -> Result<()> {
    let line_idx = line_index(db, file);
    let parse = parse(db, file);
    let finished_file = FinishedFile::new(db, file);

    let diagnostics = parse
        .errors()
        .iter()
        .filter_map(|error| {
            Some(Diagnostic {
                message: error.message().to_owned(),
                range: conversions::range(line_idx, error.range())?,
                ..Default::default()
            })
        })
        .chain(finished_file.diagnostics().iter().filter_map(|diagnostic| {
            let (severity, tags) = match diagnostic.severity {
                resolver::DiagnosticSeverity::Error => (DiagnosticSeverity::ERROR, None),
                resolver::DiagnosticSeverity::Warning => (DiagnosticSeverity::WARNING, None),
                resolver::DiagnosticSeverity::Information => {
                    (DiagnosticSeverity::INFORMATION, None)
                }
                resolver::DiagnosticSeverity::Unnecessary => (
                    DiagnosticSeverity::WARNING,
                    Some(vec![DiagnosticTag::UNNECESSARY]),
                ),
                resolver::DiagnosticSeverity::Deprecated => (
                    DiagnosticSeverity::HINT,
                    Some(vec![DiagnosticTag::DEPRECATED]),
                ),
            };
            Some(Diagnostic {
                message: diagnostic.message.clone(),
                range: conversions::range(line_idx, diagnostic.range)?,
                severity: Some(severity),
                tags,
                ..Default::default()
            })
        }))
        .collect();

    let Some(uri) = db.get_path(file).and_then(|p| conversions::to_uri(&p)) else {
        return Ok(());
    };

    let params = PublishDiagnosticsParams {
        uri,
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
