use rowan::{TextRange, TextSize};

use crate::{Event, Marker, SyntaxError, SyntaxKind};

pub struct DocComment<'a> {
    text: &'a str,
    pos: TextSize,
    events: Vec<Event>,
    errors: Vec<SyntaxError>,
}

impl<'a> DocComment<'a> {
    pub fn parse(text: &'a str) -> (Vec<Event>, Vec<SyntaxError>) {
        let mut this = Self {
            text,
            // + 3 for /**
            pos: TextSize::new(3),
            events: Vec::new(),
            errors: Vec::new(),
        };
        this.finish_token(TextSize::new(0), SyntaxKind::DocSlashAsteriskAsterisk);
        this.body();
        let mut chars = this.text[this.pos.into()..].chars();
        if chars.next() == Some('*') && chars.next() == Some('/') {
            let start = this.start_token();
            this.pos += TextSize::new(2);
            this.finish_token(start, SyntaxKind::DocAsteriskSlash);
        }
        (this.events, this.errors)
    }

    fn start(&mut self) -> Marker {
        let index = self.events.len();
        self.events.push(Event::Pending);
        Marker(index)
    }

    fn finish(&mut self, marker: Marker, kind: SyntaxKind) {
        self.events[marker.0] = Event::Start { kind };
        self.events.push(Event::Finish);
    }

    fn drop(&mut self, marker: Marker) {
        self.events.remove(marker.0);
    }

    const fn start_token(&self) -> TextSize {
        self.pos
    }

    fn finish_token(&mut self, start: TextSize, kind: SyntaxKind) {
        self.events.push(Event::Token {
            kind,
            range: TextRange::new(start, self.pos),
        });
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += TextSize::new(u32::try_from(ch.len_utf8()).unwrap_or(u32::MAX));
        Some(ch)
    }

    fn peek(&self) -> Option<char> {
        let mut chars = self.text[self.pos.into()..].chars();
        let ch = chars.next()?;
        if ch == '*' && chars.next() == Some('/') {
            return None;
        }
        Some(ch)
    }

    fn next_char_range(&self) -> TextRange {
        let next_pos = self.peek().map_or(self.pos, |ch| {
            self.pos + TextSize::new(u32::try_from(ch.len_utf8()).unwrap_or(u32::MAX))
        });

        TextRange::new(self.pos, next_pos)
    }

    fn skip_trivia(&mut self) {
        while let Some(char) = self.peek() {
            match char {
                ' ' | '\t' | '\r' => {
                    let start = self.start_token();
                    self.next();
                    self.finish_token(start, SyntaxKind::Whitespace);
                }
                _ => return,
            }
        }
    }

    fn description(&mut self) {
        self.skip_trivia();
        // Skip leading blank lines before any content
        while self.peek() == Some('\n') {
            self.new_line();
        }

        if matches!(self.peek(), None | Some('@')) {
            return;
        }

        let m = self.start();
        loop {
            let line_m = self.start();
            let start = self.start_token();
            loop {
                match self.peek() {
                    None | Some('@') => {
                        if start == self.pos {
                            self.drop(line_m);
                        } else {
                            self.finish_token(start, SyntaxKind::DocText);
                            self.finish(line_m, SyntaxKind::DocDescriptionLine);
                        }
                        self.finish(m, SyntaxKind::DocDescription);
                        return;
                    }
                    Some('\n') => {
                        self.next();
                        self.finish_token(start, SyntaxKind::DocText);
                        self.finish(line_m, SyntaxKind::DocDescriptionLine);
                        self.after_new_line();
                        break;
                    }
                    Some('\\') => {
                        self.next();
                        if self.peek() == Some('@') {
                            self.next();
                        }
                    }
                    _ => {
                        self.next();
                    }
                }
            }
        }
    }

    fn identifier(&mut self, message: String) -> TextRange {
        if !self
            .peek()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        {
            self.errors.push(SyntaxError {
                message,
                range: self.next_char_range(),
            });
            return TextRange::empty(self.pos);
        }

        let start = self.start_token();
        self.next();
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.next();
        }
        self.finish_token(start, SyntaxKind::DocIdentifier);

        TextRange::new(start, self.pos)
    }

    fn possible_type(&mut self) -> bool {
        self.skip_trivia();
        if self.peek() == Some('{') {
            self.parse_type();
            true
        } else {
            false
        }
    }

    // @param {type} name description
    fn parse_tag(&mut self) {
        let tag = self.start();

        let item = self.start();

        let start = self.start_token();
        assert_eq!(self.next(), Some('@'));
        self.finish_token(start, SyntaxKind::DocAt);

        let ident_range = self.identifier("Expected tag's name".to_owned());
        let tag_text = &self.text[ident_range];
        self.finish(item, SyntaxKind::DocTagItem);

        let kind = match tag_text {
            "param" => {
                let has_type = self.possible_type();

                self.skip_trivia();
                let name = self.start();
                self.identifier(if has_type {
                    "Expected parameter's name".to_owned()
                } else {
                    "Expected type ('{...}') or parameter's name".to_owned()
                });

                self.finish(name, SyntaxKind::DocName);

                SyntaxKind::ParamTag
            }
            "type" => {
                self.possible_type();
                SyntaxKind::TypeTag
            }
            "returns" | "return" => {
                self.possible_type();
                SyntaxKind::ReturnTag
            }
            "throws" | "throw" => {
                self.possible_type();
                SyntaxKind::ThrowTag
            }
            "yields" | "yield" => {
                self.possible_type();
                SyntaxKind::YieldTag
            }
            "varargs" | "vargv" => {
                self.possible_type();
                SyntaxKind::VarArgsTag
            }
            "entity" => SyntaxKind::EntityTag,
            "native" => SyntaxKind::NativeTag,
            "deprecated" => SyntaxKind::DeprecatedTag,
            "hide" => SyntaxKind::HideTag,
            "const" => SyntaxKind::ConstTag,
            "input" => SyntaxKind::InputTag,
            _ => {
                self.errors.push(SyntaxError {
                    message: format!("Unknown tag '{tag_text}'"),
                    range: ident_range,
                });
                SyntaxKind::UnknownTag
            }
        };

        self.description();
        self.finish(tag, kind);
    }

    fn parse_type(&mut self) {
        let m = self.start();

        let start = self.start_token();
        assert_eq!(self.next(), Some('{'));
        self.finish_token(start, SyntaxKind::DocOpenBrace);

        loop {
            self.skip_trivia();
            let name = self.start();
            let ident = self.identifier("Expected type's name".to_owned());
            if ident.is_empty() {
                self.drop(name);
            } else {
                self.finish(name, SyntaxKind::DocTypeName);
            }

            self.skip_trivia();
            match self.peek() {
                Some('|') => {
                    let start = self.start_token();
                    self.next();
                    self.finish_token(start, SyntaxKind::DocPipe);
                }
                Some('}') => {
                    let start = self.start_token();
                    self.next();
                    self.finish_token(start, SyntaxKind::DocCloseBrace);
                    break;
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    self.errors.push(SyntaxError {
                        message: "Expected '|' between types".to_owned(),
                        range: self.next_char_range(),
                    });
                }
                _ => break,
            }
        }

        self.finish(m, SyntaxKind::DocType);
    }

    fn body(&mut self) {
        self.skip_trivia();
        while let Some(ch) = self.peek() {
            match ch {
                '@' => self.parse_tag(),
                '\n' => self.new_line(),

                _ => self.description(),
            }
        }
    }

    fn new_line(&mut self) {
        let start = self.start_token();
        assert_eq!(self.next(), Some('\n'));
        self.finish_token(start, SyntaxKind::DocNewLine);
        self.after_new_line();
    }

    fn after_new_line(&mut self) {
        self.skip_trivia();
        match self.peek() {
            Some('*') => {
                let start = self.start_token();
                self.next();
                self.finish_token(start, SyntaxKind::DocAsterisk);
                self.skip_trivia();
            }
            Some(_) => {
                self.errors.push(SyntaxError {
                    message: "Expected '*' after the new line".to_owned(),
                    range: self.next_char_range(),
                });
            }
            None => {}
        }
    }
}
