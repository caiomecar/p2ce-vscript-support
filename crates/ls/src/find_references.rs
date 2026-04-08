use anyhow::Result;
use ide::{Database, FinishedFile, Source, line_index, parse, token_name_range};
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

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let Some(reference_id) = finished_file.symbol_at(range) else {
        return Ok(None);
    };

    // can't do token.text() if the token is a string that got unquoted
    let reference = finished_file.get(reference_id);
    let name = reference.name.as_str();
    let name_range = reference.name_range;
    let mut all_locations = Vec::new();

    if let Some(ranges) = finished_file.symbol_to_ranges().get(&reference_id) {
        for range in ranges {
            if *range == name_range {
                continue;
            }

            all_locations.push(Location {
                range: conversions::range(line_idx, *range).unwrap(),
                uri: uri.clone(),
            });
        }
    }

    for (candidate_file, candidate_path) in db.all_files() {
        if candidate_file == file {
            continue;
        }

        let text = candidate_file.text(db);
        if !text.contains(name) {
            continue;
        }

        let candidate = FinishedFile::new(db, candidate_file);

        let Some(ranges) = candidate.symbol_to_ranges().get(&reference_id) else {
            continue;
        };

        let line_idx = line_index(db, candidate_file);
        let uri = conversions::to_uri(&candidate_path);

        for range in ranges {
            all_locations.push(Location {
                range: conversions::range(line_idx, *range).unwrap(),
                uri: uri.clone(),
            });
        }
    }

    Ok(Some(all_locations))
}
