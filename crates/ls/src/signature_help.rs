use lsp_types::{
    Documentation, MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, SignatureHelp,
    SignatureHelpParams, SignatureInformation,
};
use resolver::{
    Database, ExpressionKind, FinishedFile, FunctionIdResolution, Source, line_index, parse,
};
use sq_3_parser::{AstNode, ast};

use crate::conversions;

pub fn handle_signature_help(db: &Database, params: SignatureHelpParams) -> Option<SignatureHelp> {
    let uri = params.text_document_position_params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position_params.position);

    let syntax = parse(db, file).syntax();
    let node = syntax
        .token_at_offset(offset)
        .right_biased()
        .and_then(|t| t.parent())
        .unwrap_or(syntax);

    let call = node.ancestors().find_map(ast::CallExpression::cast)?;

    let callee = call.callee()?;

    let finished_file = FinishedFile::new(db, file);
    let kind = finished_file.expr_kind_at(callee.syntax().text_range());
    let (name, typ) = match kind {
        Some(ExpressionKind::Literal(typ)) => (String::new(), typ),
        Some(ExpressionKind::Symbol(id)) => {
            let symbol = finished_file.get(*id);
            (symbol.name.to_string(), &symbol.typ)
        }
        None => return None,
    };

    let id = match finished_file.to_function_id(typ, offset) {
        Some(FunctionIdResolution::Function(id)) => id,
        Some(FunctionIdResolution::DefaultConstructor) => {
            return Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: format!("{name}()"),
                    parameters: None,
                    documentation: None,
                    active_parameter: None,
                }],
                active_signature: Some(0),
                active_parameter: None,
            });
        }
        None => return None,
    };

    let mut active_param = 0;

    for (i, arg) in call.arguments().enumerate() {
        if arg.syntax().text_range().contains_inclusive(offset) {
            active_param = i;
            break;
        }

        // If cursor is after this arg, keep going
        if arg.syntax().text_range().end() < offset {
            active_param = i + 1;
        }
    }

    let (label, param_ranges) = finished_file.function_markdown(&name, id);
    let func = finished_file.get(id);

    let param_infos = func
        .params
        .iter()
        .zip(&param_ranges)
        .map(|(param_id, range)| {
            let param = finished_file.get(*param_id);
            ParameterInformation {
                label: ParameterLabel::LabelOffsets(*range),
                documentation: param.description.clone().map(|d| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: d,
                    })
                }),
            }
        })
        .collect::<Vec<_>>();

    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label,
            parameters: Some(param_infos),
            documentation: func
                .symbol
                .and_then(|s| finished_file.get(s).description.clone())
                .map(|d| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: d,
                    })
                }),
            active_parameter: None,
        }],
        active_signature: Some(0),
        active_parameter: Some(u32::try_from(active_param).unwrap_or(u32::MAX)),
    })
}
