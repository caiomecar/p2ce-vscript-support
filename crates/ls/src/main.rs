mod conversions;

use anyhow::Result;
use ide::{Database, File, SymbolKind, Type, line_index, parse, source_symbol};
use lsp_server::{Connection, Message, Request as ServerRequest, RequestId, Response};
use lsp_types::notification::Notification as _; // for METHOD consts
use lsp_types::request::Request as _;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
}; // for METHOD consts
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
use sq_3_parser::{AstNode, ast}; // for METHOD consts

fn main() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        completion_provider: Some(CompletionOptions::default()),
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
    let _init: InitializeParams = serde_json::from_value(params)?;
    let mut db = Database::default();
    let mut docs: FxHashMap<Url, File> = FxHashMap::default();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                if let Err(err) = handle_request(&connection, &req, &db, &mut docs) {
                    eprintln!("[lsp] request {} failed: {err}", &req.method);
                }
            }
            Message::Notification(note) => {
                if let Err(err) = handle_notification(&connection, &note, &mut db, &mut docs) {
                    eprintln!("[lsp] notification {} failed: {err}", note.method);
                }
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
) -> Result<()> {
    match note.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = serde_json::from_value(note.params.clone())?;
            let uri = p.text_document.uri;
            let file = File::new(db, p.text_document.text);
            docs.insert(uri.clone(), file);
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
) -> Result<()> {
    match req.method.as_str() {
        Completion::METHOD => {
            let params: CompletionParams = serde_json::from_value(req.params.clone())?;
            let uri = params.text_document_position.text_document.uri;
            let Some(&file) = docs.get(&uri) else {
                return Ok(());
            };

            let source_symbol = source_symbol(db, file);

            let line_index = line_index(db, file);

            let offset =
                conversions::test_size(line_index, params.text_document_position.position).unwrap();

            let source_file = parse(db, file).syntax();
            let token = source_file.token_at_offset(offset).left_biased();

            let member_access = token.and_then(|t| {
                t.parent_ancestors()
                    .find_map(|node| ast::MemberAccessExpression::cast(node))
            });

            let symbols = if let Some(member) = member_access {
                // get the object (abc) — the part before the dot
                if let Some(obj) = member.object() {
                    let type_ = source_symbol.type_at(obj.syntax().text_range());

                    source_symbol.members_of_type(type_)
                } else {
                    Vec::new()
                }
            } else {
                source_symbol.symbols_at(offset)
            };

            let items: Vec<CompletionItem> = symbols
                .iter()
                .filter_map(|&symbol| {
                    Some(CompletionItem {
                        label: symbol.name.clone(),
                        kind: Some(match symbol.typ {
                            Type::Enum(_) => CompletionItemKind::ENUM,
                            Type::Function(_) => CompletionItemKind::FUNCTION,
                            Type::Class(_) => CompletionItemKind::CLASS,
                            _ => match symbol.kind {
                                SymbolKind::Local => CompletionItemKind::VARIABLE,
                                SymbolKind::Constant => CompletionItemKind::CONSTANT,
                                SymbolKind::Property => CompletionItemKind::PROPERTY,
                                SymbolKind::Enum => CompletionItemKind::ENUM,
                                SymbolKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
                            },
                        }),
                        ..Default::default()
                    })
                })
                .collect();

            send_ok(conn, req.id.clone(), &CompletionResponse::Array(items))?;
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
