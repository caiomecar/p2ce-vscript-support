use std::collections::HashMap;

use anyhow::Result;
use ide::{ArenaId, Database, File, FileState, line_index, parse};
use lsp_types::{RenameParams, TextEdit, Url, WorkspaceEdit};
use rustc_hash::FxHashMap;

use crate::conversions;

pub fn handle_rename(
    db: &Database,
    docs: &FxHashMap<Url, File>,
    params: RenameParams,
) -> Result<Option<WorkspaceEdit>> {
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
    let Some(symbol_id) = file_state.symbol_at(token.text_range()) else {
        return Ok(None);
    };

    if symbol_id.file() != file {
        eprintln!("Mutli-file rename is not yet supported");
        return Ok(None);
    }

    let edits = file_state
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
