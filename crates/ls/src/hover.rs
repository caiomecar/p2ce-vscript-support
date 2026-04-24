use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};
use resolver::{Database, FinishedFile, Source, line_index, parse, token_name_range};
use sq_3_parser::SyntaxKind;

use crate::conversions;

pub fn handle_hover(db: &Database, params: HoverParams) -> Option<Hover> {
    let uri = params.text_document_position_params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.text_document_position_params.position);

    let syntax = parse(db, file).syntax();
    let token = syntax.token_at_offset(offset).right_biased()?;

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let content = if let Some(id) = finished_file.symbol_at(range) {
        finished_file.symbol_markdown(id)
    } else if token.kind() == SyntaxKind::Identifier {
        format!("```sqDoc\n{}: unknown\n```", token.text())
    } else {
        return None;
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range: Some(conversions::range(line_idx, range)),
    })
}
