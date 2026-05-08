use std::collections::HashMap;

use lsp_types::{RenameParams, TextEdit, Url, WorkspaceEdit};
use resolver::{
    FinishedFile, LocalKind, Source, SymbolKind, VScriptDatabase, parse, token_name_range,
};

use crate::positions;

pub fn handle_rename(
    db: &impl VScriptDatabase,
    params: RenameParams,
) -> anyhow::Result<Option<WorkspaceEdit>> {
    let uri = params.text_document_position.text_document.uri;
    let file = db
        .get_file(&uri)
        .ok_or_else(|| anyhow::format_err!("File not found in workspace"))?;
    let finished_file = FinishedFile::new(db, file);

    let line_idx = positions::line_index(db, file);
    let offset = positions::test_size(line_idx, params.text_document_position.position)
        .ok_or_else(|| anyhow::format_err!("Position is out of bounds"))?;

    let syntax = parse(db, file).syntax();
    let token = syntax
        .token_at_offset(offset)
        .right_biased()
        .ok_or_else(|| anyhow::format_err!("No token found"))?;

    let range = token_name_range(&token);

    let Some(symbol_id) = finished_file.symbol_at(range) else {
        return Ok(None);
    };

    if finished_file.get(symbol_id).kind == SymbolKind::Local(LocalKind::VariedArgs) {
        return Ok(None);
    }

    let name = file.text(db)[range.start().into()..range.end().into()].to_string();
    let new_name = params.new_name;

    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    if matches!(finished_file.get(symbol_id).kind, SymbolKind::Local(_)) {
        if let Some(ranges) = finished_file.symbol_to_ranges().get(&symbol_id) {
            for &text_range in ranges {
                let range = positions::range(line_idx, text_range).ok_or_else(|| {
                    anyhow::format_err!("Couldn't convert text range to lsp range")
                })?;

                changes.entry(uri.clone()).or_default().push(TextEdit {
                    range,
                    new_text: new_name.clone(),
                });
            }
        }

        if changes.is_empty() {
            return Ok(None);
        }

        return Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }));
    }

    for entry in db.get_files() {
        let (url, &candidate_file) = entry.pair();

        let text = candidate_file.text(db);
        if !text.contains(&*name) {
            continue;
        }

        let candidate = FinishedFile::new(db, candidate_file);

        let Some(ranges) = candidate.symbol_to_ranges().get(&symbol_id) else {
            continue;
        };

        let candidate_line_idx = positions::line_index(db, candidate_file);

        for &text_range in ranges {
            let range = positions::range(candidate_line_idx, text_range)
                .ok_or_else(|| anyhow::format_err!("Couldn't convert text range to lsp range"))?;

            changes.entry(url.clone()).or_default().push(TextEdit {
                range,
                new_text: new_name.clone(),
            });
        }
    }

    if changes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }))
    }
}
