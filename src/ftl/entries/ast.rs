extern crate serde;
extern crate serde_json;

use self::serde_json::Map;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource(pub Map<String, String>);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub value: Option<Pattern>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub source: String,
}

