use line_index::{LineIndex, TextRange, TextSize, WideEncoding, WideLineCol};
use lsp_types::{DiagnosticSeverity, Position, Range};

pub fn test_size(line_index: &LineIndex, position: Position) -> Option<TextSize> {
    let wide = WideLineCol {
        line: position.line,
        col: position.character,
    };
    let line_col = line_index.to_utf8(WideEncoding::Utf16, wide)?;
    line_index.offset(line_col)
}

pub fn text_range(line_index: &LineIndex, range: Range) -> Option<TextRange> {
    Some(TextRange::new(
        test_size(line_index, range.start)?,
        test_size(line_index, range.end)?,
    ))
}

pub fn position(line_index: &LineIndex, offset: TextSize) -> Option<Position> {
    let line_col = line_index.try_line_col(offset)?;
    let wide = line_index.to_wide(WideEncoding::Utf16, line_col)?;
    Some(Position {
        line: wide.line,
        character: wide.col,
    })
}

pub fn range(line_index: &LineIndex, range: TextRange) -> Option<Range> {
    Some(Range {
        start: position(line_index, range.start())?,
        end: position(line_index, range.end())?,
    })
}

pub fn to_lsp_severity(d: ide::DiagnosticSeverity) -> DiagnosticSeverity {
    match d {
        ide::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
        ide::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
    }
}
