use line_index::{LineIndex, TextRange};
use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent,
    MarkupKind,
};
use resolver::{
    ExpressionKind, FinishedFile, FunctionIdResolution, LocalKind, Primitive, Source, SymbolKind,
    Type, VScriptDatabase, parse,
};
use sq_3_parser::{
    AstNode as _, SyntaxNode,
    ast::{self, Expr, ExpressionWrapper, LiteralExpressionKind},
};

use crate::positions;

pub fn handle_inlay_hint<Db: VScriptDatabase>(
    db: &Db,
    params: InlayHintParams,
) -> anyhow::Result<Option<Vec<InlayHint>>> {
    if !db.config().type_hints && !db.config().parameter_hints {
        return Ok(None);
    }

    let uri = params.text_document.uri;
    let file = db
        .get_file(&uri)
        .ok_or_else(|| anyhow::format_err!("File not found in workspace"))?;
    let finished_file = FinishedFile::new(db, file);

    let syntax = parse(db, file).syntax();
    let line_idx = positions::line_index(db, file);
    let range = positions::text_range(line_idx, params.range)
        .ok_or_else(|| anyhow::format_err!("Range is out of bounds"))?;

    let mut hints = Vec::new();
    if db.config().type_hints {
        hints.extend(type_hints(
            line_idx,
            &finished_file,
            &range,
            &syntax,
            db.config().enum_member_value,
        ));
    }

    if db.config().parameter_hints {
        hints.extend(parameter_hints(line_idx, &finished_file, &range, &syntax));
    }

    if hints.is_empty() {
        Ok(None)
    } else {
        Ok(Some(hints))
    }
}

fn type_hints(
    line_idx: &LineIndex,
    finished_file: &FinishedFile,
    range: &TextRange,
    syntax: &SyntaxNode,
    enum_member_value_allowed: bool,
) -> impl Iterator<Item = InlayHint> {
    finished_file.all_symbols().filter_map(move |(_, symbol)| {
        if !range.contains_range(symbol.name_range) {
            return None;
        }

        let node = symbol.node.to_node(syntax);
        match symbol.kind {
            SymbolKind::Local(
                LocalKind::Exception | LocalKind::Parameter | LocalKind::Variable,
            )
            | SymbolKind::Property {
                show_inlay_hint: true,
            } => {}
            SymbolKind::EnumMember => {
                if !enum_member_value_allowed {
                    return None;
                }

                let var = ast::Property::cast(node)?;
                if var.value().is_some() {
                    return None;
                }

                let Type::Primitive(Primitive::Integer(Some(value))) = symbol.typ else {
                    return None;
                };

                let position = positions::range(line_idx, symbol.name_range)?.end;

                return Some(InlayHint {
                    position,
                    label: InlayHintLabel::String(format!(" = {value}")),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(false),
                    padding_right: Some(false),
                    data: None,
                });
            }
            _ => return None,
        }

        // skip if type is unknown or null - nothing useful to show
        if !symbol.typ.is_useful() {
            return None;
        }

        if let Some(var) = ast::VariableDeclaration::cast(node.clone())
            && var
                .initialiser()
                .and_then(|i| i.expression())
                .is_some_and(|e| expr_obviously_has_type(&e, &symbol.typ))
        {
            return None;
        }

        if let Some(var) = ast::Property::cast(node.clone())
            && var
                .value()
                .is_some_and(|e| expr_obviously_has_type(&e, &symbol.typ))
        {
            return None;
        }

        if let Some(var) = ast::BinaryExpression::cast(node)
            && var
                .rhs()
                .is_some_and(|e| expr_obviously_has_type(&e, &symbol.typ))
        {
            return None;
        }

        let label = format!(": {}", finished_file.type_to_str(&symbol.typ));
        let tooltip = if let Ok(id) = symbol.typ.to_instance()
            && let Some(class_symbol_id) = finished_file.get(id).symbol
        {
            let content = finished_file.symbol_markdown(class_symbol_id);

            Some(InlayHintTooltip::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }))
        } else {
            None
        };

        let position = positions::range(line_idx, symbol.name_range)?.end;

        Some(InlayHint {
            position,
            label: InlayHintLabel::String(label),
            kind: Some(InlayHintKind::TYPE),
            text_edits: None,
            tooltip,
            padding_left: Some(false),
            padding_right: Some(false),
            data: None,
        })
    })
}

fn expr_obviously_has_type(expr: &Expr, typ: &Type) -> bool {
    let Ok(primitive) = Primitive::try_from(typ) else {
        return false;
    };

    match expr {
        Expr::Literal(literal) => {
            let Some((kind, _)) = literal.token() else {
                return false;
            };

            matches!(
                (kind, primitive),
                (
                    LiteralExpressionKind::DecimalInteger
                        | LiteralExpressionKind::HexInteger
                        | LiteralExpressionKind::OctalInteger
                        | LiteralExpressionKind::Character,
                    Primitive::Integer(_)
                ) | (
                    LiteralExpressionKind::String | LiteralExpressionKind::VerbatimString,
                    Primitive::String { .. }
                ) | (LiteralExpressionKind::Float, Primitive::Float(_))
                    | (
                        LiteralExpressionKind::True | LiteralExpressionKind::False,
                        Primitive::Bool(_)
                    )
                    | (LiteralExpressionKind::Null, Primitive::Null)
            )
        }
        Expr::Class(_) => {
            matches!(primitive, Primitive::Class(_))
        }
        Expr::Function(_) | Expr::Lambda(_) => {
            matches!(primitive, Primitive::Function(_))
        }
        Expr::TableLiteral(_) => {
            matches!(primitive, Primitive::Table(_))
        }
        Expr::ArrayLiteral(_) => {
            matches!(primitive, Primitive::Array(None))
        }
        _ => false,
    }
}

fn parameter_hints(
    line_idx: &LineIndex,
    finished_file: &FinishedFile,
    range: &TextRange,
    syntax: &SyntaxNode,
) -> impl Iterator<Item = InlayHint> {
    syntax
        .descendants()
        .filter_map(move |n| {
            let call = ast::CallExpression::cast(n)?;
            let callee = call.callee()?;
            if !range.contains_range(callee.syntax().text_range())
                && !call
                    .arguments()
                    .any(|a| range.contains_range(a.syntax().text_range()))
            {
                return None;
            }

            let kind = finished_file.expr_kind_at(callee.syntax().text_range());
            let typ = match kind {
                Some(ExpressionKind::Literal(typ)) => typ,
                Some(ExpressionKind::Symbol(id)) => &finished_file.get(*id).typ,
                None => return None,
            };

            let Some(FunctionIdResolution::Function(func_id)) =
                finished_file.to_function_id(typ, callee.syntax().text_range().end())
            else {
                return None;
            };

            let func = finished_file.get(func_id);

            let function_name = func.symbol.map(|s| finished_file.get(s).name.as_ref());

            Some(
                call.arguments()
                    .zip(func.params.iter().copied())
                    .filter_map(move |(arg, param_id)| {
                        if !range.contains_range(arg.syntax().text_range()) {
                            return None;
                        }

                        let param = finished_file.get(param_id);

                        if param.name.starts_with('_') {
                            return None;
                        }

                        // E.g. a single argument and either param name is 1 char long (like 'x')
                        // or function name matches the param name
                        if func.params.len() == 1
                            && (param.name.len() == 1
                                || function_name.is_some_and(|n| {
                                    param.name.to_lowercase().contains(&n.to_lowercase())
                                }))
                        {
                            return None;
                        }

                        // e.g. passing a variable that has the similar name as the param
                        if let ast::Expr::Name(n) = &arg
                            && n.identifier().is_some_and(|t| {
                                param.name.to_lowercase().contains(&t.text().to_lowercase())
                            })
                        {
                            return None;
                        }

                        let position = positions::range(line_idx, arg.syntax().text_range())?.start;

                        Some(InlayHint {
                            position,
                            label: InlayHintLabel::String(format!("{}:", param.name)),
                            kind: Some(InlayHintKind::PARAMETER),
                            text_edits: None,
                            tooltip: None,
                            padding_left: Some(false),
                            padding_right: Some(true),
                            data: None,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
}
