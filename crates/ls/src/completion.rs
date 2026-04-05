use anyhow::Result;
use ide::{
    Database, FindSymbol, FinishedFile, ImportMembers, Source, SymbolId, SymbolKind, Type,
    line_index, parse,
};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, CompletionTextEdit,
    TextEdit,
};
use sq_3_parser::{AstNode, SyntaxKind, SyntaxNode, TextRange, TextSize, ast};

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

    file.text(db);

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
        .filter_map(|(name, id, text_edit, filter_text)| {
            let symbol = finished_file.get(id);
            Some(CompletionItem {
                label: name,
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

pub fn can_use_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

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
        SyntaxKind::Identifier => {
            if !touching {
                return Some(ContextCompletions::Flat);
            }

            let Some(parent) = token.parent() else {
                return Some(ContextCompletions::Flat);
            };

            Some(if ast::RootAccessExpression::can_cast(parent.kind()) {
                ContextCompletions::Root
                // Member access also wraps it in 'member' node
            } else if let Some(node) = ast::MemberAccessExpression::cast(parent.parent()?) {
                let typ = finished_file.type_at(node.object()?.syntax().text_range());
                ContextCompletions::FromObject(typ, parent.text_range())
            } else {
                ContextCompletions::Flat
            })
        }
        _ => Some(ContextCompletions::Flat),
    }
}
