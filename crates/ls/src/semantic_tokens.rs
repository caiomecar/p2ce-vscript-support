use anyhow::Result;
use ide::{Database, FinishedFile, Source, SymbolKind, Type, line_index};
use lsp_types::{SemanticToken, SemanticTokens, SemanticTokensParams, SemanticTokensResult};

use crate::conversions;

pub fn handle_semantic_tokens(
    db: &Database,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    let uri = params.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(None);
    };

    let Some(file) = db.get_file(&path) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let finished_file = FinishedFile::new(db, file);

    let mut entries: Vec<_> = finished_file.range_to_symbol().iter().collect();
    entries.sort_by_key(|(range, _)| range.start());

    let mut tokens = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for (range, id) in entries {
        let symbol = finished_file.get(*id);

        let (token_type, modifiers) = match symbol.kind {
            SymbolKind::Local => match symbol.typ {
                Type::Function(_) => (1, 0),
                Type::Class(_) => (2, 0),
                _ => (0, 0),
            },
            SymbolKind::Property(_) => match symbol.typ {
                Type::Function(_) => (1, 0),
                Type::Class(_) => (2, 0),
                _ => (3, 0),
            },
            SymbolKind::Enum => (4, 0),
            SymbolKind::EnumMember => (5, 1),
            SymbolKind::Constant => (0, 1),
        };

        let Some(lsp_range) = conversions::range(line_idx, *range) else {
            continue;
        };

        let line = lsp_range.start.line;
        let start = lsp_range.start.character;
        let length = range.len();

        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 {
            start - prev_start
        } else {
            start
        };

        tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: length.into(),
            token_type,
            token_modifiers_bitset: modifiers,
        });

        prev_line = line;
        prev_start = start;
    }

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })))
}
