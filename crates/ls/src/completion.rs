use anyhow::Result;
use ide::{
    Database, FindSymbol, FinishedFile, ImportMembers, Source, SymbolId, SymbolKind, Type,
    line_index, parse,
};
use lsp_types::{
    Command, CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    CompletionTextEdit, InsertTextFormat, TextEdit,
};
use sq_3_parser::{AstNode, KEYWORDS, SyntaxKind, SyntaxNode, TextRange, TextSize, ast};

use crate::conversions;

pub fn handle_completions(db: &Database, params: CompletionParams) -> Result<CompletionResponse> {
    let uri = params.text_document_position.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let Some(file) = db.get_file(&path) else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let line_idx = line_index(db, file);

    let offset = conversions::test_size(line_idx, params.text_document_position.position).unwrap();
    let syntax = parse(db, file).syntax();

    let finished_file = FinishedFile::new(db, file);

    let symbols: Vec<(String, SymbolId, Option<CompletionTextEdit>, Option<String>)> =
        match context_completions(syntax, offset, &finished_file) {
            Some(ContextCompletions::Flat) => finished_file
                .symbols_at(offset, true)
                .into_iter()
                .map(|(name, id)| (name, id, None, None))
                .collect(),
            Some(ContextCompletions::FromObject(typ, range)) => finished_file
                .members_of_type(typ, FindSymbol::BeforeIfInExecutionRange(offset), true)
                .into_iter()
                .map(|(name, id)| {
                    if !can_use_identifier(&name) {
                        let new_text = format!("[\"{name}\"]");
                        let edit = CompletionTextEdit::Edit(TextEdit {
                            range: conversions::range(line_idx, range).unwrap(),
                            new_text: new_text.clone(),
                        });

                        let prefix = get_prefix(file.text(db), range, offset);

                        (
                            name.clone(),
                            id,
                            Some(edit),
                            Some(format!("{prefix}{name}")),
                        )
                    } else {
                        (name, id, None, None)
                    }
                })
                .collect(),
            Some(ContextCompletions::FromObjectAsString(typ, range)) => finished_file
                .members_of_type(typ, FindSymbol::BeforeIfInExecutionRange(offset), true)
                .into_iter()
                .map(|(name, id)| {
                    let new_text = format!("[\"{name}\"]");
                    let edit = CompletionTextEdit::Edit(TextEdit {
                        range: conversions::range(line_idx, range).unwrap(),
                        new_text: new_text.clone(),
                    });

                    let prefix = get_prefix(file.text(db), range, offset);

                    (
                        name.clone(),
                        id,
                        Some(edit),
                        Some(format!("{prefix}{name}")),
                    )
                })
                .collect(),
            Some(ContextCompletions::Root) => finished_file
                .members_of_table(
                    finished_file.root_table(),
                    FindSymbol::BeforeIfInExecutionRange(offset),
                    ImportMembers::Root,
                )
                .into_iter()
                .map(|(name, id)| (name, id, None, None))
                .collect(),
            None => return Ok(CompletionResponse::Array(Vec::new())),
        };

    let items: Vec<CompletionItem> = symbols
        .into_iter()
        .filter_map(|(name, id, mut text_edit, filter_text)| {
            let symbol = finished_file.get(id);
            let (is_function, insert_text) =
                if let Some(CompletionTextEdit::Edit(ref mut text_edit)) = text_edit {
                    function_parantheses_text(&finished_file, id, text_edit.new_text.clone())
                        .map_or((false, None), |text| {
                            text_edit.new_text = text;
                            (true, None)
                        })
                } else {
                    function_parantheses_text(&finished_file, id, name.clone())
                        .map_or((false, None), |text| (true, Some(text)))
                };

            let command = if is_function {
                Some(Command {
                    title: "Trigger Signature Help".to_owned(),
                    command: "editor.action.triggerParameterHints".to_owned(),
                    arguments: None,
                })
            } else {
                None
            };

            let label = if is_function {
                format!("{}()", name)
            } else {
                name
            };

            Some(CompletionItem {
                label,
                kind: Some(match symbol.typ {
                    Type::Enum(_) => CompletionItemKind::ENUM,
                    Type::Function(_) => CompletionItemKind::FUNCTION,
                    Type::Class(_) => CompletionItemKind::CLASS,
                    _ => match symbol.kind {
                        SymbolKind::Local => CompletionItemKind::VARIABLE,
                        SymbolKind::Constant => CompletionItemKind::CONSTANT,
                        SymbolKind::Property(_) => CompletionItemKind::FIELD,
                        SymbolKind::Enum => CompletionItemKind::ENUM,
                        SymbolKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
                    },
                }),
                text_edit,
                filter_text,
                command,
                insert_text,
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
        })
        .collect();

    Ok(CompletionResponse::Array(items))
}

/// When you set text_edit it starts to match against the string inside the range
/// that text_edit is meant to replace, to actually get completions we also need
/// to pass filter_text with the prefix that comes between edit text range
/// and cursor offset
/// E.g.
/// a   [  "  |"]
///     ______
/// prefix ^ for this completion
pub fn get_prefix(text: &str, range: TextRange, offset: TextSize) -> &str {
    &text[range.start().into()..offset.into()]
}

pub fn function_parantheses_text(
    finished_file: &FinishedFile,
    id: SymbolId,
    text: String,
) -> Option<String> {
    let symbol = finished_file.get(id);
    // We don't use to_function_id so we don't get auto () on classes and such
    let Type::Function(id) = symbol.typ else {
        return None;
    };

    let func = finished_file.get(id);
    Some(if func.params.is_empty() {
        format!("{text}()")
    } else {
        format!("{text}($1)")
    })
}

pub fn can_use_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if KEYWORDS.contains_key(name) {
        return name == "constructor";
    }

    if !first.is_ascii_alphabetic() {
        return false;
    }

    for char in chars {
        if !char.is_alphanumeric() {
            return false;
        }
    }

    return true;
}

pub enum ContextCompletions {
    Flat,
    Root,
    FromObject(Type, TextRange),
    FromObjectAsString(Type, TextRange),
}

pub fn context_completions(
    syntax: SyntaxNode,
    offset: TextSize,
    finished_file: &FinishedFile,
) -> Option<ContextCompletions> {
    let Some(mut token) = syntax.token_at_offset(offset).left_biased() else {
        return Some(ContextCompletions::Flat);
    };

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
        SyntaxKind::ColonColon => {
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
                let typ = finished_file.type_at(node.object()?.syntax().text_range());
                Some(ContextCompletions::FromObjectAsString(
                    typ,
                    index.text_range(),
                ))
            } else {
                None
            }
        }
        SyntaxKind::OpenBracket => {
            let Some(index) = token.parent() else {
                return Some(ContextCompletions::Flat);
            };

            let Some(parent) = index.parent() else {
                return Some(ContextCompletions::Flat);
            };

            Some(
                if let Some(node) = ast::ElementAccessExpression::cast(parent) {
                    let typ = finished_file.type_at(node.object()?.syntax().text_range());
                    ContextCompletions::FromObjectAsString(typ, index.text_range())
                } else {
                    ContextCompletions::Flat
                },
            )
        }
        SyntaxKind::Dot => {
            let Some(parent) = token.parent() else {
                return Some(ContextCompletions::Flat);
            };

            Some(
                if let Some(node) = ast::MemberAccessExpression::cast(parent) {
                    let typ = finished_file.type_at(node.object()?.syntax().text_range());
                    ContextCompletions::FromObject(typ, token.text_range())
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
                return Some(ContextCompletions::Flat);
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
                    let typ = finished_file.type_at(node.object()?.syntax().text_range());
                    let range = if let Some(dot) = node.dot_token() {
                        dot.text_range().cover(parent.text_range())
                    } else {
                        parent.text_range()
                    };
                    ContextCompletions::FromObject(typ, range)
                } else {
                    ContextCompletions::Flat
                },
            )
        }
        _ => Some(ContextCompletions::Flat),
    }
}
