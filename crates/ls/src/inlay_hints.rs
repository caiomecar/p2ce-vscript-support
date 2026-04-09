use anyhow::Result;
use ide::{Database, FinishedFile, LocalKind, PropertyKind, Source, SymbolKind, Type, line_index};
use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent,
    MarkupKind,
};

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

    let range = conversions::text_range(line_idx, params.range).unwrap();

    let hints = finished_file
        .all_symbols()
        .filter_map(|(_, symbol)| {
            if !range.contains_range(symbol.name_range) {
                return None;
            }

            if !matches!(
                symbol.kind,
                SymbolKind::Local(
                    LocalKind::Exception | LocalKind::Parameter | LocalKind::Variable
                ) | SymbolKind::Property(PropertyKind::NewSlot)
            ) {
                return None;
            }

            // skip if type is unknown or null - nothing useful to show
            let (label, tooltip) = match symbol.typ {
                Type::Unknown | Type::Null => return None,
                Type::Instance(id) => {
                    if let Some(id) = id
                        && let Some(class_symbol_id) = finished_file.get(id).symbol
                    {
                        let symbol = finished_file.get(class_symbol_id);
                        let typ = &symbol.name;

                        let content = symbol.display(&finished_file);

                        let tooltip = InlayHintTooltip::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("```sqDoc\n{content}\n```"),
                        });
                        (format!(": {typ}"), Some(tooltip))
                    } else {
                        (": instance".to_owned(), None)
                    }
                }
                typ => (format!(": {typ}"), None),
            };

            let position = conversions::range(line_idx, symbol.name_range)?.end;

            Some(InlayHint {
                position,
                label: InlayHintLabel::String(label),
                kind: Some(InlayHintKind::TYPE),
                text_edits: None,
                tooltip,
                padding_left: Some(false),
                padding_right: Some(false),
                data: None,
            })
        })
        .collect();

    Ok(Some(hints))
}
