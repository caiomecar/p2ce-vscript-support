use anyhow::Result;
use ide::{Database, FinishedFile, Source, line_index, parse};
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

use crate::conversions;

pub fn handle_hover(db: &Database, params: HoverParams) -> Result<Option<Hover>> {
    let uri = params.text_document_position_params.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(None);
    };

    let Some(file) = db.get_file(&path) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let offset =
        conversions::test_size(line_idx, params.text_document_position_params.position).unwrap();

    let syntax = parse(db, file).syntax();
    let Some(token) = syntax.token_at_offset(offset).right_biased() else {
        return Ok(None);
    };

    let finished_file = FinishedFile::new(db, file);
    let Some(id) = finished_file.symbol_at(&token) else {
        return Ok(None);
    };

    let symbol = finished_file.get(id);
    let content = symbol.display(&finished_file).to_string();

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```sqDoc\n{content}\n```"),
        }),
        range: None,
    }))
}
