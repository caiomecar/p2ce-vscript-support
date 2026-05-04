mod handlers;
mod positions;
#[allow(
    unused,
    clippy::all,
    clippy::nursery,
    clippy::pedantic,
    clippy::unwrap_used,
    unsafe_code
)]
mod vendored;
use std::{panic::RefUnwindSafe, path::PathBuf};

use anyhow::Result;
use lsp_server::Connection;
use lsp_types::{
    CompletionOptions, Diagnostic, DiagnosticSeverity, DiagnosticTag, DocumentLinkOptions,
    FileChangeType, HoverProviderCapability, InitializeParams, InitializeResult, NumberOrString,
    OneOf, RenameOptions, SemanticTokenModifier, SemanticTokenType, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
    ServerCapabilities, SignatureHelpOptions, TextDocumentSyncCapability, TextDocumentSyncKind,
    TypeDefinitionProviderCapability, Url, WorkDoneProgressOptions,
    notification::{
        Cancel, DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles,
        DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument, LogTrace, SetTrace,
    },
    request::{
        Completion, DocumentLinkRequest, DocumentSymbolRequest, GotoDefinition, GotoTypeDefinition,
        HoverRequest, InlayHintRequest, PrepareRenameRequest, References, Rename,
        SemanticTokensFullRequest, SignatureHelpRequest,
    },
};
use resolver::{Database, FinishedFile, Source as _, VScriptDatabase, VScriptDbConfig, parse};
use salsa::Setter as _;

use crate::vendored::{NotificationRegistry, RequestRegistry, Session};

fn main() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();

    stderrlog::new()
        .modules([module_path!(), "resolver"])
        .verbosity(4)
        .init()
        .expect("It's the first logger we initialise");

    let (id, init_result) = connection.initialize_start()?;
    let params: InitializeParams = serde_json::from_value(init_result)?;

    let config = extract_config(&params);

    let db = Database::new(config);

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
                "[".to_owned(),
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
        document_link_provider: Some(DocumentLinkOptions {
            resolve_provider: Some(false),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        ..Default::default()
    };

    let init_value = serde_json::to_value(InitializeResult {
        capabilities: server_capabilities,
        ..Default::default()
    })?;
    connection.initialize_finish(id, init_value)?;

    let session = Session::new(connection, db);

    let mut request_registry = RequestRegistry::<Database>::default();
    let mut notification_registry = NotificationRegistry::<Database>::default();

    session.main_loop(
        on_requests(&mut request_registry),
        on_notifications(&mut notification_registry),
    )?;
    io_threads.join()?;

    eprintln!("shutting down server");
    Ok(())
}

fn on_requests<Db: VScriptDatabase + Clone + RefUnwindSafe>(
    registry: &mut RequestRegistry<Db>,
) -> &mut RequestRegistry<Db> {
    registry
        .on_important::<Completion>(handlers::handle_completion)
        .on_important::<HoverRequest>(handlers::handle_hover)
        .on_important::<PrepareRenameRequest>(handlers::handle_prepare_rename)
        .on_important::<SemanticTokensFullRequest>(handlers::handle_semantic_tokens)
        .on_important::<SignatureHelpRequest>(handlers::handle_signature_help)
        .on::<DocumentLinkRequest>(handlers::handle_document_link)
        .on::<DocumentSymbolRequest>(handlers::handle_document_symbol)
        .on::<References>(handlers::handle_references)
        .on::<GotoDefinition>(handlers::handle_go_to_definition)
        .on::<GotoTypeDefinition>(handlers::handle_go_to_type_definition)
        .on::<InlayHintRequest>(handlers::handle_inlay_hint)
        .on::<Rename>(handlers::handle_rename)
}

fn on_notifications<Db: VScriptDatabase + Clone + RefUnwindSafe>(
    registry: &mut NotificationRegistry<Db>,
) -> &mut NotificationRegistry<Db> {
    registry
        .on_mut::<DidOpenTextDocument>(|session, params| {
            let uri = &params.text_document.uri;
            session.db.open_file(uri, params.text_document.text);
            session.schedule_diagnostics(uri.clone(), compute_diagnostics);
            Ok(())
        })
        .on_mut::<DidChangeTextDocument>(|session, params| {
            let uri = &params.text_document.uri;
            let file = session
                .db
                .get_file(uri)
                .ok_or_else(|| anyhow::format_err!("File not found in workspace"))?;

            let mut text = file.text(&session.db).clone();

            let line_index = positions::line_index(&session.db, file);
            for change in params.content_changes {
                let range = change.range.expect("Incremental changes always have range");
                let Some(text_range) = positions::text_range(line_index, range) else {
                    continue;
                };
                text.replace_range(std::ops::Range::<usize>::from(text_range), &change.text);
            }

            file.set_text(&mut session.db).to(text);

            session.schedule_diagnostics(uri.clone(), compute_diagnostics);

            Ok(())
        })
        .on_mut::<DidChangeConfiguration>(|session, params| {
            let settings = params.settings;

            let Some(value) = settings.get("tf2Root") else {
                return Ok(());
            };

            let tf2_root_path = value.as_str().and_then(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(v))
                }
            });

            session.db.update_tf2_root(tf2_root_path);
            Ok(())
        })
        .on_mut::<DidChangeWatchedFiles>(|session, params| {
            for change in params.changes {
                if change.typ != FileChangeType::CHANGED {
                    continue;
                }

                let uri = &change.uri;

                if session.db.get_file(uri).is_some() {
                    session.schedule_diagnostics(uri.clone(), compute_diagnostics);
                }
            }
            Ok(())
        })
        .on_mut::<Cancel>(|s, p| {
            let id: lsp_server::RequestId = match p.id {
                NumberOrString::Number(id) => id.into(),
                NumberOrString::String(id) => id.into(),
            };
            if let Some(response) = s.req_queue.incoming.cancel(id) {
                s.connection.sender.send(response.into())?;
            }
            Ok(())
        })
        .on::<DidSaveTextDocument>(|_s, _p| Ok(()))
        .on::<DidCloseTextDocument>(|_s, _p| Ok(()))
        .on::<SetTrace>(|_s, _p| Ok(()))
        .on::<LogTrace>(|_s, _p| Ok(()))
}

fn compute_diagnostics<Db: VScriptDatabase>(db: &Db, url: &Url) -> Result<Vec<Diagnostic>> {
    let file = db
        .get_file(url)
        .ok_or_else(|| anyhow::format_err!("File not found in workspace"))?;

    let line_idx = positions::line_index(db, file);
    let parse = parse(db, file);
    let finished_file = FinishedFile::new(db, file);

    Ok(parse
        .errors()
        .iter()
        .filter_map(|error| {
            Some(Diagnostic {
                message: error.message().to_owned(),
                range: positions::range(line_idx, error.range())?,
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
                range: positions::range(line_idx, diagnostic.range)?,
                severity: Some(severity),
                tags,
                ..Default::default()
            })
        }))
        .collect())
}

fn extract_config(params: &InitializeParams) -> VScriptDbConfig {
    let options = params.initialization_options.as_ref();

    VScriptDbConfig {
        tf2_root_path: options
            .and_then(|o| o.get("tf2RootPath"))
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
        builtins_path: options
            .and_then(|o| o.get("builtinsPath"))
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
        squirrel_lib_path: options
            .and_then(|o| o.get("squirrelLibPath"))
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
        vscript_lib_path: options
            .and_then(|o| o.get("vscriptLibPath"))
            .and_then(|v| v.as_str())
            .map(PathBuf::from),
    }
}
