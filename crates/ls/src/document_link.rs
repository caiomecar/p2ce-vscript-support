use std::path::PathBuf;

use lsp_types::{DocumentLink, DocumentLinkParams};
use resolver::{
    Database, ExpressionKind, FinishedFile, Primitive, Source, StringKind, Type, line_index,
};

use crate::conversions;

pub fn handle_document_link(db: &Database, params: DocumentLinkParams) -> Vec<DocumentLink> {
    let uri = params.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Vec::new();
    };

    let Some(file) = db.get_file(&path) else {
        return Vec::new();
    };
    let line_idx = line_index(db, file);

    let finished_file = FinishedFile::new(db, file);

    finished_file
        .range_to_expr()
        .values()
        .filter_map(|expr| {
            let ExpressionKind::Literal(Type::Primitive(Primitive::String {
                kind,
                literal: Some(literal),
            })) = expr
            else {
                return None;
            };

            if *kind != StringKind::Script {
                return None;
            }

            let literal = finished_file.get(*literal);
            let rel_path = PathBuf::from(literal.text.to_string());
            let script = finished_file.db().get_script(rel_path).ok()?;
            let path = db.get_path(script)?;

            let uri = conversions::to_uri(&path)?;

            Some(DocumentLink {
                range: conversions::range(line_idx, literal.unquoted_range)?,
                target: Some(uri),
                tooltip: Some("Open file".to_owned()),
                data: None,
            })
        })
        .collect()
}
