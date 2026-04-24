use lsp_types::{Location, ReferenceParams};
use resolver::{Database, FinishedFile, Source, SymbolKind, line_index, parse, token_name_range};

use crate::conversions;

pub fn handle_references(db: &Database, params: ReferenceParams) -> Option<Vec<Location>> {
    let uri = params.text_document_position.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position.position);

    let syntax = parse(db, file).syntax();
    let token = syntax.token_at_offset(offset).right_biased()?;

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let reference_id = finished_file.symbol_at(range)?;

    // can't do token.text() if the token is a string that got unquoted
    let reference_file = FinishedFile::new(db, finished_file.file());
    let reference = reference_file.get(reference_id);
    let name = reference.name.as_ref();
    let name_range = reference.name_range;
    let mut all_locations = Vec::new();

    if let Some(ranges) = reference_file.symbol_to_ranges().get(&reference_id) {
        for range in ranges {
            if *range == name_range {
                continue;
            }

            all_locations.push(Location {
                range: conversions::range(line_idx, *range),
                uri: uri.clone(),
            });
        }
    }

    if matches!(reference.kind, SymbolKind::Local(_)) {
        return Some(all_locations);
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
                range: conversions::range(line_idx, *range),
                uri: uri.clone(),
            });
        }
    }

    Some(all_locations)
}
