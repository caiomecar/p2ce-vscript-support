use anyhow::Result;
use ide::{Database, File, FileState, line_index, parse};
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind, Url};
use rustc_hash::FxHashMap;

use crate::conversions;

pub fn handle_hover(
    db: &Database,
    docs: &FxHashMap<Url, File>,
    params: HoverParams,
) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;
    let Some(&file) = docs.get(&uri) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let offset =
        conversions::test_size(line_idx, params.text_document_position_params.position).unwrap();

    let syntax = parse(db, file).syntax();
    let Some(token) = syntax.token_at_offset(offset).right_biased() else {
        return Ok(None);
    };

    let file_state = FileState::Finished(db, file);
    let Some(id) = file_state.symbol_at(token.text_range()) else {
        return Ok(None);
    };

    let symbol = file_state.get(id);
    let content = symbol.display(&file_state).to_string();

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```sqDoc\n{content}\n```"),
        }),
        range: None,
    }))
}
