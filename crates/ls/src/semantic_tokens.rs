use anyhow::Result;
use ide::{Database, File, FileState, SymbolKind, Type, line_index};
use lsp_types::{SemanticToken, SemanticTokens, SemanticTokensParams, SemanticTokensResult, Url};
use rustc_hash::FxHashMap;

use crate::conversions;

pub fn handle_semantic_tokens(
    db: &Database,
    docs: &FxHashMap<Url, File>,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    let uri = params.text_document.uri;
    let Some(&file) = docs.get(&uri) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let file_state = FileState::Finished(db, file);

    let mut entries: Vec<_> = file_state.name_kinds().iter().collect();
    entries.sort_by_key(|(range, _)| range.start());

    let mut tokens = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for (range, id) in entries {
        let symbol = file_state.get(*id);

        let (token_type, modifiers) = match symbol.kind {
            SymbolKind::Local => match symbol.typ {
                Type::Function(_) => (1, 0),
                Type::Class(_) => (2, 0),
                _ => (0, 0),
            },
            SymbolKind::Property => match symbol.typ {
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
        let length = lsp_range.end.character - lsp_range.start.character;

        // LSP semantic tokens are delta-encoded
        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 {
            start - prev_start
        } else {
            start
        };

        tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
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
