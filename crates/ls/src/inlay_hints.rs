use ide::{Database, FinishedFile, LocalKind, PropertyKind, Source, SymbolKind, Type, line_index};
use lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, InlayHintTooltip, MarkupContent,
    MarkupKind,
};

use crate::conversions;

pub fn handle_inlay_hints(db: &Database, params: InlayHintParams) -> Option<Vec<InlayHint>> {
    let uri = params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let finished_file = FinishedFile::new(db, file);

    let range = conversions::text_range(line_idx, params.range);

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
            if matches!(symbol.typ, Type::Unknown | Type::Null) {
                return None;
            }

            let label = format!(": {}", finished_file.type_to_str(symbol.typ));
            let tooltip = if let Type::Instance(Some(id)) = symbol.typ
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

            let position = conversions::range(line_idx, symbol.name_range).end;

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

    Some(hints)
}
