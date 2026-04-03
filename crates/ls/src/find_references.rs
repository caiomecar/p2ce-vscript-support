use anyhow::Result;
use ide::{Database, File, FileState, line_index, parse};
use lsp_types::{Location, ReferenceParams, Url};
use rustc_hash::FxHashMap;

use crate::conversions;

pub fn handle_references(
    db: &Database,
    docs: &FxHashMap<Url, File>,
    params: ReferenceParams,
) -> Result<Option<Vec<Location>>> {
    let uri = params.text_document_position.text_document.uri;
    let Some(&file) = docs.get(&uri) else {
        eprintln!("Couldn't find file '{uri}'");
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position.position).unwrap();

    let syntax = parse(db, file).syntax();
    let Some(token) = syntax.token_at_offset(offset).right_biased() else {
        return Ok(None);
    };

    let file_state = FileState::Finished(db, file);
    let Some(reference_id) = file_state.symbol_at(token.text_range()) else {
        return Ok(None);
    };

    let locations = file_state
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
