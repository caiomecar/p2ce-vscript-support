use ::line_index::LineIndex;
use lsp_types::{
    Command, CompletionItem, CompletionItemKind, CompletionItemTag, CompletionParams,
    CompletionResponse, CompletionTextEdit, InsertTextFormat, TextEdit,
};
use resolver::{
    Database, ExpressionKind, FindSymbol, FinishedFile, FunctionId, ImportMembers, Primitive,
    ScopeId, Source, StringKind, Symbol, SymbolFlags, SymbolKind, Type, TypeFlags, TypeState,
    line_index, parse,
};
use sq_3_parser::{
    AstNode, KEYWORDS, SyntaxKind, SyntaxNode, TextRange, TextSize,
    ast::{self, HasName, IsClassMember},
};
use std::fmt::Write as _;

use crate::conversions;

macro_rules! keyword_completion {
    ($kw:literal) => {
        CompletionItem {
            label: $kw.to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        }
    };
    ($kw:literal, Space) => {
        CompletionItem {
            label: $kw.to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            insert_text: Some(concat!($kw, " ").to_owned()),
            insert_text_format: None,
            ..Default::default()
        }
    };
    ($kw:literal, Parentheses) => {
        CompletionItem {
            label: $kw.to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            insert_text: Some(concat!($kw, "($1)").to_owned()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }
    };
    ($kw:literal, ParenthesesSpace) => {
        CompletionItem {
            label: $kw.to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            insert_text: Some(concat!($kw, " ($1)").to_owned()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }
    };
}

fn statement_keywords() -> Vec<CompletionItem> {
    vec![
        keyword_completion!("base"),
        keyword_completion!("break"),
        keyword_completion!("class", Space),
        keyword_completion!("clone", Space),
        keyword_completion!("const", Space),
        keyword_completion!("continue"),
        keyword_completion!("delete", Space),
        keyword_completion!("do"),
        keyword_completion!("enum", Space),
        keyword_completion!("foreach", ParenthesesSpace),
        keyword_completion!("for", ParenthesesSpace),
        keyword_completion!("function", Space),
        keyword_completion!("if", ParenthesesSpace),
        keyword_completion!("local", Space),
        keyword_completion!("rawcall", Parentheses),
        keyword_completion!("resume", Space),
        keyword_completion!("return"),
        keyword_completion!("switch", ParenthesesSpace),
        keyword_completion!("this"),
        keyword_completion!("throw", Space),
        keyword_completion!("try"),
        keyword_completion!("while", ParenthesesSpace),
        keyword_completion!("yield", Space),
    ]
}

fn expression_keywords() -> Vec<CompletionItem> {
    vec![
        keyword_completion!("base"),
        keyword_completion!("class", Space),
        keyword_completion!("clone", Space),
        keyword_completion!("delete", Space),
        keyword_completion!("resume", Space),
        keyword_completion!("this"),
        keyword_completion!("function", Parentheses),
        keyword_completion!("null"),
        keyword_completion!("false"),
        keyword_completion!("true"),
        keyword_completion!("typeof", Space),
        keyword_completion!("__FILE__"),
        keyword_completion!("__LINE__"),
    ]
}

pub fn handle_completions(db: &Database, params: CompletionParams) -> CompletionResponse {
    let uri = params.text_document_position.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return CompletionResponse::Array(Vec::new());
    };

    let Some(file) = db.get_file(&path) else {
        return CompletionResponse::Array(Vec::new());
    };

    let line_idx = line_index(db, file);

    let Some(offset) = conversions::test_size(line_idx, params.text_document_position.position)
    else {
        return CompletionResponse::Array(Vec::new());
    };

    let syntax = parse(db, file).syntax();

    let finished_file = FinishedFile::new(db, file);

    let scope = finished_file.scope(offset);
    let trigger_char = params.context.and_then(|c| c.trigger_character);

    CompletionResponse::Array(
        match context_completions(&syntax, offset, trigger_char.as_deref(), &finished_file) {
            // Some(ContextCompletions::Flat) => completions_flat(offset, &finished_file),
            Some(ContextCompletions::Statement) => {
                let mut completions = completions_flat(offset, &finished_file);
                completions.extend(statement_keywords());
                completions
            }
            Some(ContextCompletions::Expression) => {
                let mut completions = completions_flat(offset, &finished_file);
                completions.extend(expression_keywords());
                completions
            }
            Some(ContextCompletions::FromObject { typ, prefix_range }) => {
                completions_from_object(line_idx, offset, &finished_file, scope, typ, prefix_range)
            }
            Some(ContextCompletions::FromQualifiedName { typ }) => {
                completions_from_qualified_name(offset, &finished_file, scope, typ)
            }
            Some(ContextCompletions::FromObjectAsString { typ, replace_range }) => {
                completions_from_object_as_string(
                    line_idx,
                    offset,
                    &finished_file,
                    scope,
                    typ,
                    replace_range,
                )
            }
            Some(ContextCompletions::InsideString {
                kind,
                replace_range,
            }) => completions_inside_string(line_idx, &finished_file, kind, replace_range),
            Some(ContextCompletions::Root) => completions_root(offset, &finished_file, scope),
            Some(ContextCompletions::AfterLocal | ContextCompletions::Table) => {
                vec![keyword_completion!("function", Space)]
            }
            Some(ContextCompletions::AfterForeachValue) => {
                vec![keyword_completion!("in", Space)]
            }
            Some(ContextCompletions::ForInitialiser) => {
                let mut completions = completions_flat(offset, &finished_file);
                completions.extend(expression_keywords());
                completions.push(keyword_completion!("local", Space));
                completions
            }
            Some(ContextCompletions::Class) => {
                vec![
                    keyword_completion!("function", Space),
                    keyword_completion!("constructor", Parentheses),
                    keyword_completion!("static", Space),
                ]
            }
            Some(ContextCompletions::ClassAfterStatic) => {
                vec![
                    keyword_completion!("function", Space),
                    keyword_completion!("constructor", Parentheses),
                ]
            }
            Some(ContextCompletions::Switch) => {
                let mut completions = completions_flat(offset, &finished_file);
                completions.extend(statement_keywords());
                completions.push(keyword_completion!("case", Space));
                completions.push(keyword_completion!("default"));
                completions
            }
            Some(ContextCompletions::DocTag { replace_range }) => {
                completion_doc_tag(line_idx, replace_range)
            }
            Some(ContextCompletions::DocType) => completions_doc_type(offset, &finished_file),
            Some(ContextCompletions::DocParamNames { id }) => {
                completions_doc_param_names(id, &finished_file)
            }
            Some(ContextCompletions::DocAutoGenerated { typ, replace_range }) => {
                completion_doc_auto_generated(line_idx, &finished_file, &typ, replace_range)
            }
            None => Vec::new(),
        },
    )
}

#[derive(Debug)]
enum ContextCompletions {
    Statement,
    Expression,
    Root,
    FromQualifiedName {
        typ: Option<Type>,
    },
    FromObject {
        typ: Type,
        prefix_range: TextRange,
    },
    FromObjectAsString {
        typ: Type,
        replace_range: TextRange,
    },
    InsideString {
        kind: StringKind,
        replace_range: TextRange,
    },
    AfterLocal,
    AfterForeachValue,
    ForInitialiser,
    Table,
    Class,
    ClassAfterStatic,
    Switch,
    DocTag {
        replace_range: Option<TextRange>,
    },
    DocType,
    DocParamNames {
        id: FunctionId,
    },
    DocAutoGenerated {
        typ: Type,
        replace_range: TextRange,
    },
}

fn param_tag_name(
    finished_file: &FinishedFile<'_>,
    parent: &SyntaxNode,
) -> Option<ContextCompletions> {
    let Some(parent) = parent.parent() else {
        return Some(ContextCompletions::DocTag {
            replace_range: None,
        });
    };

    if !ast::ParamTag::can_cast(parent.kind()) {
        return Some(ContextCompletions::DocTag {
            replace_range: None,
        });
    }

    let comment = parent.parent()?;
    if !ast::DocComment::can_cast(comment.kind()) {
        return None;
    }
    let range = comment.text_range();
    let symbol = finished_file.doc_to_symbol().get(&range)?;
    let Ok(id) = finished_file.get(*symbol).typ.to_function() else {
        return None;
    };

    Some(ContextCompletions::DocParamNames { id })
}

#[allow(clippy::too_many_lines)]
fn context_completions(
    syntax: &SyntaxNode,
    offset: TextSize,
    trigger_char: Option<&str>,
    finished_file: &FinishedFile,
) -> Option<ContextCompletions> {
    let Some(mut token) = syntax.token_at_offset(offset).left_biased() else {
        return Some(ContextCompletions::Statement);
    };

    // Inherently trigger char is None when this is true since we cannot have space between the cursor
    // and the character the cursor just wrote
    let touching = token.kind() != SyntaxKind::Whitespace;
    if !touching {
        let Some(prev_token) = token.prev_token() else {
            return Some(ContextCompletions::Statement);
        };

        token = prev_token;
    }

    match token.kind() {
        SyntaxKind::LineComment | SyntaxKind::BlockComment if touching => None,
        SyntaxKind::ColonColon => {
            let parent = token.parent()?;

            // e.g.
            // ::|
            // ::   |
            if ast::RootAccessExpression::can_cast(parent.kind()) {
                return Some(ContextCompletions::Root);
            }

            // e.g.
            // function abc::|
            // function abc::   |
            let part = ast::QualifiedNamePart::cast(parent)?;
            let name = part.name()?;
            let symbol = finished_file
                .range_to_symbol()
                .get(&name.syntax().text_range())?;

            Some(ContextCompletions::FromQualifiedName {
                typ: Some(finished_file.get(*symbol).typ.clone()),
            })
        }
        SyntaxKind::String => {
            if !touching {
                return Some(ContextCompletions::Expression);
            }

            // if !matches!(trigger_char, None | Some("\"")) {
            //     return None;
            // }

            let Some(ExpressionKind::Literal(Type::Primitive(Primitive::String {
                kind,
                literal: Some(literal),
            }))) = finished_file.expr_kind_at(token.text_range()).cloned()
            else {
                return None;
            };

            let expr = token.parent()?;
            if !ast::LiteralExpression::can_cast(expr.kind()) {
                let replace_range = TextRange::new(
                    finished_file.get(literal).unquoted_range.start(),
                    token.text_range().end(),
                );
                return Some(ContextCompletions::InsideString {
                    kind,
                    replace_range,
                });
            }

            let index = expr.parent()?;
            if !ast::Index::can_cast(index.kind()) {
                let replace_range = TextRange::new(
                    finished_file.get(literal).unquoted_range.start(),
                    token.text_range().end(),
                );
                return Some(ContextCompletions::InsideString {
                    kind,
                    replace_range,
                });
            }

            let parent = index.parent()?;
            if let Some(node) = ast::ElementAccessExpression::cast(parent) {
                let typ = finished_file.type_at(node.object()?.syntax().text_range());
                let unquoted_range = finished_file.get(literal).unquoted_range;
                let replace_range =
                    TextRange::new(unquoted_range.start(), index.text_range().end());

                Some(ContextCompletions::FromObjectAsString { typ, replace_range })
            } else {
                let replace_range = TextRange::new(
                    finished_file.get(literal).unquoted_range.start(),
                    token.text_range().end(),
                );
                Some(ContextCompletions::InsideString {
                    kind,
                    replace_range,
                })
            }
        }
        SyntaxKind::Dot => {
            let parent = token.parent()?;

            let node = ast::MemberAccessExpression::cast(parent)?;

            let typ = finished_file.type_at(node.object()?.syntax().text_range());
            Some(ContextCompletions::FromObject {
                typ,
                prefix_range: token.text_range().cover_offset(offset),
            })
        }
        SyntaxKind::FunctionKeyword if !touching => {
            let parent = token.parent()?;
            // function |
            if ast::FunctionStatement::can_cast(parent.kind()) {
                return Some(ContextCompletions::FromQualifiedName { typ: None });
            }

            // The rest are
            // local function |
            // (method) function |
            // (expression) function |
            // These should not have any completions since we're writing out the name
            None
        }
        SyntaxKind::LocalKeyword if !touching => Some(ContextCompletions::AfterLocal),
        SyntaxKind::StaticKeyword if !touching => Some(ContextCompletions::ClassAfterStatic),
        SyntaxKind::EnumKeyword | SyntaxKind::ConstKeyword if !touching => None,

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
            let mut parent = token.parent()?;
            if matches!(parent.kind(), SyntaxKind::Name | SyntaxKind::Error) {
                parent = parent.parent()?;
            }

            if !touching {
                if ast::ForEachValue::can_cast(parent.kind()) {
                    return Some(ContextCompletions::AfterForeachValue);
                }
                return Some(ContextCompletions::Expression);
            }

            // local a = {
            //    fun|
            // }
            if ast::SimpleName::can_cast(parent.kind()) {
                let property = ast::Property::cast(parent.parent()?)?;
                return match property.syntax().parent()?.kind() {
                    SyntaxKind::TableLiteralExpression => Some(ContextCompletions::Table),
                    SyntaxKind::ClassStatement | SyntaxKind::ClassExpression => {
                        if property.static_keyword().is_none() {
                            Some(ContextCompletions::Class)
                        } else {
                            Some(ContextCompletions::ClassAfterStatic)
                        }
                    }
                    _ => None,
                };
            }

            // const b| = 1;
            // local b = {
            //    function ab|
            // }
            if matches!(
                parent.kind(),
                SyntaxKind::LocalFunctionDeclaration
                    | SyntaxKind::Method
                    | SyntaxKind::Constructor
                    | SyntaxKind::ForeachKey
                    | SyntaxKind::ForeachValue
                    | SyntaxKind::ConstStatement
                    | SyntaxKind::EnumStatement
            ) {
                return None;
            }

            // local a| = 1
            // function abc(a|)
            if let Some(var) = ast::VariableDeclaration::cast(parent.clone()) {
                let local_decl = ast::LocalVariableDeclaration::cast(parent.parent()?)?;
                let first = local_decl.declarations().next()?;
                if first == var {
                    return Some(ContextCompletions::AfterLocal);
                }
                return None;
            }

            // ::  a|
            if ast::RootAccessExpression::can_cast(parent.kind()) {
                return Some(ContextCompletions::Root);
            }

            // function abc::wow::new|
            // function new|
            if let Some(qualified_name) = ast::QualifiedName::cast(parent.clone()) {
                if let Some(last) = qualified_name.parts().last() {
                    let name = last.name()?;
                    let symbol = finished_file
                        .range_to_symbol()
                        .get(&name.syntax().text_range())?;

                    return Some(ContextCompletions::FromQualifiedName {
                        typ: Some(finished_file.get(*symbol).typ.clone()),
                    });
                }

                return Some(ContextCompletions::FromQualifiedName { typ: None });
            }

            // function abc::wow|::new
            if let Some(part) = ast::QualifiedNamePart::cast(parent.clone()) {
                let qualified_name = ast::QualifiedName::cast(part.syntax().parent()?)?;
                let mut last = None;
                for part in qualified_name.parts() {
                    if part.syntax().text_range().end() < offset {
                        last = Some(part);
                    } else {
                        break;
                    }
                }

                let name = last?.name()?;
                let symbol = finished_file
                    .range_to_symbol()
                    .get(&name.syntax().text_range())?;

                return Some(ContextCompletions::FromQualifiedName {
                    typ: Some(finished_file.get(*symbol).typ.clone()),
                });
            }

            if ast::ForInitialiser::can_cast(parent.kind()) {
                return Some(ContextCompletions::ForInitialiser);
            }

            if ast::Stmt::can_cast(parent.kind()) {
                return Some(match parent.parent().map(|p| p.kind()) {
                    Some(SyntaxKind::SwitchStatement | SyntaxKind::CaseClause) => {
                        ContextCompletions::Switch
                    }
                    // Some(SyntaxKind::ForInitialiser) => ContextCompletions::ForInitialiser,
                    _ => ContextCompletions::Statement,
                });
            }

            if !ast::MemberPart::can_cast(parent.kind()) {
                return Some(ContextCompletions::Statement);
            }

            let Some(member_access) = parent.parent() else {
                return Some(ContextCompletions::Statement);
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
                    ContextCompletions::Expression
                },
            )
        }
        SyntaxKind::CloseBrace | SyntaxKind::CloseParenthesis => {
            if trigger_char.is_some() {
                return None;
            }

            let parent = token.parent()?;

            // if (...) |
            if ast::Stmt::can_cast(parent.kind()) {
                Some(ContextCompletions::Statement)
            } else {
                // a = [wow() |]
                Some(ContextCompletions::Expression)
            }
        }
        SyntaxKind::Comma => match token.parent()?.kind() {
            SyntaxKind::TableLiteralExpression => Some(ContextCompletions::Table),
            SyntaxKind::ClassStatement | SyntaxKind::ClassExpression => {
                Some(ContextCompletions::Class)
            }
            SyntaxKind::ForEachStatement => Some(ContextCompletions::AfterForeachValue),
            SyntaxKind::ParameterList
            | SyntaxKind::LocalVariableDeclaration
            | SyntaxKind::EnumStatement => None,
            _ => Some(ContextCompletions::Expression),
        },

        SyntaxKind::OpenParenthesis => {
            match token.parent()?.kind() {
                // for (|)
                SyntaxKind::ForStatement => Some(ContextCompletions::ForInitialiser),
                // function a(|)
                SyntaxKind::ParameterList => None,
                _ => Some(ContextCompletions::Expression),
            }
        }

        SyntaxKind::LineFeed | SyntaxKind::Semicolon => {
            match token.parent()?.kind() {
                SyntaxKind::TableLiteralExpression => Some(ContextCompletions::Table),
                SyntaxKind::ClassStatement | SyntaxKind::ClassExpression => {
                    Some(ContextCompletions::Class)
                }
                //no default clause because no clauses below it make sense?
                SyntaxKind::SwitchStatement | SyntaxKind::CaseClause // | SyntaxKind::DefaultClause
                    => Some(ContextCompletions::Switch),
                SyntaxKind::ForEachStatement => Some(ContextCompletions::AfterForeachValue),
                SyntaxKind::EnumStatement => None,
                SyntaxKind::ForStatement => Some(ContextCompletions::Expression),
                _ => Some(ContextCompletions::Statement),
            }
        }

        SyntaxKind::DocOpenBrace | SyntaxKind::DocPipe => Some(ContextCompletions::DocType),
        SyntaxKind::DocCloseBrace => {
            let Some(parent) = token.parent() else {
                return Some(ContextCompletions::DocTag {
                    replace_range: None,
                });
            };

            if !ast::DocType::can_cast(parent.kind()) {
                return Some(ContextCompletions::DocTag {
                    replace_range: None,
                });
            }

            // e.g.
            // @param {type} |
            param_tag_name(finished_file, &parent)
        }
        SyntaxKind::DocText | SyntaxKind::DocAsterisk => {
            if trigger_char.is_some() {
                return None;
            }

            Some(ContextCompletions::DocTag {
                replace_range: None,
            })
        }
        SyntaxKind::DocAt => {
            // e.g
            // @| -> show
            // @  | -> don't show
            if !touching {
                return Some(ContextCompletions::DocTag {
                    replace_range: None,
                });
            }

            Some(ContextCompletions::DocTag {
                replace_range: token
                    .parent()
                    .and_then(ast::DocTagItem::cast)
                    .map(|i| i.syntax().text_range()),
            })
        }
        SyntaxKind::DocIdentifier => {
            let Some(parent) = token.parent() else {
                return Some(ContextCompletions::DocTag {
                    replace_range: None,
                });
            };

            // @param name| -> show names
            // @param name  | -> show tags
            if ast::DocName::can_cast(parent.kind()) {
                if !touching {
                    return Some(ContextCompletions::DocTag {
                        replace_range: None,
                    });
                }

                return param_tag_name(finished_file, &parent);
            }

            if !touching
                && ast::DocTagItem::can_cast(parent.kind())
                && let Some(completions) = param_tag_name(finished_file, &parent)
            {
                return Some(completions);
            }

            if ast::DocTypeName::can_cast(parent.kind()) {
                return Some(ContextCompletions::DocType);
            }

            Some(ContextCompletions::DocTag {
                replace_range: ast::DocTagItem::cast(parent).map(|i| i.syntax().text_range()),
            })
        }
        SyntaxKind::DocAsteriskSlash if !touching => Some(ContextCompletions::Statement),
        SyntaxKind::DocAsteriskSlash | SyntaxKind::DocSlashAsteriskAsterisk => {
            let tag =
                // We don't want to show tags after writing * in /**| */
                // since this will prevent the user from pressing enter to expand
                // the doc to the next line
                trigger_char
                    .is_none()
                    .then_some(ContextCompletions::DocTag {
                        replace_range: None,
                    });

            let Some(parent) = token.parent() else {
                return tag;
            };

            if !ast::DocComment::can_cast(parent.kind()) {
                return tag;
            }

            let range = parent.text_range();
            let Some(symbol) = finished_file.doc_to_symbol().get(&range) else {
                return tag;
            };

            Some(ContextCompletions::DocAutoGenerated {
                typ: finished_file.get(*symbol).typ.clone(),
                replace_range: range,
            })
        }

        _ => {
            if trigger_char.is_some() {
                return None;
            }

            Some(ContextCompletions::Expression)
        }
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
    let Type::Primitive(Primitive::Function(id)) = symbol.typ else {
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

fn can_use_identifier(name: &str) -> bool {
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

fn symbol_tags(symbol: &Symbol) -> Option<Vec<CompletionItemTag>> {
    symbol
        .flags
        .contains(SymbolFlags::DEPRECATED)
        .then(|| vec![CompletionItemTag::DEPRECATED])
}

const fn to_completion_kind(symbol: &Symbol) -> CompletionItemKind {
    match symbol.typ {
        Type::Enum(_) => CompletionItemKind::ENUM,
        Type::Primitive(Primitive::Function(_)) => CompletionItemKind::FUNCTION,
        Type::Primitive(Primitive::Class(_)) => CompletionItemKind::CLASS,
        _ => match symbol.kind {
            SymbolKind::Local(_) => CompletionItemKind::VARIABLE,
            SymbolKind::Constant => CompletionItemKind::CONSTANT,
            SymbolKind::Property(_) => CompletionItemKind::FIELD,
            SymbolKind::Enum => CompletionItemKind::ENUM,
            SymbolKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
        },
    }
}

fn completions_flat(offset: TextSize, finished_file: &FinishedFile<'_>) -> Vec<CompletionItem> {
    finished_file
        .symbols_at(offset, true)
        .into_iter()
        .map(|(name, id)| {
            let mut label = name.into_string();
            let symbol = finished_file.get(id);
            let kind = Some(to_completion_kind(symbol));
            let mut insert_text = None;
            let (insert_text_format, command) =
                modify_if_function(finished_file, symbol, &mut label, &mut insert_text)
                    .map_or((None, None), |(a, b)| (Some(a), Some(b)));

            CompletionItem {
                label,
                kind,
                insert_text,
                command,
                insert_text_format,
                tags: symbol_tags(symbol),
                ..Default::default()
            }
        })
        .collect()
}

fn completions_from_object(
    line_idx: &LineIndex,
    offset: TextSize,
    finished_file: &FinishedFile<'_>,
    scope: ScopeId,
    typ: Type,
    prefix_range: TextRange,
) -> Vec<CompletionItem> {
    finished_file
        .members_of_type(
            typ,
            FindSymbol::BeforeIfInExecutionRange(offset, scope),
            true,
        )
        .into_iter()
        .filter_map(|(name, id)| {
            let mut label = name.into_string();
            let symbol = finished_file.get(id);
            let kind = Some(to_completion_kind(symbol));
            if can_use_identifier(&label) {
                let mut insert_text = None;
                let (insert_text_format, command) =
                    modify_if_function(finished_file, symbol, &mut label, &mut insert_text)
                        .map_or((None, None), |(a, b)| (Some(a), Some(b)));

                return Some(CompletionItem {
                    label,
                    kind,
                    insert_text,
                    command,
                    insert_text_format,
                    tags: symbol_tags(symbol),
                    ..Default::default()
                });
            }

            let mut insert_text = Some(format!("[\"{label}\"]"));
            let additional_text_edits = Some(vec![TextEdit {
                range: conversions::range(line_idx, prefix_range)?,
                new_text: String::new(),
            }]);

            let (insert_text_format, command) =
                modify_if_function(finished_file, symbol, &mut label, &mut insert_text)
                    .map_or((None, None), |(a, b)| (Some(a), Some(b)));

            Some(CompletionItem {
                label,
                kind,
                insert_text,
                command,
                insert_text_format,
                additional_text_edits,
                tags: symbol_tags(symbol),
                ..Default::default()
            })
        })
        .collect()
}

fn completions_from_qualified_name(
    offset: TextSize,
    finished_file: &FinishedFile<'_>,
    scope: ScopeId,
    typ: Option<Type>,
) -> Vec<CompletionItem> {
    let all_members = typ.map_or_else(
        || finished_file.symbols_at(offset, false),
        |typ| {
            finished_file.members_of_type(
                typ,
                FindSymbol::BeforeIfInExecutionRange(offset, scope),
                false,
            )
        },
    );

    all_members
        .into_iter()
        .filter_map(|(name, id)| {
            let symbol = finished_file.get(id);
            if matches!(
                symbol.kind,
                SymbolKind::Enum | SymbolKind::Local(_) | SymbolKind::Constant
            ) {
                return None;
            }

            if !symbol.typ.type_flags().intersects(TypeFlags::HAS_MEMBERS) {
                return None;
            }

            Some(CompletionItem {
                label: format!("{name}::"),
                kind: Some(to_completion_kind(symbol)),
                ..Default::default()
            })
        })
        .collect()
}

fn completions_from_object_as_string(
    line_idx: &LineIndex,
    offset: TextSize,
    finished_file: &FinishedFile<'_>,
    scope: ScopeId,
    typ: Type,
    replace_range: TextRange,
) -> Vec<CompletionItem> {
    finished_file
        .members_of_type(
            typ,
            FindSymbol::BeforeIfInExecutionRange(offset, scope),
            true,
        )
        .into_iter()
        .filter_map(|(name, id)| {
            let mut label = name.into_string();
            let symbol = finished_file.get(id);
            let kind = Some(to_completion_kind(symbol));

            let mut insert_text = Some(format!("{label}\"]"));
            let (insert_text_format, command) =
                modify_if_function(finished_file, symbol, &mut label, &mut insert_text)
                    .map_or((None, None), |(a, b)| (Some(a), Some(b)));

            let text_edit = Some(CompletionTextEdit::Edit(TextEdit {
                range: conversions::range(line_idx, replace_range)?,
                new_text: insert_text.expect("modify_if_function cannot convert Some to None"),
            }));

            Some(CompletionItem {
                label,
                kind,
                text_edit,
                command,
                insert_text_format,
                tags: symbol_tags(symbol),
                ..Default::default()
            })
        })
        .collect()
}

fn completions_inside_string(
    line_idx: &LineIndex,
    finished_file: &FinishedFile,
    kind: StringKind,
    replace_range: TextRange,
) -> Vec<CompletionItem> {
    let Some(range) = conversions::range(line_idx, replace_range) else {
        return Vec::new();
    };

    match kind {
        StringKind::Script => finished_file
            .db()
            .script_literals()
            .iter()
            .map(|value| CompletionItem {
                label: value.clone(),
                kind: Some(CompletionItemKind::VALUE),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: format!("{value}\""),
                })),
                ..Default::default()
            })
            .collect(),

        _ => kind.values().map_or(vec![], |sets| {
            sets.iter()
                .flat_map(|set| set.1.iter())
                .map(|value| CompletionItem {
                    label: (*value).to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: format!("{value}\""),
                    })),
                    ..Default::default()
                })
                .collect()
        }),
    }
}

fn completions_root(
    offset: TextSize,
    finished_file: &FinishedFile<'_>,
    scope: ScopeId,
) -> Vec<CompletionItem> {
    finished_file
        .members_of_table(
            finished_file.root_table(),
            FindSymbol::BeforeIfInExecutionRange(offset, scope),
            ImportMembers::Root,
        )
        .into_iter()
        .map(|(name, id)| {
            let mut label = name.into_string();
            let symbol = finished_file.get(id);
            let kind = Some(to_completion_kind(symbol));
            let mut insert_text = None;
            let (insert_text_format, command) =
                modify_if_function(finished_file, symbol, &mut label, &mut insert_text)
                    .map_or((None, None), |(a, b)| (Some(a), Some(b)));

            CompletionItem {
                label,
                kind,
                insert_text,
                command,
                insert_text_format,
                tags: symbol_tags(symbol),
                ..Default::default()
            }
        })
        .collect()
}

fn completion_doc_tag(
    line_idx: &LineIndex,
    replace_range: Option<TextRange>,
) -> Vec<CompletionItem> {
    let tags = [
        "@param ",
        "@returns ",
        "@throws ",
        "@yields ",
        "@type ",
        "@varargs ",
        "@deprecated",
        "@hide",
        "@native",
        "@entity",
        "@const",
        "@input",
        "@extends ",
    ];
    let range = replace_range.and_then(|r| conversions::range(line_idx, r));
    tags.into_iter()
        .map(|name| CompletionItem {
            label: name.to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            text_edit: range.map(|range| {
                CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: name.to_owned(),
                })
            }),
            ..Default::default()
        })
        .collect()
}

fn completions_doc_type(offset: TextSize, finished_file: &FinishedFile<'_>) -> Vec<CompletionItem> {
    let tags = [
        "any",
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
        "thread",
        "weakref",
        // string literals
        "script",
        "attribute",
        "input",
        "output",
        "classname",
        "convar",
        "integer_property",
        "integer_array_property",
        "float_property",
        "float_array_property",
        "entity_property",
        "entity_array_property",
        "bool_property",
        "bool_array_property",
        "string_property",
        "string_array_property",
        "vector_property",
        "vector_array_property",
        "property",
        "property_array",
    ];

    let symbols = finished_file.symbols_at(offset, true);

    tags.into_iter()
        .map(|name| CompletionItem {
            label: name.to_owned(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        })
        .chain(symbols.into_iter().filter_map(|(name, id)| {
            if !matches!(
                finished_file.get(id).typ,
                Type::Primitive(Primitive::Class(_))
            ) {
                return None;
            }
            Some(CompletionItem {
                label: name.into_string(),
                kind: Some(CompletionItemKind::CLASS),
                ..Default::default()
            })
        }))
        .collect()
}

fn completions_doc_param_names(
    id: FunctionId,
    finished_file: &FinishedFile,
) -> Vec<CompletionItem> {
    finished_file
        .get(id)
        .params
        .iter()
        .map(|id| {
            let label = finished_file.get(*id).name.to_string();
            CompletionItem {
                label,
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            }
        })
        .collect()
}

fn completion_doc_auto_generated(
    line_idx: &LineIndex,
    finished_file: &FinishedFile<'_>,
    typ: &Type,
    replace_range: TextRange,
) -> Vec<CompletionItem> {
    #[allow(clippy::literal_string_with_formatting_args)]
    let text = match typ {
        Type::Primitive(Primitive::Function(Some(id))) => {
            let mut text = "/**\n * ${1:Description}".to_owned();
            let mut last_index = 1;
            let func = finished_file.get(*id);
            for param in &func.params {
                last_index += 1;

                let symbol = finished_file.get(*param);

                let typ = match symbol.typ {
                    Type::UNKNOWN | Type::NULL => &Type::Any,
                    _ => &symbol.typ,
                };

                let name = &symbol.name;

                let _ = write!(
                    text,
                    "\n * @param {{${{{}:{}}}}} {}",
                    last_index,
                    finished_file.type_to_str(typ),
                    name
                );
            }

            if func.ret != Type::NULL {
                last_index += 1;

                let typ = match func.ret {
                    Type::UNKNOWN => &Type::Any,
                    _ => &func.ret,
                };

                let _ = write!(
                    text,
                    "\n * @returns {{${{{}:{}}}}}",
                    last_index,
                    finished_file.type_to_str(typ)
                );
            }

            if func.throws_state != TypeState::NotAssigned {
                last_index += 1;

                let typ = match func.throws {
                    Type::UNKNOWN => &Type::Any,
                    _ => &func.throws,
                };

                let _ = write!(
                    text,
                    "\n * @throws {{${{{}:{}}}}}",
                    last_index,
                    finished_file.type_to_str(typ)
                );
            }

            if func.yields_state != TypeState::NotAssigned {
                let typ = match func.yields {
                    Type::UNKNOWN => &Type::Any,
                    _ => &func.yields,
                };

                let _ = write!(
                    text,
                    "\n * @yields {{${{{}:{}}}}}",
                    last_index + 1,
                    finished_file.type_to_str(typ)
                );
            }

            let _ = write!(text, "\n */");
            text
        }
        _ => {
            format!(
                "/**\n * ${{1:Description}}\n * @type {{${{2:{}}}}}\n */",
                finished_file.type_to_str(typ),
            )
        }
    };

    let additional_text_edits = conversions::range(line_idx, replace_range).map(|range| {
        vec![TextEdit {
            range,
            new_text: String::new(),
        }]
    });

    vec![CompletionItem {
        label: "Autogenerated Doc Comment ...".to_owned(),
        kind: Some(CompletionItemKind::KEYWORD),
        insert_text: Some(text),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        additional_text_edits,
        ..Default::default()
    }]
}
