use anyhow::Result;
use ide::{Database, FinishedFile, Source, SymbolKind, Type, line_index};
use lsp_types::{InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams};

use crate::conversions;

pub fn handle_inlay_hints(
    db: &Database,
    params: InlayHintParams,
) -> Result<Option<Vec<InlayHint>>> {
    let uri = params.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(None);
    };

    let Some(file) = db.get_file(&path) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let finished_file = FinishedFile::new(db, file);

    let hints = finished_file
        .all_symbols()
        .filter_map(|(_, symbol)| {
            // only locals at their declaration
            // Possibly add <- property declarations too?
            if symbol.kind != SymbolKind::Local {
                return None;
            }

            // skip if type is unknown or null - nothing useful to show
            let label = match symbol.typ {
                Type::Unknown | Type::Null => return None,
                typ => format!(": {typ}"),
            };

            let position = conversions::range(line_idx, symbol.name_range)?.end;

            Some(InlayHint {
                position,
                label: InlayHintLabel::String(label),
                kind: Some(InlayHintKind::TYPE),
                text_edits: None,
                tooltip: None,
                padding_left: Some(false),
                padding_right: Some(false),
                data: None,
            })
        })
        .collect();

    Ok(Some(hints))
}
