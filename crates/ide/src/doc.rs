use rustc_hash::FxHashMap;

use crate::Type;

pub struct JsDoc {
    pub description: String,
    pub tags: Vec<TagItem>,
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
    pub typ: Type,
    pub desc: String,
}

pub struct ParameterTag {
    pub name: Option<String>,
    pub typ: Type,
    pub desc: String,
}

pub enum JsDocType {
    Known(Type),
    Unknown,
}

// impl JsDoc {
//     pub fn new(text: &str) -> JsDoc {
//         let mut doc_description = Vec::new();
//         let mut tag_description = Vec::new();
//         let mut current_tag = None;
//         let mut tags = Vec::new();
//         for line in text.lines() {
//             let line = line.trim().trim_start_matches(['/', '*', ' ']);
//             let Some(rest) = line.strip_prefix('@') else {
//                 if current_tag.is_none() {
//                     doc_description.push(line);
//                 } else {
//                     tag_description.push(line);
//                 }
//                 continue;
//             };

//             let (tag, rest) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
//         }
//         JsDoc {
//             description: doc_description.join("\n"),
//             tags,
//         }
//     }

//     pub fn tag(text: &str) -> (TagItem, &str) {
//         let (tag, rest) = text.split_once(char::is_whitespace).unwrap_or((rest, ""));
//         match tag {
//             "return" | "returns" => {
//                 if let Some(rest) = rest.strip_prefix('{') {
//                     if let Some((typ, rest)) = rest.split_once('}') {
//                         let name = rest.trim().split_whitespace().next().map(str::to_owned);
//                         return (TagItem::Return(ReturnTag { typ: () });
//                     }
//                 }
//                 return TagItem::Return(ReturnTag { typ: JsDocType::Unknown })
//             }
//             "param" => {

//             }
//         }
//     }

//     pub fn typ(text: &str)
// }
