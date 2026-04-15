use anyhow::Result;
use ide::{Database, FinishedFile, LocalKind, PropertyKind, Source, SymbolKind, Type, line_index};
use lsp_types::{SemanticToken, SemanticTokens, SemanticTokensParams, SemanticTokensResult};

use crate::conversions;

enum TokenType {
    Variable = 0,
    Parameter = 1,
    Function = 2,
    Class = 3,
    Property = 4,
    Enum = 5,
    EnumMember = 6,
}
enum TokenModifier {
    None = 0,
    Readonly = 1 << 0,
    Static = 1 << 1,
}

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

    let mut entries: Vec<_> = finished_file
        .range_to_symbol()
        .clone()
        .into_iter()
        .collect();
    entries.sort_by_key(|(range, _)| range.start());

    let mut tokens = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for (range, id) in entries {
        let symbol = finished_file.get(id);

        let (token_type, modifiers) = match symbol.kind {
            SymbolKind::Local(kind) => match symbol.typ.0 {
                Type::Function(_) => (TokenType::Function, TokenModifier::None),
                Type::Class(_) => (TokenType::Class, TokenModifier::None),
                _ => {
                    if kind == LocalKind::Parameter {
                        (TokenType::Parameter, TokenModifier::None)
                    } else {
                        (TokenType::Variable, TokenModifier::None)
                    }
                }
            },
            SymbolKind::Property(kind) => {
                if kind == PropertyKind::Embedded && range == symbol.name_range {
                    continue;
                }

                let modifiers = if kind == PropertyKind::Yes {
                    TokenModifier::Static
                } else {
                    TokenModifier::None
                };
                match symbol.typ.0 {
                    Type::Function(_) => (TokenType::Function, modifiers),
                    Type::Class(_) => (TokenType::Class, modifiers),
                    _ => (TokenType::Property, modifiers),
                }
            }
            SymbolKind::Enum => (TokenType::Enum, TokenModifier::Readonly),
            SymbolKind::EnumMember => (TokenType::EnumMember, TokenModifier::Readonly),
            SymbolKind::Constant => (TokenType::Variable, TokenModifier::Readonly),
        };

        let Some(lsp_range) = conversions::range(line_idx, range) else {
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
            token_type: token_type as u32,
            token_modifiers_bitset: modifiers as u32,
        });

        prev_line = line;
        prev_start = start;
    }

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })))
}
