use anyhow::Result;
use ide::{Database, FinishedFile, Source, line_index, parse, token_name_range};
use lsp_types::{PrepareRenameResponse, TextDocumentPositionParams};

use crate::conversions;

pub fn handle_prepare_rename(
    db: &Database,
    params: TextDocumentPositionParams,
) -> Result<Option<PrepareRenameResponse>> {
    let uri = params.text_document.uri;

    let Ok(path) = uri.to_file_path() else {
        return Ok(None);
    };

    let Some(file) = db.get_file(&path) else {
        return Ok(None);
    };

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.position).unwrap();

    let syntax = parse(db, file).syntax();
    let Some(token) = syntax.token_at_offset(offset).right_biased() else {
        return Ok(None);
    };

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let Some(_) = finished_file.symbol_at(range) else {
        return Ok(None);
    };

    Ok(Some(PrepareRenameResponse::Range(
        conversions::range(line_idx, range).unwrap(),
    )))
}
