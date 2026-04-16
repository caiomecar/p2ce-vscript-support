use std::collections::HashMap;

use ide::{
    Database, FinishedFile, LocalKind, Source, SymbolKind, line_index, parse, token_name_range,
};
use lsp_types::{RenameParams, TextEdit, Url, WorkspaceEdit};

use crate::conversions;

pub fn handle_rename(db: &Database, params: RenameParams) -> Option<WorkspaceEdit> {
    let uri = params.text_document_position.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position.position);

    let syntax = parse(db, file).syntax();
    let token = syntax.token_at_offset(offset).right_biased()?;

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let symbol_id = finished_file.symbol_at(range)?;

    if finished_file.get(symbol_id).kind == SymbolKind::Local(LocalKind::VariedArgs) {
        return None;
    }

    let name = file.text(db)[range.start().into()..range.end().into()].to_string();
    let new_name = params.new_name;

    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    if matches!(finished_file.get(symbol_id).kind, SymbolKind::Local(_)) {
        if let Some(ranges) = finished_file.symbol_to_ranges().get(&symbol_id) {
            for range in ranges {
                changes.entry(uri.clone()).or_default().push(TextEdit {
                    range: conversions::range(line_idx, *range),
                    new_text: new_name.clone(),
                });
            }
        }

        return Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        });
    }

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
                range: conversions::range(candidate_line_idx, *range),
                new_text: new_name.clone(),
            });
        }
    }

    Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}
