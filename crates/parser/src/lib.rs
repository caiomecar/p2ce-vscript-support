pub mod ast;
mod cst;
mod lexer;
mod parser;
mod token_set;

use crate::parser::Event;
use rowan::GreenNodeBuilder;
use std::fmt::Display;

pub use crate::cst::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
pub use rowan::{
    GreenNode, TextRange, TextSize,
    ast::{AstChildren, AstNode},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxError {
    range: TextRange,
    message: String,
}

impl SyntaxError {
    pub fn new(range: TextRange, message: impl Display) -> SyntaxError {
        Self {
            range,
            message: message.to_string(),
        }
    }

    pub fn range(&self) -> TextRange {
        self.range
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct Parse {
    green_node: GreenNode,
    errors: Box<[SyntaxError]>,
}

impl Parse {
    pub fn into_syntax(self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node)
    }

    pub fn new(text: &str) -> Parse {
        let (tokens, mut lex_errors) = lexer::tokenise(text);
        let (events, parse_errors) = parser::parse(tokens);

        let mut builder = GreenNodeBuilder::new();
        let mut i = 0;
        while i < events.len() {
            match events[i] {
                Event::Start { kind } => builder.start_node(kind.into()),
                Event::Finish => builder.finish_node(),
                Event::Token { kind, range } => builder.token(kind.into(), &text[range]),
                Event::Pending => {
                    panic!(
                        "Pending event found, rest of the tokens: {:#?}",
                        &events[i..]
                    )
                }
            }
            i += 1;
        }

        lex_errors.extend(parse_errors);

        Parse {
            green_node: builder.finish(),
            errors: lex_errors.into_boxed_slice(),
        }
    }

    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }
}
