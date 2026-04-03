mod completion;
mod conversions;
mod document_symbols;
mod go_to_definiton;
mod hover;
mod semantic_tokens;

use std::time::Instant;

use anyhow::Result;
use ide::{Database, File, SourceSymbolic, line_index, parse, source_symbol};
use lsp_server::{Connection, Message, Request as ServerRequest, RequestId, Response};
use lsp_types::notification::Notification as _; // for METHOD consts
use lsp_types::request::{
    DocumentSymbolRequest, GotoDefinition, HoverRequest, Request, SemanticTokensFullRequest,
};
use lsp_types::{
    CompletionOptions, CompletionParams, DocumentSymbolParams, GotoDefinitionParams, HoverParams,
    HoverProviderCapability, OneOf, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensServerCapabilities,
};
use serde::Deserialize; // for METHOD consts
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
    Url,
    // notifications
    notification::{DidChangeTextDocument, DidOpenTextDocument, PublishDiagnostics},
    request::Completion,
};
use salsa::Setter;

use rustc_hash::FxHashMap;

use crate::completion::handle_completions;
use crate::document_symbols::handle_document_symbols;
use crate::go_to_definiton::handle_go_to_definition;
use crate::hover::handle_hover;
use crate::semantic_tokens::handle_semantic_tokens;

#[derive(Deserialize)]
struct InitOptions {
    #[serde(rename = "builtinsPath")]
    builtins: Option<String>,
    #[serde(rename = "squirrelLibPath")]
    squirrel_lib: Option<String>,
    #[serde(rename = "vscriptLibPath")]
    vscript_lib: Option<String>,
}

fn main() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".to_owned(), "[".to_owned()]),
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
        ..Default::default()
    };

    let init_value = serde_json::to_value(server_capabilities)?;
    let init_params = connection.initialize(init_value)?;

    main_loop(connection, init_params)?;
    io_threads.join()?;
    eprintln!("shutting down server");
    Ok(())
}

fn insert_path(
    path: &str,
    file: File,
    docs: &mut FxHashMap<Url, File>,
    file_to_url: &mut FxHashMap<File, Url>,
) {
    match Url::from_file_path(&path) {
        Ok(url) => {
            docs.insert(url.clone(), file);
            file_to_url.insert(file, url);
        }
        Err(_) => {
            eprintln!("Couldn't convert path {path} to url");
        }
    };
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    let init: InitializeParams = serde_json::from_value(params)?;
    let mut db = Database::default();
    let mut docs: FxHashMap<Url, File> = FxHashMap::default();
    let mut file_to_url: FxHashMap<File, Url> = FxHashMap::default();

    if let Some(options) = init.initialization_options {
        let opts: InitOptions = serde_json::from_value(options).unwrap();

        if let Some(path) = opts.builtins {
            let text = std::fs::read_to_string(path.clone()).unwrap();
            let file = db.init_builtins(text);
            insert_path(&path, file, &mut docs, &mut file_to_url);
        }

        if let Some(path) = opts.squirrel_lib {
            let text = std::fs::read_to_string(path.clone()).unwrap();
            let file = db.init_squirrel_lib(text);
            insert_path(&path, file, &mut docs, &mut file_to_url);
        }

        if let Some(path) = opts.vscript_lib {
            let text = std::fs::read_to_string(path.clone()).unwrap();
            let file = db.init_vscript_lib(text);
            insert_path(&path, file, &mut docs, &mut file_to_url);
        }
    }

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                let now = Instant::now();
                if let Err(err) = handle_request(&connection, &req, &db, &docs, &file_to_url) {
                    eprintln!("[lsp] request {} failed: {err}", &req.method);
                }
                eprintln!("Handling request took {:?}", now.elapsed());
            }
            Message::Notification(note) => {
                let now = Instant::now();
                if let Err(err) =
                    handle_notification(&connection, &note, &mut db, &mut docs, &mut file_to_url)
                {
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
    conn: &Connection,
    note: &lsp_server::Notification,
    db: &mut Database,
    docs: &mut FxHashMap<Url, File>,
    file_to_url: &mut FxHashMap<File, Url>,
) -> Result<()> {
    match note.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = serde_json::from_value(note.params.clone())?;
            let uri = p.text_document.uri;
            let file = File::new(db, p.text_document.text);
            docs.insert(uri.clone(), file);
            file_to_url.insert(file, uri.clone());
            publish_diagnostics(conn, db, uri, file)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = serde_json::from_value(note.params.clone())?;
            let uri = p.text_document.uri;
            let Some(&file) = docs.get(&uri) else {
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
            publish_diagnostics(conn, db, uri, file)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_request(
    conn: &Connection,
    req: &ServerRequest,
    db: &Database,
    docs: &FxHashMap<Url, File>,
    file_to_url: &FxHashMap<File, Url>,
) -> Result<()> {
    match req.method.as_str() {
        Completion::METHOD => {
            let params: CompletionParams = serde_json::from_value(req.params.clone())?;
            let result = handle_completions(db, docs, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        GotoDefinition::METHOD => {
            let params: GotoDefinitionParams = serde_json::from_value(req.params.clone())?;
            let result = handle_go_to_definition(db, docs, file_to_url, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        HoverRequest::METHOD => {
            let params: HoverParams = serde_json::from_value(req.params.clone())?;
            let result = handle_hover(db, docs, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        SemanticTokensFullRequest::METHOD => {
            let params: SemanticTokensParams = serde_json::from_value(req.params.clone())?;
            let result = handle_semantic_tokens(db, docs, params)?;
            send_ok(conn, req.id.clone(), &result)?;
        }
        DocumentSymbolRequest::METHOD => {
            let params: DocumentSymbolParams = serde_json::from_value(req.params.clone())?;
            let result = handle_document_symbols(db, docs, params)?;
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

fn publish_diagnostics(conn: &Connection, db: &Database, uri: Url, file: File) -> Result<()> {
    let parse = parse(db, file);
    let source_symbol = source_symbol(db, file);
    let line_index = line_index(db, file);

    let diagnostics = parse
        .errors()
        .iter()
        .map(|error| Diagnostic {
            message: error.message().to_owned(),
            range: conversions::range(&line_index, error.range()).unwrap(),
            ..Default::default()
        })
        .chain(
            source_symbol
                .diagnostics()
                .iter()
                .map(|diagnostic| Diagnostic {
                    message: diagnostic.message.to_owned(),
                    range: conversions::range(&line_index, diagnostic.range).unwrap(),
                    severity: Some(conversions::to_lsp_severity(diagnostic.severity)),
                    ..Default::default()
                }),
        )
        .collect();

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
