use lsp_types::{PrepareRenameResponse, TextDocumentPositionParams};
use resolver::{Database, FinishedFile, Source, line_index, parse, token_name_range};

use crate::conversions;

pub fn handle_prepare_rename(
    db: &Database,
    params: TextDocumentPositionParams,
) -> Option<PrepareRenameResponse> {
    let uri = params.text_document.uri;

    let path = uri.to_file_path().ok()?;
    let file = db.get_file(&path)?;

    let line_idx = line_index(db, file);
    let offset = conversions::test_size(line_idx, params.position)?;

    let syntax = parse(db, file).syntax();
    let token = syntax.token_at_offset(offset).right_biased()?;

    let range = token_name_range(&token);

    let finished_file = FinishedFile::new(db, file);
    let _ = finished_file.symbol_at(range)?;

    Some(PrepareRenameResponse::Range(conversions::range(
        line_idx, range,
    )?))
}
