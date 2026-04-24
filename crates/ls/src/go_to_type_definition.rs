use lsp_types::{
    Location,
    request::{GotoTypeDefinitionParams, GotoTypeDefinitionResponse},
};
use resolver::{ArenaId, Database, FinishedFile, Source, line_index, parse, token_name_range};

use crate::conversions;

pub fn handle_go_to_type_definition(
    db: &Database,
    params: GotoTypeDefinitionParams,
) -> Option<GotoTypeDefinitionResponse> {
    let uri = params.text_document_position_params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position_params.position);

    let syntax = parse(db, file).syntax();
    let token = syntax.token_at_offset(offset).right_biased()?;

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let symbol_id = finished_file.symbol_at(range)?;

    let symbol = finished_file.get(symbol_id);
    let type_id = finished_file.type_to_symbol(symbol.typ)?;

    let file = type_id.file();
    let line_idx = line_index(db, file);
    let name_range = finished_file.get(type_id).name_range;

    let range = conversions::range(line_idx, name_range);

    let Some(path) = db.get_path(type_id.file()) else {
        eprintln!("Couldn't get uri when processing '{uri}'");
        return None;
    };

    Some(GotoTypeDefinitionResponse::Scalar(Location {
        range,
        uri: conversions::to_uri(&path),
    }))
}
