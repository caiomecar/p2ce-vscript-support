use std::collections::HashMap;

use anyhow::Result;
use ide::{ArenaId, Database, FinishedFile, Source, line_index, parse};
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};

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

    let finished_file = FinishedFile::new(db, file);
    let Some(symbol_id) = finished_file.symbol_at(token.text_range()) else {
        return Ok(None);
    };

    if symbol_id.file() != file {
        eprintln!("Mutli-file rename is not yet supported");
        return Ok(None);
    }

    let edits = finished_file
        .name_kinds()
        .iter()
        .filter_map(|(range, id)| {
            if symbol_id != *id {
                return None;
            }

            Some(TextEdit {
                range: conversions::range(line_idx, *range)?,
                new_text: params.new_name.clone(),
            })
        })
        .collect();

    let mut changes = HashMap::new();
    changes.insert(uri, edits);

    Ok(Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    }))
}
