use std::path::Path;

use line_index::{LineIndex, TextRange, TextSize, WideEncoding, WideLineCol};
use lsp_types::{Position, Range, Url};

/// # Panics
///
/// Panics if the position is outside the bounds of the file.
/// Shouldn't panic for positions coming from LSP if `line_idx`
/// is in sync
pub fn test_size(line_idx: &LineIndex, position: Position) -> TextSize {
    let wide = WideLineCol {
        line: position.line,
        col: position.character,
    };
    let line_col = line_idx
        .to_utf8(WideEncoding::Utf16, wide)
        .expect("Should only use positions from LSP which are always valid");

    line_idx
        .offset(line_col)
        .expect("Offset conversion should always succeed for valid line/col")
}

pub fn text_range(line_idx: &LineIndex, range: Range) -> TextRange {
    TextRange::new(
        test_size(line_idx, range.start),
        test_size(line_idx, range.end),
    )
}

/// # Panics
///
/// Panics if the offset is outside the bounds of the file.
/// Shouldn't panic for positions coming from the syntax tree
/// if `line_idx` is in sync unless syntax tree is malformed
pub fn position(line_idx: &LineIndex, offset: TextSize) -> Position {
    let line_col = line_idx
        .try_line_col(offset)
        .expect("Should only use offsets from syntax tree which are always valid");
    let wide = line_idx
        .to_wide(WideEncoding::Utf16, line_col)
        .expect("UTF-16 conversion should always succeed for valid line/col");

    Position {
        line: wide.line,
        character: wide.col,
    }
}

pub fn range(line_idx: &LineIndex, range: TextRange) -> Range {
    Range {
        start: position(line_idx, range.start()),
        end: position(line_idx, range.end()),
    }
}

pub fn to_uri(path: &Path) -> Url {
    Url::from_file_path(path).expect("Urls stored in the database are pulled from the device itself and are guaranteed to be valid")
}
