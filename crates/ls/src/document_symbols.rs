use crate::conversions;
use lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, SymbolKind as LspSymbolKind,
    SymbolTag,
};
use resolver::{
    Database, DisplayType, FinishedFile, PropertyKind, Source, Symbol, SymbolFlags, SymbolKind,
    line_index,
};

pub fn handle_document_symbols(
    db: &Database,
    params: DocumentSymbolParams,
) -> Option<DocumentSymbolResponse> {
    let uri = params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

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
        if symbol.kind == SymbolKind::Property(PropertyKind::Embedded) {
            return;
        }

        let Some(range) = conversions::range(line_idx, symbol.range) else {
            return;
        };

        let Some(name_range) = conversions::range(line_idx, symbol.name_range) else {
            return;
        };

        let kind = match DisplayType::from(symbol) {
            DisplayType::Function => LspSymbolKind::FUNCTION,
            DisplayType::Class => LspSymbolKind::CLASS,
            DisplayType::Variable => LspSymbolKind::VARIABLE,
            DisplayType::Constant => LspSymbolKind::CONSTANT,
            DisplayType::Field => LspSymbolKind::FIELD,
            DisplayType::Enum => LspSymbolKind::ENUM,
            DisplayType::EnumMember => LspSymbolKind::ENUM_MEMBER,
        };

        let name = if symbol.name.is_empty() {
            "<unnamed>".to_owned()
        } else {
            symbol.name.to_string()
        };

        if !symbol.range.contains_range(symbol.name_range) {
            eprintln!("'name_range' is outside of 'range'");
            dbg!(symbol);
            dbg!(range);
            dbg!(name_range);
            return;
        }

        #[allow(deprecated)]
        let doc_symbol = DocumentSymbol {
            name,
            detail: Some(finished_file.type_to_str(&symbol.typ).into_string()),
            kind,
            range,
            selection_range: name_range,
            children: if children.is_empty() {
                None
            } else {
                Some(children)
            },
            tags: symbol
                .flags
                .contains(SymbolFlags::DEPRECATED)
                .then(|| vec![SymbolTag::DEPRECATED]),
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
            let (psymbol, children) = stack
                .pop()
                .expect("We can enter this point only after .last is Some");
            build_symbol(&mut stack, psymbol, children);
        }
        stack.push((symbol, Vec::new()));
    }

    while let Some((symbol, children)) = stack.pop() {
        build_symbol(&mut stack, symbol, children);
    }

    Some(DocumentSymbolResponse::Nested(roots))
}
