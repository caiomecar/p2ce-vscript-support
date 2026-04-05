use anyhow::Result;
use ide::{Database, FinishedFile, Source, line_index, parse};
use lsp_types::{Location, ReferenceParams};

use crate::conversions;

pub fn handle_references(db: &Database, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
    let uri = params.text_document_position.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(None);
    };

    let Some(file) = db.get_file(&path) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position.position).unwrap();

    let syntax = parse(db, file).syntax();
    let Some(token) = syntax.token_at_offset(offset).right_biased() else {
        return Ok(None);
    };

    let finished_file = FinishedFile::new(db, file);
    let Some(reference_id) = finished_file.symbol_at(token.text_range()) else {
        return Ok(None);
    };

    let locations = finished_file
        .name_kinds()
        .iter()
        .filter_map(|(range, id)| {
            if reference_id != *id {
                return None;
            }

            Some(Location {
                range: conversions::range(line_idx, *range)?,
                uri: uri.clone(),
            })
        })
        .collect();

    Ok(Some(locations))
}
