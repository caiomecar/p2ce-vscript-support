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
    match s.find(char::is_whitespace) {
        Some(idx) => {
            let (first, rest) = s.split_at(idx);
            (first, rest.trim_start())
        }
        None => (s, ""),
    }
}

impl Doc {
    pub fn new(text: &str) -> Doc {
        let mut doc_description = Vec::new();
        let mut tags: Vec<Tag> = Vec::new();
        for line in text.lines() {
            let line = line.trim().trim_start_matches(['/', '*', ' ']);
            let Some(rest) = line.strip_prefix('@') else {
                if let Some(tag) = tags.last_mut() {
                    tag.description.push_str(line);
                } else {
                    doc_description.push(line);
                };
                continue;
            };

            if let Some(tag) = Doc::tag(rest) {
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

        match rest.find(|c| c == '}') {
            Some(idx) => {
                let (typ, rest) = rest.split_at(idx);
                let types = typ
                    .split('|')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();

                (Some(types), rest[1..].trim_start())
            }
            None => {
                let types = rest
                    .split('|')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();

                (Some(types), "")
            }
        }
    }

    pub fn tag(text: &str) -> Option<Tag> {
        let (tag, rest) = split_once_ws(text);
        let (item, rest) = match tag {
            "return" | "returns" => {
                let (typ, rest) = Doc::typ(rest);
                (
                    TagItem::Return(ReturnTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "type" => {
                let (typ, rest) = Doc::typ(rest);
                (
                    TagItem::Type(TypeTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "param" => {
                let (typ, rest) = Doc::typ(rest);
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
                let (typ, rest) = Doc::typ(rest);
                (
                    TagItem::Throw(ThrowTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "yield" | "yields" => {
                let (typ, rest) = Doc::typ(rest);
                (
                    TagItem::Yield(YieldTag {
                        typ: typ.map(|v| v.into_iter().map(str::to_owned).collect()),
                    }),
                    rest,
                )
            }
            "varargs" | "vargv" => {
                let (typ, rest) = Doc::typ(rest);
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
            _ => return None,
        };

        Some(Tag {
            item,
            description: rest.strip_suffix("*/").unwrap_or(rest).to_owned(),
        })
    }
}
