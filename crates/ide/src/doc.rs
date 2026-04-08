pub struct Doc {
    pub description: String,
    pub tags: Vec<Tag>,
}

pub struct Tag {
    item: TagItem,
    description: String,
}

pub enum TagItem {
    Return(ReturnTag),
    Parameter(ParameterTag),
}

pub struct ReturnTag {
    pub typ: Option<String>,
}

pub struct TypeTag {
    pub typ: Option<String>,
}

pub struct ParameterTag {
    pub name: Option<String>,
    pub typ: Option<String>,
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

    pub fn typ(text: &str) -> (Option<String>, String) {
        let Some(rest) = text.strip_prefix('{') else {
            return (None, text.to_owned());
        };

        rest.split_once('}')
            .map_or((None, text.to_owned()), |(typ, rest)| {
                (
                    Some(typ.to_owned()),
                    rest.split_whitespace().next().unwrap_or(rest).to_owned(),
                )
            })
    }

    pub fn name(text: &str) -> (Option<String>, String) {
        let mut iter = text.split_whitespace();
        let Some(name) = iter.next() else {
            return (None, text.to_owned());
        };

        let rest = if let Some(rest) = iter.next() {
            rest
        } else {
            ""
        };

        (Some(name.to_owned()), rest.to_owned())
    }

    pub fn tag(text: &str) -> Option<Tag> {
        let mut iter = text.split_whitespace();
        let tag = iter.next()?;
        let rest = iter.next().unwrap_or("");
        Some(match tag {
            "return" | "returns" => {
                let (typ, description) = Doc::typ(rest);
                Tag {
                    item: TagItem::Return(ReturnTag { typ }),
                    description,
                }
            }
            "type" => {
                todo!()
            }
            "param" => {
                let (typ, rest) = Doc::typ(rest);
                let (name, description) = Doc::name(&rest);
                Tag {
                    item: TagItem::Parameter(ParameterTag { name, typ }),
                    description,
                }
            }
            _ => return None,
        })
    }
}
