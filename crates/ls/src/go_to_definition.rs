use anyhow::Result;
use ide::{ArenaId, Database, FinishedFile, Source, line_index, parse};
use lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location};

use crate::conversions;

pub fn handle_go_to_definition(
    db: &Database,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
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

    let finished_file = FinishedFile::new(db, file);
    let Some(id) = finished_file.symbol_at(&token) else {
        return Ok(None);
    };

    let file = id.file();
    let line_idx = line_index(db, file);
    let symbol = finished_file.get(id);

    let Some(range) = conversions::range(line_idx, symbol.name_range) else {
        eprintln!("Couldn't convert text_range at '{uri}'");
        return Ok(None);
    };

    let Some(path) = db.get_path(id.file()) else {
        eprintln!("Couldn't get uri when processing '{uri}'");
        return Ok(None);
    };

    Ok(Some(GotoDefinitionResponse::Scalar(Location {
        range,
        uri: conversions::to_uri(&path),
    })))
}
