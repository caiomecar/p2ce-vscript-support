use sq_3_parser::{
    AstNode, SyntaxNode, SyntaxToken,
    ast::{BinaryExpression, HasDoc, Property, VariableDeclaration},
};

#[derive(Debug)]
pub struct Doc {
    pub description: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug)]
pub struct Tag {
    pub item: TagItem,
    pub description: String,
}

#[derive(Debug)]
pub enum TagItem {
    Return(ReturnTag),
    Parameter(ParameterTag),
    Type(TypeTag),
    Throw(ThrowTag),
    Yield(YieldTag),
    VarArgs(VarArgsTag),
    Native,
    Entity,
    Input,
    Const,
    Deprecated,
    Hide,
}

#[derive(Debug)]
pub struct ReturnTag {
    pub typ: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ParameterTag {
    pub name: String,
    pub typ: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct TypeTag {
    pub typ: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ThrowTag {
    pub typ: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct YieldTag {
    pub typ: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct VarArgsTag {
    pub typ: Option<Vec<String>>,
}

fn split_once_ws(s: &str) -> (&str, &str) {
    s.find(char::is_whitespace).map_or((s, ""), |idx| {
        let (first, rest) = s.split_at(idx);
        (first, rest.trim_start())
    })
}

impl Doc {
    pub fn new(text: &str) -> Self {
        let mut doc_description = Vec::new();
        let mut tags: Vec<Tag> = Vec::new();
        for line in text.lines() {
            let line = line.trim().trim_start_matches(['/', '*', ' ']);
            let Some(rest) = line.strip_prefix('@') else {
                if let Some(tag) = tags.last_mut() {
                    tag.description.push_str(line);
                } else {
                    doc_description.push(line);
                }
                continue;
            };

            if let Some(tag) = Self::tag(rest) {
                tags.push(tag);
            }
        }

        Self {
            description: doc_description.join("\n"),
            tags,
        }
    }

    pub fn typ(text: &str) -> (Option<Vec<&str>>, &str) {
        let Some(rest) = text.strip_prefix('{') else {
            return (None, text);
        };

        rest.find('}').map_or_else(
            || {
                let types = rest
                    .split('|')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();

                (Some(types), "")
            },
            |idx| {
                let (typ, rest) = rest.split_at(idx);
                let types = typ
                    .split('|')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();

                (Some(types), rest[1..].trim_start())
            },
        )
    }

    pub fn tag(text: &str) -> Option<Tag> {
        let (tag, rest) = split_once_ws(text);
        let (item, rest) = match tag {
            "return" | "returns" => {
                let (typ, rest) = Self::typ(rest);
                (
                    TagItem::Return(ReturnTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "type" => {
                let (typ, rest) = Self::typ(rest);
                (
                    TagItem::Type(TypeTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "param" => {
                let (typ, rest) = Self::typ(rest);
                let (name, rest) = split_once_ws(rest);
                (
                    TagItem::Parameter(ParameterTag {
                        name: name.to_owned(),
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "throw" | "throws" => {
                let (typ, rest) = Self::typ(rest);
                (
                    TagItem::Throw(ThrowTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "yield" | "yields" => {
                let (typ, rest) = Self::typ(rest);
                (
                    TagItem::Yield(YieldTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "varargs" | "vargv" => {
                let (typ, rest) = Self::typ(rest);
                (
                    TagItem::VarArgs(VarArgsTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "native" => (TagItem::Native, rest),
            "entity" => (TagItem::Entity, rest),
            "input" => (TagItem::Input, rest),
            "const" => (TagItem::Const, rest),
            "hide" => (TagItem::Hide, rest),
            "deprecated" => (TagItem::Deprecated, rest),
            _ => return None,
        };

        Some(Tag {
            item,
            description: rest.strip_suffix("*/").unwrap_or(rest).to_owned(),
        })
    }
}

pub fn parent_doc(node: &SyntaxNode) -> Option<SyntaxToken> {
    let parent = node.parent()?;
    // /** ... */
    // new <- function() {}
    if let Some(bin) = BinaryExpression::cast(parent.clone()) {
        return bin.doc();
    }

    // class a = {
    //    /** ... */
    //    prop = function() {}
    // }
    if let Some(prop) = Property::cast(parent.clone()) {
        return prop.doc();
    }

    // Initially wrapped in 'Initialiser' node
    let parent = parent.parent()?;
    let init = VariableDeclaration::cast(parent.clone())?;

    // local
    // /** ... */
    // a = function() {}
    init.doc().or_else(||
                    // /** ... */
                    // local a = function() {}
                    VariableDeclaration::cast(parent.parent()?)?.doc())
}
