use crate::conversions;
use anyhow::Result;
use ide::{Database, FinishedFile, PropertyKind, Source, Symbol, SymbolKind, Type, line_index};
use lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, SymbolKind as LspSymbolKind,
};

pub fn handle_document_symbols(
    db: &Database,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>> {
    let uri = params.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(None);
    };

    let Some(file) = db.get_file(&path) else {
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

    let mut stack: Vec<(&Symbol, Vec<DocumentSymbol>)> = Vec::new();
    let mut roots = Vec::new();
    let mut build_symbol = |stack: &mut Vec<(&Symbol, Vec<DocumentSymbol>)>,
                            symbol: &Symbol,
                            children: Vec<DocumentSymbol>| {
        match symbol.kind {
            SymbolKind::Property(PropertyKind::Embedded) => return,
            // SymbolKind::Local(_) if stack.len() != 0 => return,
            _ => {}
        }

        let range = conversions::range(line_idx, symbol.range).unwrap();
        let name_range = conversions::range(line_idx, symbol.name_range).unwrap();
        let kind = match symbol.typ.0 {
            Type::Function(_) => LspSymbolKind::FUNCTION,
            Type::Class(_) => LspSymbolKind::CLASS,
            Type::Enum(_) => LspSymbolKind::ENUM,
            _ => match symbol.kind {
                SymbolKind::Constant => LspSymbolKind::CONSTANT,
                SymbolKind::EnumMember => LspSymbolKind::ENUM_MEMBER,
                SymbolKind::Property(_) => LspSymbolKind::FIELD,
                _ => LspSymbolKind::VARIABLE,
            },
        };

        let name = if symbol.name.len() > 0 {
            symbol.name.clone()
        } else {
            "<unnamed>".to_owned()
        };

        if !symbol.range.contains_range(symbol.name_range) {
            eprintln!("'name_range' is outside of 'range'");
            dbg!(symbol);
            dbg!(range);
            dbg!(name_range);
        }

        #[allow(deprecated)]
        let doc_symbol = DocumentSymbol {
            name,
            detail: Some(finished_file.type_to_string(symbol.typ.0)),
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
        };

        if let Some((_, parent_children)) = stack.last_mut() {
            parent_children.push(doc_symbol);
        } else {
            roots.push(doc_symbol);
        }
    };

    for symbol in &symbols {
        while let Some((parent, _)) = stack.last() {
            if parent.range.contains_range(symbol.range) {
                break;
            }
            let (psymbol, children) = stack.pop().unwrap();
            build_symbol(&mut stack, psymbol, children);
        }
        stack.push((symbol, Vec::new()));
    }

    while let Some((symbol, children)) = stack.pop() {
        build_symbol(&mut stack, symbol, children);
    }

    Ok(Some(DocumentSymbolResponse::Nested(roots)))
}
