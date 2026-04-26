use std::path::Path;

use line_index::{LineIndex, TextRange, TextSize, WideEncoding, WideLineCol};
use lsp_types::{Position, Range, Url};

/// # Panics
///
/// Panics if the position is outside the bounds of the file.
/// Shouldn't panic for positions coming from LSP if `line_idx`
/// is in sync
pub fn test_size(line_idx: &LineIndex, position: Position) -> Option<TextSize> {
    let wide = WideLineCol {
        line: position.line,
        col: position.character,
    };

    let Some(line_col) = line_idx.to_utf8(WideEncoding::Utf16, wide) else {
        eprintln!(
            "Couldn't convert position {position:?} to a text size since it was missing from the line index"
        );
        return None;
    };

    line_idx.offset(line_col)
}

pub fn text_range(line_idx: &LineIndex, range: Range) -> Option<TextRange> {
    Some(TextRange::new(
        test_size(line_idx, range.start)?,
        test_size(line_idx, range.end)?,
    ))
}

pub fn position(line_idx: &LineIndex, offset: TextSize) -> Option<Position> {
    let Some(line_col) = line_idx.try_line_col(offset) else {
        eprintln!(
            "Couldn't convert text size {offset:?} to a position since it was missing from the line index"
        );
        return None;
    };

    let wide = line_idx.to_wide(WideEncoding::Utf16, line_col)?;

    Some(Position {
        line: wide.line,
        character: wide.col,
    })
}

pub fn range(line_idx: &LineIndex, range: TextRange) -> Option<Range> {
    Some(Range {
        start: position(line_idx, range.start())?,
        end: position(line_idx, range.end())?,
    })
}

pub fn to_uri(path: &Path) -> Option<Url> {
    Url::from_file_path(path).ok()
}
