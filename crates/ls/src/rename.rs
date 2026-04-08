use std::collections::HashMap;

use anyhow::Result;
use ide::{Database, FinishedFile, Source, line_index, parse, token_name_range};
use lsp_types::{RenameParams, TextEdit, Url, WorkspaceEdit};

use crate::conversions;

pub fn handle_rename(db: &Database, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
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
    let Some(symbol_id) = finished_file.symbol_at(range) else {
        return Ok(None);
    };

    let name = file.text(db)[range.start().into()..range.end().into()].to_string();
    let new_name = params.new_name;

    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    for (candidate_file, candidate_path) in db.all_files() {
        let text = candidate_file.text(db);
        if !text.contains(&*name) {
            continue;
        }

        let candidate = FinishedFile::new(db, candidate_file);

        let Some(ranges) = candidate.symbol_to_ranges().get(&symbol_id) else {
            continue;
        };

        let candidate_line_idx = line_index(db, candidate_file);
        let uri = conversions::to_uri(&candidate_path);

        for range in ranges {
            changes.entry(uri.clone()).or_default().push(TextEdit {
                range: conversions::range(candidate_line_idx, *range).unwrap(),
                new_text: new_name.clone(),
            });
        }
    }

    Ok(Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    }))
}
