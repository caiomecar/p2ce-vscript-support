use anyhow::Result;
use ide::{Database, File, FileState, FindSymbol, SymbolKind, Type, line_index, parse};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Url};
use rustc_hash::FxHashMap;
use sq_3_parser::{AstNode, SyntaxKind, ast};

use crate::conversions;

pub fn handle_completions(
    db: &Database,
    docs: &FxHashMap<Url, File>,
    params: CompletionParams,
) -> Result<CompletionResponse> {
    let uri = params.text_document_position.text_document.uri;
    let Some(&file) = docs.get(&uri) else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let line_index = line_index(db, file);

    let offset =
        conversions::test_size(line_index, params.text_document_position.position).unwrap();

    let syntax = parse(db, file).syntax();

    let token = syntax.token_at_offset(offset).left_biased().and_then(|t| {
        if t.kind() == SyntaxKind::Whitespace {
            t.prev_token()
        } else {
            Some(t)
        }
    });

    let member = token.and_then(|t| {
        t.parent_ancestors()
            .find_map(|n| ast::MemberAccessExpression::cast(n))
    });

    let file_state = FileState::Finished(db, file);

    // 3. Decide if we're in member access
    let symbols = if let Some(member) = member {
        let range = member.syntax().text_range();

        if offset <= range.end() {
            // still inside or right after member access → show members
            if let Some(obj) = member.object() {
                let typ = file_state.type_at(obj.syntax().text_range());
                file_state
                    .members_of_type(typ, FindSymbol::BeforeIfInExecutionRange(offset))
                    .into_values()
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            file_state.symbols_at(offset)
        }
    } else {
        file_state.symbols_at(offset)
    };

    let items: Vec<CompletionItem> = symbols
        .iter()
        .filter_map(|id| {
            let symbol = file_state.get(*id);
            Some(CompletionItem {
                label: symbol.name.clone(),
                kind: Some(match symbol.typ {
                    Type::Enum(_) => CompletionItemKind::ENUM,
                    Type::Function(_) => CompletionItemKind::FUNCTION,
                    Type::Class(_) => CompletionItemKind::CLASS,
                    _ => match symbol.kind {
                        SymbolKind::Local => CompletionItemKind::VARIABLE,
                        SymbolKind::Constant => CompletionItemKind::CONSTANT,
                        SymbolKind::Property => CompletionItemKind::PROPERTY,
                        SymbolKind::Enum => CompletionItemKind::ENUM,
                        SymbolKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
                    },
                }),

                ..Default::default()
            })
        })
        .collect();

    Ok(CompletionResponse::Array(items))
}
