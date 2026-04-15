use anyhow::Result;
use ide::{ArenaId, Database, FinishedFile, Source, line_index, parse, token_name_range};
use lsp_types::{
    Location,
    request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse},
};

use crate::conversions;

pub fn handle_go_to_type_definition(
    db: &Database,
    params: GotoTypeDefinitionParams,
) -> Result<Option<GotoTypeDefinitionResponse>> {
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
    let Some(token) = syntax.token_at_offset(offset).right_biased() else {
        return Ok(None);
    };

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let Some(symbol_id) = finished_file.symbol_at(range) else {
        return Ok(None);
    };

    let symbol = finished_file.get(symbol_id);
    let Some(type_id) = finished_file.type_to_symbol(symbol.typ.0) else {
        return Ok(None);
    };

    let file = type_id.file();
    let line_idx = line_index(db, file);
    let name_range = finished_file.get(type_id).name_range;

    let Some(range) = conversions::range(line_idx, name_range) else {
        eprintln!("Couldn't convert text_range at '{uri}'");
        return Ok(None);
    };

    let Some(path) = db.get_path(type_id.file()) else {
        eprintln!("Couldn't get uri when processing '{uri}'");
        return Ok(None);
    };

    Ok(Some(GotoTypeDefinitionResponse::Scalar(Location {
        range,
        uri: conversions::to_uri(&path),
    })))
}
