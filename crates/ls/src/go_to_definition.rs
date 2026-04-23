use std::path::PathBuf;

use ide::{
    ArenaId, Database, ExpressionKind, FinishedFile, Source, StringKind, Type, line_index, parse,
    token_name_range,
};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Range};
use sq_3_parser::{TextRange, TextSize};

use crate::conversions;

pub fn handle_go_to_definition(
    db: &Database,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let uri = params.text_document_position_params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position_params.position);

    let syntax = parse(db, file).syntax();
    let token = syntax.token_at_offset(offset).right_biased()?;

    let finished_file = FinishedFile::new(db, file);

    if let Some(ExpressionKind::Literal(Type::String {
        kind,
        literal: Some(literal),
    })) = finished_file.expr_kind_at(token.text_range())
        && kind == StringKind::Script
    {
        let path = PathBuf::from(finished_file.get(literal).text.to_string());
        if let Ok(script) = finished_file.db().get_script(path) {
            let path = db.get_path(script).expect("We got this file from db");
            return Some(GotoDefinitionResponse::Scalar(Location {
                range: Range::default(),
                uri: conversions::to_uri(&path),
            }));
        }
    }

    let range = token_name_range(&token);
    let id = finished_file.symbol_at(range)?;

    let file = id.file();
    let line_idx = line_index(db, file);
    let symbol = finished_file.get(id);

    if symbol.name_range == TextRange::empty(TextSize::new(0)) {
        return None;
    }

    let range = conversions::range(line_idx, symbol.name_range);

    let Some(path) = db.get_path(id.file()) else {
        eprintln!("Couldn't get uri when processing '{uri}'");
        return None;
    };

    Some(GotoDefinitionResponse::Scalar(Location {
        range,
        uri: conversions::to_uri(&path),
    }))
}
