use ide::{
    Database, FinishedFile, LocalKind, PropertyKind, Source, SymbolFlags, SymbolKind, Type,
    line_index,
};
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

bitflags::bitflags! {
    pub struct TokenModifier: u8 {
        const READONLY = 1 << 0;
        const STATIC = 1 << 1;
        const DEPRECATED = 1 << 2;
    }
}

pub fn handle_semantic_tokens(
    db: &Database,
    params: SemanticTokensParams,
) -> Option<SemanticTokensResult> {
    let uri = params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

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
        let mut modifiers = TokenModifier::empty();

        if symbol.flags.contains(SymbolFlags::CONST) {
            modifiers |= TokenModifier::READONLY;
        }

        if symbol.flags.contains(SymbolFlags::DEPRECATED) {
            modifiers |= TokenModifier::DEPRECATED;
        }

        let token_type = match symbol.kind {
            SymbolKind::Local(kind) => match symbol.typ {
                Type::Function(_) => TokenType::Function,
                Type::Class(_) => TokenType::Class,
                _ => {
                    if kind == LocalKind::Parameter {
                        TokenType::Parameter
                    } else {
                        TokenType::Variable
                    }
                }
            },
            SymbolKind::Property(kind) => {
                if kind == PropertyKind::Embedded && range == symbol.name_range {
                    continue;
                }

                if kind == PropertyKind::Yes {
                    modifiers |= TokenModifier::STATIC;
                }

                match symbol.typ {
                    Type::Function(_) => TokenType::Function,
                    Type::Class(_) => TokenType::Class,
                    _ => TokenType::Property,
                }
            }
            SymbolKind::Enum => {
                modifiers |= TokenModifier::READONLY;
                TokenType::Enum
            }
            SymbolKind::EnumMember => {
                modifiers |= TokenModifier::READONLY;
                TokenType::EnumMember
            }
            SymbolKind::Constant => {
                modifiers |= TokenModifier::READONLY;
                TokenType::Variable
            }
        };

        let lsp_range = conversions::range(line_idx, range);

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
            token_modifiers_bitset: u32::from(modifiers.bits()),
        });

        prev_line = line;
        prev_start = start;
    }

    Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    }))
}
