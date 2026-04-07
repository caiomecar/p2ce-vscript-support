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

    pub fn tag(text: &str) -> Option<Tag> {
        let (tag, rest) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
        Some(match tag {
            "return" | "returns" => {
                if let Some(rest) = rest.strip_prefix('{')
                    && let Some((typ, rest)) = rest.split_once('}')
                {
                    Tag {
                        item: TagItem::Return(ReturnTag {
                            typ: Some(typ.to_owned()),
                        }),
                        description: rest.to_owned(),
                    }
                } else {
                    Tag {
                        item: TagItem::Return(ReturnTag { typ: None }),
                        description: rest.to_owned(),
                    }
                }
            }
            "type" => {
                todo!()
            }
            "param" => {
                todo!();
                let name = rest.trim().split_whitespace().next().map(str::to_owned);
            }
            _ => return None,
        })
    }
}
