use ide::{
    Database, FindSymbol, FinishedFile, ImportMembers, Source, Symbol, SymbolFlags, SymbolKind,
    Type, line_index, parse,
};
use lsp_types::{
    Command, CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams,
    CompletionResponse, CompletionTextEdit, InsertTextFormat, TextEdit,
};
use sq_3_parser::{AstNode, KEYWORDS, SyntaxKind, SyntaxNode, TextRange, TextSize, ast};

use crate::conversions;

#[allow(clippy::too_many_lines)]
pub fn handle_completions(db: &Database, params: CompletionParams) -> CompletionResponse {
    let uri = params.text_document_position.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return CompletionResponse::Array(Vec::new());
    };

    let Some(file) = db.get_file(&path) else {
        return CompletionResponse::Array(Vec::new());
    };

    let line_idx = line_index(db, file);

    let offset = conversions::test_size(line_idx, params.text_document_position.position);
    let syntax = parse(db, file).syntax();

    let finished_file = FinishedFile::new(db, file);

    let scope = finished_file.scope(offset);
    let trigger_char = params.context.and_then(|c| c.trigger_character);

    CompletionResponse::Array(
        match context_completions(&syntax, offset, trigger_char.as_deref(), &finished_file) {
            Some(ContextCompletions::Flat) => finished_file
                .symbols_at(offset, true)
                .into_iter()
                .map(|(mut label, id)| {
                    let symbol = finished_file.get(id);
                    let kind = Some(to_completion_kind(symbol));
                    let mut insert_text = None;
                    let (insert_text_format, command) =
                        modify_if_function(&finished_file, symbol, &mut label, &mut insert_text)
                            .map_or((None, None), |(a, b)| (Some(a), Some(b)));

                    CompletionItem {
                        label,
                        kind,
                        insert_text,
                        command,
                        insert_text_format,
                        tags: symbol
                            .flags
                            .contains(SymbolFlags::DEPRECATED)
                            .then(|| vec![CompletionItemTag::DEPRECATED]),
                        ..Default::default()
                    }
                })
                .collect(),
            Some(ContextCompletions::FromObject { typ, prefix_range }) => finished_file
                .members_of_type(
                    typ,
                    FindSymbol::BeforeIfInExecutionRange(offset, scope),
                    true,
                )
                .into_iter()
                .map(|(mut label, id)| {
                    let symbol = finished_file.get(id);
                    let kind = Some(to_completion_kind(symbol));
                    if can_use_identifier(&label) {
                        let mut insert_text = None;
                        let (insert_text_format, command) = modify_if_function(
                            &finished_file,
                            symbol,
                            &mut label,
                            &mut insert_text,
                        )
                        .map_or((None, None), |(a, b)| (Some(a), Some(b)));

                        return CompletionItem {
                            label,
                            kind,
                            insert_text,
                            command,
                            insert_text_format,
                            tags: symbol
                                .flags
                                .contains(SymbolFlags::DEPRECATED)
                                .then(|| vec![CompletionItemTag::DEPRECATED]),
                            ..Default::default()
                        };
                    }

                    let mut insert_text = Some(format!("[\"{label}\"]"));
                    let additional_text_edits = Some(vec![TextEdit {
                        range: conversions::range(line_idx, prefix_range),
                        new_text: String::new(),
                    }]);

                    let (insert_text_format, command) =
                        modify_if_function(&finished_file, symbol, &mut label, &mut insert_text)
                            .map_or((None, None), |(a, b)| (Some(a), Some(b)));

                    CompletionItem {
                        label,
                        kind,
                        insert_text,
                        command,
                        insert_text_format,
                        additional_text_edits,
                        tags: symbol
                            .flags
                            .contains(SymbolFlags::DEPRECATED)
                            .then(|| vec![CompletionItemTag::DEPRECATED]),
                        ..Default::default()
                    }
                })
                .collect(),
            Some(ContextCompletions::FromObjectAsString { typ, replace_range }) => finished_file
                .members_of_type(
                    typ,
                    FindSymbol::BeforeIfInExecutionRange(offset, scope),
                    true,
                )
                .into_iter()
                .map(|(mut label, id)| {
                    let symbol = finished_file.get(id);
                    let kind = Some(to_completion_kind(symbol));

                    let mut insert_text = Some(format!("{label}\"]"));
                    let (insert_text_format, command) =
                        modify_if_function(&finished_file, symbol, &mut label, &mut insert_text)
                            .map_or((None, None), |(a, b)| (Some(a), Some(b)));

                    let text_edit = Some(CompletionTextEdit::Edit(TextEdit {
                        range: conversions::range(line_idx, replace_range),
                        new_text: insert_text
                            .expect("modify_if_function cannot convert Some to None"),
                    }));

                    CompletionItem {
                        label,
                        kind,
                        text_edit,
                        command,
                        insert_text_format,
                        tags: symbol
                            .flags
                            .contains(SymbolFlags::DEPRECATED)
                            .then(|| vec![CompletionItemTag::DEPRECATED]),
                        ..Default::default()
                    }
                })
                .collect(),
            Some(ContextCompletions::Root) => finished_file
                .members_of_table(
                    finished_file.root_table(),
                    FindSymbol::BeforeIfInExecutionRange(offset, scope),
                    ImportMembers::Root,
                )
                .into_iter()
                .map(|(mut label, id)| {
                    let symbol = finished_file.get(id);
                    let kind = Some(to_completion_kind(symbol));
                    let mut insert_text = None;
                    let (insert_text_format, command) =
                        modify_if_function(&finished_file, symbol, &mut label, &mut insert_text)
                            .map_or((None, None), |(a, b)| (Some(a), Some(b)));

                    CompletionItem {
                        label,
                        kind,
                        insert_text,
                        command,
                        insert_text_format,
                        tags: symbol
                            .flags
                            .contains(SymbolFlags::DEPRECATED)
                            .then(|| vec![CompletionItemTag::DEPRECATED]),
                        ..Default::default()
                    }
                })
                .collect(),
            Some(ContextCompletions::DocTags { replace_range }) => {
                let tags = [
                    "@param ",
                    "@returns",
                    "@return",
                    "@throws",
                    "@throw",
                    "@yields",
                    "@yield",
                    "@type",
                    "@varargs",
                    "@vargv",
                    "@deprecated",
                    "@hide",
                    "@native",
                    "@entity",
                    "@const",
                    "@input",
                ];
                let range = replace_range.map(|r| conversions::range(line_idx, r));
                tags.into_iter()
                    .map(|name| CompletionItem {
                        label: name.to_owned(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        text_edit: range.map(|r| {
                            CompletionTextEdit::Edit(TextEdit {
                                range: r,
                                new_text: name.to_owned(),
                            })
                        }),
                        ..Default::default()
                    })
                    .collect()
            }
            Some(ContextCompletions::DocTypes) => {
                let tags = [
                    "integer",
                    "float",
                    "string",
                    "bool",
                    "table",
                    "array",
                    "class",
                    "instance",
                    "function",
                    "null",
                    "generator",
                ];
                let symbols = finished_file.symbols_at(offset, true);

                let tags = tags.into_iter().map(std::borrow::ToOwned::to_owned).chain(
                    symbols.into_iter().filter_map(|(name, id)| {
                        if !matches!(finished_file.get(id).typ.0, Type::Class(_)) {
                            return None;
                        }
                        Some(name)
                    }),
                );

                tags.map(|label| CompletionItem {
                    label,
                    kind: Some(CompletionItemKind::KEYWORD),
                    ..Default::default()
                })
                .collect()
            }
            None => Vec::new(),
        },
    )
}

const fn to_completion_kind(symbol: &Symbol) -> CompletionItemKind {
    match symbol.typ.0 {
        Type::Enum(_) => CompletionItemKind::ENUM,
        Type::Function(_) => CompletionItemKind::FUNCTION,
        Type::Class(_) => CompletionItemKind::CLASS,
        _ => match symbol.kind {
            SymbolKind::Local(_) => CompletionItemKind::VARIABLE,
            SymbolKind::Constant => CompletionItemKind::CONSTANT,
            SymbolKind::Property(_) => CompletionItemKind::FIELD,
            SymbolKind::Enum => CompletionItemKind::ENUM,
            SymbolKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
        },
    }
}

fn modify_if_function(
    finished_file: &FinishedFile,
    symbol: &Symbol,
    label: &mut String,
    insert_text: &mut Option<String>,
) -> Option<(InsertTextFormat, Command)> {
    // we don't use finished_file.to_function_id since
    // we don't want () autocompletion on classes and such
    let Type::Function(id) = symbol.typ.0 else {
        return None;
    };

    let text = insert_text
        .as_mut()
        .map_or(label.as_str(), |text| text.as_str());

    if let Some(id) = id {
        let func = finished_file.get(id);

        if func.params.is_empty() {
            *insert_text = Some(format!("{text}()"));
            *label = format!("{label}()");
            return None;
        }
    }

    *insert_text = Some(format!("{text}($1)"));
    *label = format!("{label}(…)");
    Some((
        InsertTextFormat::SNIPPET,
        Command {
            title: "Trigger Signature Help".to_owned(),
            command: "editor.action.triggerParameterHints".to_owned(),
            arguments: None,
        },
    ))
}

pub fn can_use_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if KEYWORDS.contains_key(name) {
        return name == "constructor";
    }

    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    for char in chars {
        if !char.is_alphanumeric() && char != '_' {
            return false;
        }
    }

    true
}

enum ContextCompletions {
    Flat,
    Root,
    FromObject { typ: Type, prefix_range: TextRange },
    FromObjectAsString { typ: Type, replace_range: TextRange },
    DocTags { replace_range: Option<TextRange> },
    DocTypes,
}

fn doc_trigger(trigger_char: Option<&str>) -> bool {
    matches!(trigger_char, Some("@" | "|" | "{"))
}

fn not_doc_trigger(trigger_char: Option<&str>) -> bool {
    matches!(trigger_char, Some("." | "\""))
}

#[allow(clippy::too_many_lines)]
fn context_completions(
    syntax: &SyntaxNode,
    offset: TextSize,
    trigger_char: Option<&str>,
    finished_file: &FinishedFile,
) -> Option<ContextCompletions> {
    let Some(mut token) = syntax.token_at_offset(offset).left_biased() else {
        return Some(ContextCompletions::Flat);
    };

    dbg!(&token);

    let touching = if token.kind() == SyntaxKind::Whitespace {
        let Some(prev_token) = token.prev_token() else {
            return Some(ContextCompletions::Flat);
        };

        token = prev_token;
        false
    } else {
        true
    };

    match token.kind() {
        SyntaxKind::LineComment | SyntaxKind::BlockComment => None,
        SyntaxKind::ColonColon => {
            if doc_trigger(trigger_char) {
                return None;
            }
            let Some(parent) = token.parent() else {
                return Some(ContextCompletions::Flat);
            };

            Some(if ast::RootAccessExpression::can_cast(parent.kind()) {
                ContextCompletions::Root
            } else {
                ContextCompletions::Flat
            })
        }
        SyntaxKind::String => {
            if !touching {
                return None;
            }

            if trigger_char != Some("\"") {
                return None;
            }

            let expr = token.parent()?;
            if !ast::LiteralExpression::can_cast(expr.kind()) {
                return None;
            }

            let index = expr.parent()?;
            if !ast::Index::can_cast(index.kind()) {
                return None;
            }

            let parent = index.parent()?;
            if let Some(node) = ast::ElementAccessExpression::cast(parent) {
                let Type::String(Some(id)) = finished_file.type_at(token.text_range()) else {
                    return None;
                };

                let typ = finished_file.type_at(node.object()?.syntax().text_range());
                let unquoted_range = finished_file.get(id).unquoted_range;
                let replace_range =
                    TextRange::new(unquoted_range.start(), index.text_range().end());

                Some(ContextCompletions::FromObjectAsString { typ, replace_range })
            } else {
                None
            }
        }
        SyntaxKind::Dot => {
            if doc_trigger(trigger_char) {
                return None;
            }

            let Some(parent) = token.parent() else {
                return Some(ContextCompletions::Flat);
            };

            Some(
                if let Some(node) = ast::MemberAccessExpression::cast(parent) {
                    let typ = finished_file.type_at(node.object()?.syntax().text_range());
                    ContextCompletions::FromObject {
                        typ,
                        prefix_range: token.text_range().cover_offset(offset),
                    }
                } else {
                    ContextCompletions::Flat
                },
            )
        }

        SyntaxKind::BaseKeyword
        | SyntaxKind::BreakKeyword
        | SyntaxKind::CaseKeyword
        | SyntaxKind::CatchKeyword
        | SyntaxKind::ClassKeyword
        | SyntaxKind::CloneKeyword
        | SyntaxKind::ConstKeyword
        | SyntaxKind::ContinueKeyword
        | SyntaxKind::DefaultKeyword
        | SyntaxKind::DeleteKeyword
        | SyntaxKind::DoKeyword
        | SyntaxKind::ElseKeyword
        | SyntaxKind::EnumKeyword
        | SyntaxKind::ExtendsKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::ForEachKeyword
        | SyntaxKind::ForKeyword
        | SyntaxKind::FunctionKeyword
        | SyntaxKind::IfKeyword
        | SyntaxKind::InKeyword
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::LocalKeyword
        | SyntaxKind::NullKeyword
        | SyntaxKind::RawCallKeyword
        | SyntaxKind::ResumeKeyword
        | SyntaxKind::ReturnKeyword
        | SyntaxKind::StaticKeyword
        | SyntaxKind::SwitchKeyword
        | SyntaxKind::ThisKeyword
        | SyntaxKind::ThrowKeyword
        | SyntaxKind::TrueKeyword
        | SyntaxKind::TryKeyword
        | SyntaxKind::TypeOfKeyword
        | SyntaxKind::WhileKeyword
        | SyntaxKind::YieldKeyword
        | SyntaxKind::FileKeyword
        | SyntaxKind::LineKeyword
        | SyntaxKind::Identifier => {
            if !touching {
                return None;
            }

            if doc_trigger(trigger_char) {
                return None;
            }

            // It's either in 'Name' or 'Error' node
            let Some(parent) = token.parent().and_then(|p| p.parent()) else {
                return Some(ContextCompletions::Flat);
            };

            if ast::RootAccessExpression::can_cast(parent.kind()) {
                return Some(ContextCompletions::Root);
            }

            // Member access also wraps it in 'member' node
            let Some(member_access) = parent.parent() else {
                return Some(ContextCompletions::Flat);
            };

            Some(
                if let Some(node) = ast::MemberAccessExpression::cast(member_access) {
                    let object_range = node.object()?.syntax().text_range();
                    let typ = finished_file.type_at(object_range);
                    let prefix_range = node.dot_token().map_or_else(
                        || TextRange::new(object_range.end(), parent.text_range().start()),
                        |dot| TextRange::new(dot.text_range().start(), parent.text_range().start()),
                    );
                    ContextCompletions::FromObject { typ, prefix_range }
                } else {
                    ContextCompletions::Flat
                },
            )
        }
        SyntaxKind::DocOpenBrace | SyntaxKind::DocPipe => {
            if not_doc_trigger(trigger_char) {
                return None;
            }

            Some(ContextCompletions::DocTypes)
        }
        SyntaxKind::DocText | SyntaxKind::DocAsterisk | SyntaxKind::DocCloseBrace => {
            if not_doc_trigger(trigger_char) {
                return None;
            }

            Some(ContextCompletions::DocTags {
                replace_range: None,
            })
        }
        SyntaxKind::DocAt => {
            if not_doc_trigger(trigger_char) {
                return None;
            }

            if !touching {
                return Some(ContextCompletions::DocTags {
                    replace_range: None,
                });
            }

            Some(ContextCompletions::DocTags {
                replace_range: token
                    .parent()
                    .and_then(ast::DocTagItem::cast)
                    .map(|i| i.syntax().text_range()),
            })
        }
        SyntaxKind::DocIdentifier => {
            if not_doc_trigger(trigger_char) {
                return None;
            }

            let Some(parent) = token.parent() else {
                return Some(ContextCompletions::DocTags {
                    replace_range: None,
                });
            };

            if ast::DocTypeName::can_cast(parent.kind()) {
                if !touching {
                    return None;
                }

                return Some(ContextCompletions::DocTypes);
            }

            Some(ContextCompletions::DocTags {
                replace_range: ast::DocTagItem::cast(parent).map(|i| i.syntax().text_range()),
            })
        }
        _ => {
            if doc_trigger(trigger_char) {
                return None;
            }

            Some(ContextCompletions::Flat)
        }
    }
}
