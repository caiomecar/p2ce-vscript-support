use anyhow::Result;
use ide::{
    Database, ExpressionKind, FinishedFile, FunctionIdResolution, ParamsState, Source, Type,
    line_index, parse,
};
use lsp_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpParams, SignatureInformation,
};
use sq_3_parser::{AstNode, ast};

use crate::conversions;

pub fn handle_signature_help(
    db: &Database,
    params: SignatureHelpParams,
) -> Result<Option<SignatureHelp>> {
    let uri = params.text_document_position_params.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(None);
    };

    let Some(file) = db.get_file(&path) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let offset =
        conversions::test_size(line_idx, params.text_document_position_params.position).unwrap();

    let syntax = parse(db, file).syntax();
    let node = syntax
        .token_at_offset(offset)
        .right_biased()
        .and_then(|t| t.parent())
        .unwrap_or(syntax);

    let Some(call) = node.ancestors().find_map(ast::CallExpression::cast) else {
        return Ok(None);
    };

    let Some(callee) = call.callee() else {
        return Ok(None);
    };

    let finished_file = FinishedFile::new(db, file);
    let kind = finished_file.expr_at(callee.syntax().text_range());
    let (name, typ) = match kind {
        Some(ExpressionKind::Literal(typ)) => ("".to_owned(), typ),
        Some(ExpressionKind::Symbol(id)) => {
            let symbol = finished_file.get(id);
            (symbol.name.clone(), symbol.typ)
        }
        None => return Ok(None),
    };

    let id = match finished_file.to_function_id(typ, offset) {
        Some(FunctionIdResolution::Function(id)) => id,
        Some(FunctionIdResolution::DefaultConstructor) => {
            return Ok(Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: format!("{name}()"),
                    parameters: None,
                    documentation: None,
                    active_parameter: None,
                }],
                active_signature: Some(0),
                active_parameter: None,
            }));
        }
        None => return Ok(None),
    };

    let mut active_param: u32 = 0;

    for (i, arg) in call.arguments().enumerate() {
        if arg.syntax().text_range().contains_inclusive(offset) {
            active_param = i as u32;
            break;
        }

        // If cursor is after this arg, keep going
        if arg.syntax().text_range().end() < offset {
            active_param = i as u32 + 1;
        }
    }

    let func = finished_file.get(id);
    let mut label = format!("{}(", name);
    let mut param_infos = Vec::new();

    for (i, param_id) in func.params.iter().enumerate() {
        if i > 0 {
            label.push_str(", ");
        }

        let start = label.len();

        let param = finished_file.get(*param_id);
        label.push_str(&param.name);
        if param.typ != Type::Unknown {
            label.push_str(format!(": {}", finished_file.type_to_string(param.typ)).as_str());
        }

        let end = label.len();

        param_infos.push(ParameterInformation {
            label: ParameterLabel::LabelOffsets([start as u32, end as u32]),
            documentation: None,
        });
    }

    if let ParamsState::VarArgs(after, id) = func.params_state {
        if !func.params.is_empty() {
            label.push_str(", ");
        }

        let start = label.len();
        label.push_str("...vargv");
        let symbol = finished_file.get(id);
        if symbol.typ != Type::Unknown {
            label.push_str(format!(": {}", finished_file.type_to_string(symbol.typ)).as_str());
        }
        let end = label.len();

        param_infos.push(ParameterInformation {
            label: ParameterLabel::LabelOffsets([start as u32, end as u32]),
            documentation: None,
        });

        if active_param > after {
            active_param = after;
        }
    }

    label.push(')');

    if func.throws.is_some() {
        label.push('!');
    }

    if !matches!(func.ret, Type::Unknown | Type::Null) {
        label.push_str(format!(" -> {}", finished_file.type_to_string(func.ret)).as_str());
    }

    Ok(Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label,
            parameters: Some(param_infos),
            documentation: None,
            active_parameter: None,
        }],
        active_signature: Some(0),
        active_parameter: Some(active_param),
    }))
}
