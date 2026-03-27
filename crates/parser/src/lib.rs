pub mod ast;
mod cst;
mod lexer;
mod parser;
mod token_set;

use crate::{ast::SourceFile, parser::Event};
use rowan::GreenNodeBuilder;

pub use crate::cst::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
pub use rowan::{
    GreenNode, TextRange, TextSize,
    ast::{AstChildren, AstNode, AstPtr, SyntaxNodePtr},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    message: String,
    range: TextRange,
}

impl SyntaxError {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn range(&self) -> TextRange {
        self.range
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parse {
    green_node: GreenNode,
    errors: Vec<SyntaxError>,
}

impl Parse {
    pub fn new(text: &str) -> Parse {
        let now = std::time::Instant::now();
        let (tokens, mut lex_errors) = lexer::tokenise(text);
        eprintln!("Lexing took {:?}", now.elapsed());
        eprintln!("Tokens: {}", tokens.len());

        let now = std::time::Instant::now();
        let (events, parse_errors) = parser::parse(tokens);
        eprintln!("Parsing took {:?}", now.elapsed());
        eprintln!("Events: {}", events.len());

        lex_errors.extend(parse_errors);

        let now = std::time::Instant::now();

        let mut builder = GreenNodeBuilder::new();
        for event in events.into_iter() {
            match event {
                Event::Start { kind } => builder.start_node(kind.into()),
                Event::Finish => builder.finish_node(),
                Event::Token { kind, range } => builder.token(kind.into(), &text[range]),
                Event::Pending => {
                    panic!("Pending event found, current tree: {:#?}", builder.finish())
                }
            }
        }

        eprintln!("Building a tree took {:?}", now.elapsed());

        Parse {
            green_node: builder.finish(),
            errors: lex_errors,
        }
    }

    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }

    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }

    pub fn source_file(&self) -> SourceFile {
        SourceFile::cast(self.syntax()).unwrap()
    }

    pub fn into_syntax(self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node)
    }

    pub fn finish(self) -> (SourceFile, Vec<SyntaxError>) {
        (self.source_file(), self.errors)
    }
}
