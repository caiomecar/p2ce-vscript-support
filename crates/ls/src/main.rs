mod conversions;

use anyhow::Result;
use ide::{
    Database, File, FileState, SourceSymbolic, SymbolKind, Type, line_index, parse, source_symbol,
};
use lsp_server::{Connection, Message, Request as ServerRequest, RequestId, Response};
use lsp_types::notification::Notification as _; // for METHOD consts
use lsp_types::request::Request as _;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
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
use sq_3_parser::{AstNode, SyntaxKind, ast};

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
    let mut db = Database::default();
    let mut docs: FxHashMap<Url, File> = FxHashMap::default();

    if let Some(options) = init.initialization_options {
        let opts: InitOptions = serde_json::from_value(options).unwrap();

        if let Some(path) = opts.builtins {
            let text = std::fs::read_to_string(path).unwrap();
            db.init_builtins(text);
        }

        if let Some(path) = opts.squirrel_lib {
            let text = std::fs::read_to_string(path).unwrap();
            db.init_squirrel_lib(text);
        }

        if let Some(path) = opts.vscript_lib {
            let text = std::fs::read_to_string(path).unwrap();
            db.init_vscript_lib(text);
        }
    }

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

            let line_index = line_index(db, file);

            let offset =
                conversions::test_size(line_index, params.text_document_position.position).unwrap();

            let syntax = parse(db, file).syntax();

            let token = syntax.token_at_offset(offset).left_biased().and_then(|t| {
                if t.kind() == SyntaxKind::Whitespace {
                    t.prev_token()
                } else {
                    Some(t)
                }
            });

            let member = token.and_then(|t| {
                t.parent_ancestors()
                    .find_map(|n| ast::MemberAccessExpression::cast(n))
            });

            let file_state = FileState::Finished(db, file);

            // 3. Decide if we're in member access
            let symbols = if let Some(member) = member {
                let range = member.syntax().text_range();

                if offset <= range.end() {
                    // still inside or right after member access → show members
                    if let Some(obj) = member.object() {
                        let typ = file_state.type_at(obj.syntax().text_range());
                        file_state.members_of_type(typ)
                    } else {
                        Vec::new()
                    }
                } else {
                    // past the expression → normal completion
                    file_state.symbols_at(offset)
                }
            } else {
                file_state.symbols_at(offset)
            };

            let items: Vec<CompletionItem> = symbols
                .iter()
                .filter_map(|symbol| {
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
