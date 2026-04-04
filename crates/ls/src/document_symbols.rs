use crate::conversions;
use ::line_index::LineIndex;
use anyhow::Result;
use ide::{Database, File, FinishedFile, Source, Symbol, SymbolKind, Type, line_index};
use lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, SymbolKind as LspSymbolKind, Url,
};
use rustc_hash::FxHashMap;
use sq_3_parser::TextRange;

pub fn handle_document_symbols(
    db: &Database,
    docs: &FxHashMap<Url, File>,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri;
    let Some(&file) = docs.get(&uri) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let finished_file = FinishedFile::new(db, file);

    let mut symbols: Vec<_> = finished_file
        .all_symbols()
        .map(|(_, symbol)| symbol)
        .collect();

    symbols.sort_by(|a, b| {
        a.range
            .start()
            .cmp(&b.range.start())
            .then(b.range.len().cmp(&a.range.len()))
    });

    fn build_symbol(
        symbol: &Symbol,
        children: Vec<DocumentSymbol>,
        line_idx: &LineIndex,
    ) -> Option<DocumentSymbol> {
        let range = conversions::range(line_idx, symbol.range)?;
        let name_range = conversions::range(line_idx, symbol.name_range)?;
        let kind = match symbol.typ {
            Type::Function(_) => LspSymbolKind::FUNCTION,
            Type::Class(_) => LspSymbolKind::CLASS,
            Type::Enum(_) => LspSymbolKind::ENUM,
            _ => match symbol.kind {
                SymbolKind::Constant => LspSymbolKind::CONSTANT,
                SymbolKind::EnumMember => LspSymbolKind::ENUM_MEMBER,
                SymbolKind::Property => LspSymbolKind::FIELD,
                _ => LspSymbolKind::VARIABLE,
            },
        };

        let name = if symbol.name.len() > 0 {
            symbol.name.clone()
        } else {
            "\"\"".to_owned()
        };

        #[allow(deprecated)]
        Some(DocumentSymbol {
            name,
            detail: Some(symbol.typ.to_string()),
            kind,
            range,
            selection_range: name_range,
            children: if children.is_empty() {
                None
            } else {
                Some(children)
            },
            tags: None,
            deprecated: None,
        })
    }

    let mut stack: Vec<(TextRange, &Symbol, Vec<DocumentSymbol>)> = Vec::new();
    let mut roots = Vec::new();

    for symbol in &symbols {
        while let Some((parent_range, _, _)) = stack.last() {
            if parent_range.contains_range(symbol.range) {
                break;
            }
            let (_, psymbol, children) = stack.pop().unwrap();
            if let Some(doc_sym) = build_symbol(psymbol, children, line_idx) {
                if let Some((_, _, parent_children)) = stack.last_mut() {
                    parent_children.push(doc_sym);
                } else {
                    roots.push(doc_sym);
                }
            }
        }
        stack.push((symbol.range, symbol, Vec::new()));
    }

    while let Some((_, symbol, children)) = stack.pop() {
        if let Some(doc_sym) = build_symbol(symbol, children, line_idx) {
            if let Some((_, _, parent_children)) = stack.last_mut() {
                parent_children.push(doc_sym);
            } else {
                roots.push(doc_sym);
            }
        }
    }

    Ok(Some(DocumentSymbolResponse::Nested(roots)))
}
