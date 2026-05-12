use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent,
    MarkupKind,
};
use resolver::{FinishedFile, LocalKind, Source, SymbolKind, VScriptDatabase};

use crate::positions;

pub fn handle_inlay_hint<Db: VScriptDatabase>(
    db: &Db,
    params: InlayHintParams,
) -> anyhow::Result<Option<Vec<InlayHint>>> {
    let uri = params.text_document.uri;
    let file = db
        .get_file(&uri)
        .ok_or_else(|| anyhow::format_err!("File not found in workspace"))?;
    let finished_file = FinishedFile::new(db, file);

    let line_idx = positions::line_index(db, file);
    let range = positions::text_range(line_idx, params.range)
        .ok_or_else(|| anyhow::format_err!("Range is out of bounds"))?;

    let hints: Vec<_> = finished_file
        .all_symbols()
        .filter_map(|(_, symbol)| {
            if !range.contains_range(symbol.name_range) {
                return None;
            }

            if symbol.name.starts_with('_') {
                return None;
            }

            if !matches!(
                symbol.kind,
                SymbolKind::Local(
                    LocalKind::Exception | LocalKind::Parameter | LocalKind::Variable
                ) | SymbolKind::Property {
                    show_inlay_hint: true
                }
            ) {
                return None;
            }

            // skip if type is unknown or null - nothing useful to show
            if !symbol.typ.is_useful() {
                return None;
            }

            let label = format!(": {}", finished_file.type_to_str(&symbol.typ));
            let tooltip = if let Ok(id) = symbol.typ.to_instance()
                && let Some(class_symbol_id) = finished_file.get(id).symbol
            {
                let content = finished_file.symbol_markdown(class_symbol_id);

                Some(InlayHintTooltip::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }))
            } else {
                None
            };

            let position = positions::range(line_idx, symbol.name_range)?.end;

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

    if hints.is_empty() {
        Ok(None)
    } else {
        Ok(Some(hints))
    }
}
