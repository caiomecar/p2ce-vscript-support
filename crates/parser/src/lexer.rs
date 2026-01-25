use std::fmt::Display;

use crate::SyntaxError;
use crate::cst::SyntaxKind;

use phf::phf_map;
use rowan::{TextRange, TextSize};

static KEYWORDS: phf::Map<&'static str, SyntaxKind> = phf_map! {
    "base" => SyntaxKind::BaseKeyword,
    "break" => SyntaxKind::BreakKeyword,
    "case" => SyntaxKind::CaseKeyword,
    "catch" => SyntaxKind::CatchKeyword,
    "class" => SyntaxKind::ClassKeyword,
    "clone" => SyntaxKind::CloneKeyword,
    "const" => SyntaxKind::ConstKeyword,
    "constructor" => SyntaxKind::ConstructorKeyword,
    "continue" => SyntaxKind::ContinueKeyword,
    "default" => SyntaxKind::DefaultKeyword,
    "delete" => SyntaxKind::DeleteKeyword,
    "do" => SyntaxKind::DoKeyword,
    "else" => SyntaxKind::ElseKeyword,
    "enum" => SyntaxKind::EnumKeyword,
    "extends" => SyntaxKind::ExtendsKeyword,
    "false" => SyntaxKind::FalseKeyword,
    "foreach" => SyntaxKind::ForEachKeyword,
    "for" => SyntaxKind::ForKeyword,
    "function" => SyntaxKind::FunctionKeyword,
    "if" => SyntaxKind::IfKeyword,
    "in" => SyntaxKind::InKeyword,
    "instanceof" => SyntaxKind::InstanceOfKeyword,
    "local" => SyntaxKind::LocalKeyword,
    "null" => SyntaxKind::NullKeyword,
    "rawcall" => SyntaxKind::RawCallKeyword,
    "resume" => SyntaxKind::ResumeKeyword,
    "return" => SyntaxKind::ReturnKeyword,
    "static" => SyntaxKind::StaticKeyword,
    "switch" => SyntaxKind::SwitchKeyword,
    "this" => SyntaxKind::ThisKeyword,
    "throw" => SyntaxKind::ThrowKeyword,
    "true" => SyntaxKind::TrueKeyword,
    "try" => SyntaxKind::TryKeyword,
    "typeof" => SyntaxKind::TypeOfKeyword,
    "while" => SyntaxKind::WhileKeyword,
    "yield" => SyntaxKind::YieldKeyword,
    "__FILE__" => SyntaxKind::FileKeyword,
    "__LINE__" => SyntaxKind::LineKeyword,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Token {
    pub kind: SyntaxKind,
    pub range: TextRange,
}

struct Lexer<'a> {
    text: &'a str,
    pos: TextSize,
    token_start: TextSize,
    errors: Vec<SyntaxError>,
}

pub(crate) fn tokenise(text: &str) -> (Box<[Token]>, Vec<SyntaxError>) {
    let mut lexer = Lexer::new(text);
    let mut tokens = vec![];
    loop {
        let token = lexer.next_token();
        tokens.push(token);
        if token.kind == SyntaxKind::Eof {
            break;
        }
    }

    (tokens.into_boxed_slice(), lexer.errors)
}

impl Token {
    pub fn dummy() -> Self {
        Self {
            kind: SyntaxKind::Unknown,
            range: TextRange::empty(TextSize::new(0)),
        }
    }
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            pos: TextSize::new(0),
            token_start: TextSize::new(0),
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.text[self.pos.into()..].chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += TextSize::new(ch.len_utf8() as u32);
        Some(ch)
    }

    fn next_and_peek(&mut self) -> Option<char> {
        self.next();
        self.peek()
    }

    fn next_and_return(&mut self, kind: SyntaxKind) -> SyntaxKind {
        self.next();
        kind
    }

    fn start_token(&mut self) {
        self.token_start = self.pos;
    }

    fn token_range(&self) -> TextRange {
        TextRange::new(self.token_start, self.pos)
    }

    // This should only used for unterminated tokens that end in a new line or eof
    // In vscode, diagnostic range of width 0 will be displayed as a squiggly line
    // starting from the position of this range and ending when end of the line is
    // reached. This means that it's not ideal to use 0 width diagnostics for cases
    // other than the ones described above
    fn cursor_range(&self) -> TextRange {
        TextRange::empty(self.pos)
    }

    fn next_char_range(&self) -> TextRange {
        let next_pos = match self.peek() {
            Some(ch) => self.pos + TextSize::new(ch.len_utf8() as u32),
            None => self.pos,
        };

        TextRange::new(self.pos, next_pos)
    }

    fn error(&mut self, range: TextRange, message: impl Display) {
        self.errors.push(SyntaxError::new(range, message));
    }

    fn error_at_token(&mut self, message: impl Display) {
        self.error(self.token_range(), message);
    }

    fn next_token(&mut self) -> Token {
        let Some(chr) = self.peek() else {
            return Token {
                kind: SyntaxKind::Eof,
                range: self.token_range(),
            };
        };

        self.start_token();

        let kind = match chr {
            // \r can't be used as a line separator so it's better to just treat it as whitespace
            // If we have a malformed text "abc\r\rab\t\r\n" will not replicate the behaviour
            // of the compiler
            ' ' | '\t' | '\r' => self.whitespace(),
            '\n' => self.next_and_return(SyntaxKind::LineFeed),

            '#' => self.line_comment(),
            '/' => match self.next_and_peek() {
                Some('*') => self.block_comment(),
                Some('/') => self.line_comment(),
                Some('=') => self.next_and_return(SyntaxKind::SlashEquals),
                Some('>') => self.next_and_return(SyntaxKind::SlashGreaterThan),
                _ => SyntaxKind::Slash,
            },

            '=' => match self.next_and_peek() {
                Some('=') => {
                    // JavaScript '===' error recovery
                    if self.next_and_peek() == Some('=') {
                        self.next();
                        self.error_at_token("'===' is not a valid comparison operator");
                    }

                    SyntaxKind::EqualsEquals
                }
                // JavaScript '=>' recovery
                Some('>') => {
                    self.next();
                    self.error_at_token("'=>' is not a valid lambda expression");

                    SyntaxKind::Unknown
                }
                _ => SyntaxKind::Equals,
            },
            '<' => match self.next_and_peek() {
                Some('=') => match self.next_and_peek() {
                    Some('>') => self.next_and_return(SyntaxKind::LessThanEqualsGreaterThan),
                    _ => SyntaxKind::LessThanEquals,
                },
                Some('-') => self.next_and_return(SyntaxKind::LessThanMinus),
                Some('<') => {
                    // '<<<' error recovery
                    if self.next_and_peek() == Some('<') {
                        self.next();
                        self.error_at_token("'<<<' is not a valid shift operator");
                    }

                    SyntaxKind::LessThanLessThan
                }
                Some('/') => self.next_and_return(SyntaxKind::LessThanSlash),
                // PHP '<>' error recovery
                Some('>') => {
                    self.next();
                    self.error_at_token("'<>' is not a valid comparison operator");

                    SyntaxKind::ExclamationEquals
                }
                _ => SyntaxKind::LessThan,
            },
            '>' => match self.next_and_peek() {
                Some('=') => self.next_and_return(SyntaxKind::GreaterThanEquals),
                Some('>') => match self.next_and_peek() {
                    Some('>') => {
                        self.next_and_return(SyntaxKind::GreaterThanGreaterThanGreaterThan)
                    }
                    _ => SyntaxKind::GreaterThanGreaterThan,
                },
                _ => SyntaxKind::GreaterThan,
            },
            '!' => match self.next_and_peek() {
                Some('=') => {
                    // JavaScript '!==' error recovery
                    if self.next_and_peek() == Some('=') {
                        self.next();
                        self.error_at_token("'!==' is not a valid comparsison operator");
                    }

                    SyntaxKind::ExclamationEquals
                }
                _ => SyntaxKind::Exclamation,
            },

            '@' => match self.next_and_peek() {
                Some('"') => self.verbatim_string(),
                _ => SyntaxKind::At,
            },
            '"' => self.string(),
            // '`' recovery
            '`' => {
                self.error(self.next_char_range(), "'`' is not a valid quote");
                self.string()
            }
            '\'' => self.character(),

            '{' => self.next_and_return(SyntaxKind::OpenBrace),
            '}' => self.next_and_return(SyntaxKind::CloseBrace),
            '(' => self.next_and_return(SyntaxKind::OpenParenthesis),
            ')' => self.next_and_return(SyntaxKind::CloseParenthesis),
            '[' => self.next_and_return(SyntaxKind::OpenBracket),
            ']' => self.next_and_return(SyntaxKind::CloseBracket),

            ';' => self.next_and_return(SyntaxKind::Semicolon),
            ',' => self.next_and_return(SyntaxKind::Comma),
            '?' => match self.next_and_peek() {
                // JavaScript '??' error recovery
                Some('?') => {
                    if let Some('=') = self.next_and_peek() {
                        self.next();
                        self.error_at_token("'??=' is not a valid assignment operator");

                        SyntaxKind::Equals
                    } else {
                        self.error_at_token("'??' is not a valid operator");

                        SyntaxKind::BarBar
                    }
                }
                // JavaScript '?.' error recovery
                Some('.') => {
                    self.next();
                    self.error_at_token("'?.' is not a valid member access operator");

                    SyntaxKind::Dot
                }
                _ => SyntaxKind::Question,
            },
            '^' => match self.next_and_peek() {
                // '^=' error recovery
                Some('=') => {
                    self.next();
                    self.error_at_token("'^=' is not a valid assignment operator");

                    SyntaxKind::AsteriskEquals
                }
                _ => SyntaxKind::Caret,
            },
            '~' => match self.next_and_peek() {
                // Lua '~=' error recovery
                Some('=') => {
                    self.next();
                    self.error_at_token("'~=' is not a valid comparison operator");

                    SyntaxKind::ExclamationEquals
                }
                _ => SyntaxKind::Tilde,
            },
            '.' => match self.next_and_peek() {
                Some('.') => match self.next_and_peek() {
                    Some('.') => self.next_and_return(SyntaxKind::DotDotDot),
                    // Rust '..=' error recovery
                    Some('=') => {
                        self.next();
                        self.error_at_token("'..=' is not a valid operator");

                        // This is used for range, there's really no appropriate recovery
                        SyntaxKind::Unknown
                    }
                    // '..' error recovery
                    _ => {
                        self.error_at_token("'..' is not a valid operator");

                        SyntaxKind::DotDotDot
                    }
                },
                _ => SyntaxKind::Dot,
            },
            '&' => match self.next_and_peek() {
                Some('&') => match self.next_and_peek() {
                    // JavaScript '&&=' error recovery
                    Some('=') => {
                        self.next();
                        self.error_at_token("'&&=' is not a valid assignment operator");

                        SyntaxKind::Equals
                    }
                    _ => SyntaxKind::AmpersandAmpersand,
                },
                // '&=' error recovery
                Some('=') => {
                    self.next();
                    self.error_at_token("'&=' is not a valid assignment operator");

                    SyntaxKind::AsteriskEquals
                }
                _ => SyntaxKind::Ampersand,
            },
            '|' => match self.next_and_peek() {
                Some('|') => match self.next_and_peek() {
                    // JavaScript '||=' error recovery
                    Some('=') => {
                        self.next();
                        self.error_at_token("'||=' is not a valid assignment operator");

                        SyntaxKind::Equals
                    }
                    _ => SyntaxKind::BarBar,
                },
                // '|=' error recovery
                Some('=') => {
                    self.next();
                    self.error_at_token("'|=' is not a valid assignment operator");

                    SyntaxKind::PlusEquals
                }
                _ => SyntaxKind::Bar,
            },
            ':' => match self.next_and_peek() {
                Some(':') => self.next_and_return(SyntaxKind::ColonColon),
                // ':=' error recovery
                Some('=') => {
                    self.next();
                    self.error_at_token("':=' is not a valid assignment operator");

                    SyntaxKind::Equals
                }
                _ => SyntaxKind::Colon,
            },
            '*' => match self.next_and_peek() {
                Some('=') => self.next_and_return(SyntaxKind::AsteriskEquals),
                // '**' error recovery
                Some('*') => {
                    self.next();
                    self.error_at_token("'**' is not a valid operator");

                    SyntaxKind::Asterisk
                }
                _ => SyntaxKind::Asterisk,
            },
            '%' => match self.next_and_peek() {
                Some('=') => self.next_and_return(SyntaxKind::PercentEquals),
                _ => SyntaxKind::Percent,
            },
            '-' => match self.next_and_peek() {
                Some('-') => self.next_and_return(SyntaxKind::MinusMinus),
                Some('=') => self.next_and_return(SyntaxKind::MinusEquals),
                // C '->' error recovery
                // Also can be used for return type annotation and it's also
                // used for switch case in Go but those cases are non recoverable
                Some('>') => {
                    self.next();
                    self.error_at_token("'->' is not a valid member access operator");

                    SyntaxKind::Dot
                }
                _ => SyntaxKind::Minus,
            },

            '+' => match self.next_and_peek() {
                Some('+') => self.next_and_return(SyntaxKind::PlusPlus),
                Some('=') => self.next_and_return(SyntaxKind::PlusEquals),
                _ => SyntaxKind::Plus,
            },

            '0'..='9' => self.number(),

            _ if chr.is_alphabetic() || chr == '_' || chr == '$' => self.identifier_or_keyword(),

            _ => {
                self.next();

                self.error_at_token(format!("Unexpected character '{chr}'"));

                SyntaxKind::Unknown
            }
        };

        Token {
            kind,
            range: self.token_range(),
        }
    }

    //  , \t, \r, \n
    fn whitespace(&mut self) -> SyntaxKind {
        assert!(matches!(self.peek(), Some(' ' | '\t' | '\r')));
        while matches!(self.next_and_peek(), Some(' ' | '\t' | '\r')) {}

        // let value = self.current_token_value();
        // SyntaxKind::Whitespace(value)
        SyntaxKind::Whitespace
    }

    // // ...
    fn line_comment(&mut self) -> SyntaxKind {
        while let Some(chr) = self.next_and_peek()
            && chr != '\n'
        {}

        // let value = self.current_token_value();
        // SyntaxKind::LineComment(value)
        SyntaxKind::LineComment
    }

    // /* ... */
    fn block_comment(&mut self) -> SyntaxKind {
        self.next();
        let mut is_doc = false;
        if let Some('*') = self.peek() {
            if let Some('/') = self.next_and_peek() {
                // let value = self.current_token_value();
                // return SyntaxKind::BlockComment(value);
                return self.next_and_return(SyntaxKind::BlockComment);
            }
            is_doc = true;
        }

        loop {
            match self.peek() {
                None => {
                    self.error(
                        self.cursor_range(),
                        "Unterminated block comment ('*/' expected)",
                    );
                    break;
                }
                Some('*') => {
                    if let Some('/') = self.next_and_peek() {
                        self.next();
                        break;
                    }
                }
                _ => {
                    self.next();
                }
            }
        }
        /*
        let value = self.current_token_value();

        if is_doc {
            SyntaxKind::DocComment(value)
        } else {
            SyntaxKind::BlockComment(value)
        } */

        if is_doc {
            SyntaxKind::DocComment
        } else {
            SyntaxKind::BlockComment
        }
    }

    // Called after body character of a string / char literal is expected
    fn literal_character(&mut self) -> Option<char> {
        match self.peek() {
            None | Some('\r' | '\n') => None,

            Some('\\') => match self.next_and_peek() {
                Some('x') => Some(self.hex_escape(2)),
                Some('u') => Some(self.hex_escape(4)),
                Some('U') => Some(self.hex_escape(8)),

                Some('t') => {
                    self.next();
                    Some('\t')
                }
                Some('a') => {
                    self.next();
                    Some('\x07')
                }
                Some('b') => {
                    self.next();
                    Some('\x08')
                }
                Some('n') => {
                    self.next();
                    Some('\n')
                }
                Some('r') => {
                    self.next();
                    Some('\r')
                }
                Some('v') => {
                    self.next();
                    Some('\x0b')
                }
                Some('f') => {
                    self.next();
                    Some('\x0c')
                }
                Some('0') => {
                    self.next();
                    Some('\0')
                }
                Some('\\') => {
                    self.next();
                    Some('\\')
                }
                Some('"') => {
                    self.next();
                    Some('"')
                }
                Some('\'') => {
                    self.next();
                    Some('\'')
                }

                Some(esc) => {
                    let start = self
                        .pos
                        .checked_sub(TextSize::new(1))
                        .expect("We must have read at least 1 character before this point");

                    let end = self
                        .pos
                        .checked_add(TextSize::new(1))
                        .expect("Number of characters is overflowing");

                    self.error(
                        TextRange::new(start, end),
                        format!("Invalid escape sequence '{esc}'"),
                    );

                    Some('\\')
                }

                None => None,
            },

            Some(chr) => {
                self.next();

                Some(chr)
            }
        }
    }

    // "..." `...`
    fn string(&mut self) -> SyntaxKind {
        let quote = self.next().unwrap();
        assert!(matches!(quote, '"' | '`'));

        loop {
            if self.peek() == Some(quote) {
                self.next();
                break;
            }

            if self.literal_character().is_none() {
                self.error(self.cursor_range(), "Unterminated string literal");
                break;
            };
        }

        SyntaxKind::String
    }

    // '...'
    fn character(&mut self) -> SyntaxKind {
        assert_eq!(self.next(), Some('\''));

        let mut len: u32 = 0;

        loop {
            if let Some('\'') = self.peek() {
                self.next();
                break;
            }

            len += match self.literal_character() {
                Some(chr) => chr.len_utf16(),
                None => {
                    self.error(self.cursor_range(), "Unterminated character literal");
                    break;
                }
            } as u32;
        }

        if len == 0 {
            self.error_at_token("Empty character literal");
        }

        if len > 1 {
            self.error_at_token("Character literal may only contain one codepoint");
        }

        SyntaxKind::Character
    }

    // @"..."
    fn verbatim_string(&mut self) -> SyntaxKind {
        assert_eq!(self.next(), Some('"'));

        loop {
            match self.peek() {
                Some('"') => {
                    if let Some('"') = self.next_and_peek() {
                        self.next();
                        '"'
                    } else {
                        break;
                    }
                }
                Some(chr) => {
                    self.next();
                    chr
                }
                None => {
                    self.error(self.cursor_range(), "Unterminated verbatim string literal");
                    break;
                }
            };
        }

        SyntaxKind::VerbatimString
    }

    // \x12, \u1234, \U12345678
    fn hex_escape(&mut self, digits: u8) -> char {
        assert!(matches!(self.peek(), Some('x' | 'u' | 'U')));
        self.next();

        if !matches!(self.peek(), Some('a'..='f' | 'A'..='F' | '0'..='9')) {
            self.error(self.next_char_range(), "Hexadecimal number expected");
            return ' ';
        }

        let start = self.pos;

        let mut value: u32 = 0;

        for _i in 0..digits {
            let Some(chr) = self.peek() else {
                break;
            };

            let digit = match chr {
                '0'..='9' => (chr as u8) - b'0',
                'a'..='f' => (chr as u8) - b'a' + 10,
                'A'..='F' => (chr as u8) - b'A' + 10,
                _ => break,
            };

            value = (value << 4) | digit as u32;
            eprintln!("{:?}", value);

            self.next();
        }

        let end = self.pos;

        char::from_u32(value).unwrap_or_else(|| {
            self.error(
                TextRange::new(start, end),
                "Invalid unicode character escape",
            );
            ' '
        })
    }

    // wow, local, function, yes
    fn identifier_or_keyword(&mut self) -> SyntaxKind {
        while let Some(chr) = self.peek() {
            if chr.is_ascii_alphanumeric() || chr == '_' {
                self.next();
            } else if chr.is_alphanumeric() || chr == '$' {
                self.error(
                    self.next_char_range(),
                    format!("Character '{chr}' is not allowed in the identifier"),
                );
                self.next();
            } else {
                break;
            }
        }

        match KEYWORDS.get(&self.text[self.token_range()]) {
            Some(&kind) => kind,
            None => SyntaxKind::Identifier,
        }
    }

    // 1233, 1e-21, 1.21, 1231.213e-2
    fn number(&mut self) -> SyntaxKind {
        let initial = self.next().unwrap();
        assert!(matches!(initial, '0'..='9'));

        if initial == '0' {
            match self.peek() {
                Some('0'..='7') => {
                    self.octal_number();
                    // let value = self.current_token_value();
                    // return SyntaxKind::Integer(value);
                    return SyntaxKind::Integer;
                }
                Some('x' | 'X') => {
                    self.hexadecimal_number();
                    // let value = self.current_token_value();
                    // return SyntaxKind::Integer(value);
                    return SyntaxKind::Integer;
                } /*
                Some('8' | '9') => {
                self.diagnostics.push(Diagnostic::warning(
                self.current_token_range(),
                "Leading zero can be removed",
                ));
                }
                 */
                _ => {}
            }
        }

        let mut is_float = false;

        loop {
            match self.peek() {
                Some('.') => {
                    is_float = true;
                    self.next();
                }
                Some('0'..='9') => {
                    self.next();
                }
                Some('e' | 'E') => {
                    is_float = true;
                    self.next();

                    if let Some('-' | '+') = self.peek() {
                        self.next();
                    }

                    match self.peek() {
                        Some('0'..='9') => {
                            self.next();
                        }
                        _ => {
                            self.error(self.next_char_range(), "Exponent expected");
                        }
                    }
                }
                // Some('a'..='z' | 'A'..='Z') => {
                //     self.error(
                //         self.next_char_range(),
                //         "Letters are only allowed inside a hexadecimal number",
                //     );
                //     self.next();
                // }
                // Some('_') => {
                //     self.error(
                //         self.next_char_range(),
                //         "'_' is not allowed as a number separator",
                //     );
                //     self.next();
                // }
                _ => break,
            }
        }

        /* let value = self.current_token_value();
        if is_float {
            SyntaxKind::Float(value)
        } else {
            SyntaxKind::Integer(value)
        } */

        if is_float {
            SyntaxKind::Float
        } else {
            SyntaxKind::Integer
        }
    }

    fn octal_number(&mut self) {
        assert!(matches!(self.next(), Some('0'..='7')));
        loop {
            match self.peek() {
                Some('0'..='7') => {
                    self.next();
                }
                Some('8' | '9') => {
                    self.error(
                        self.next_char_range(),
                        "Invalid octal digit, expected number from 0 to 7",
                    );
                    self.next();
                }
                // Some('a'..='z' | 'A'..='Z') => {
                //     self.error(
                //         self.next_char_range(),
                //         "Letters are only allowed inside a hexadecimal number",
                //     );
                //     self.next();
                // }
                // Some('_') => {
                //     self.error(
                //         self.next_char_range(),
                //         "'_' is not allowed as a number separator",
                //     );
                //     self.next();
                // }
                _ => return,
            }
        }
    }

    fn hexadecimal_number(&mut self) {
        assert!(matches!(self.next(), Some('x' | 'X')));
        loop {
            match self.peek() {
                Some('0'..='9' | 'a'..='f' | 'A'..='F') => {
                    self.next();
                }
                // Some('g'..='z' | 'G'..='Z') => {
                //     self.error(
                //         self.next_char_range(),
                //         "Invalid hexadecimal digit, expected number or letter from A to Z",
                //     );
                //     self.next();
                // }
                // Some('_') => {
                //     self.error(
                //         self.next_char_range(),
                //         "'_' is not allowed as a number separator",
                //     );
                //     self.next();
                // }
                _ => return,
            }
        }
    }
}
