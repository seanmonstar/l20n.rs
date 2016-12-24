extern crate serde;

use self::serde::ser::Serializer;
use self::serde::ser::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Resource(pub Vec<Entry>);

// impl Serialize for Resource {
//   fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
//     where S: Serializer
//   {
//     let mut map = serializer.serialize_map(Some(2)).unwrap();
//     serializer.serialize_map_key(&mut map, "type");
//     serializer.serialize_map_value(&mut map, "Resource");
//     serializer.serialize_map_key(&mut map, "body");
//     serializer.serialize_map_value(&mut map, &self.0);
//     serializer.serialize_map_end(map)
//   }
// }

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: Identifier,
    pub value: Option<Pattern>,
    pub traits: Option<Vec<Member>>,
    pub comment: Option<String>,
}

// impl Serialize for Entity {
//   fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
//     where S: Serializer
//   {
//     let mut map = serializer.serialize_map(Some(5)).unwrap();
//     serializer.serialize_map_key(&mut map, "type");
//     serializer.serialize_map_value(&mut map, "Entity");
//     serializer.serialize_map_key(&mut map, "id");
//     serializer.serialize_map_value(&mut map, &self.id);
//     serializer.serialize_map_key(&mut map, "value");
//     serializer.serialize_map_value(&mut map, &self.value);
//     serializer.serialize_map_key(&mut map, "traits");
//     serializer.serialize_map_value(&mut map, &self.traits);
//     serializer.serialize_map_key(&mut map, "comment");
//     serializer.serialize_map_value(&mut map, &self.comment);
//     serializer.serialize_map_end(map)
//   }
// }

#[derive(Serialize, Deserialize)]
pub struct Comment(pub String);

#[derive(Serialize, Deserialize)]
pub struct Section {
    pub key: Keyword,
    pub body: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
pub enum Entry {
    Comment(Comment),
    Entity(Entity),
    Section(Section),
}

#[derive(Serialize, Deserialize)]
pub struct Identifier(pub String);

#[derive(Serialize, Deserialize)]
pub struct Keyword(pub String);

fn is_false(s: &bool) -> bool {
    return !*s;
}

#[derive(Serialize, Deserialize)]
pub struct Member {
    pub key: Keyword,
    pub value: Option<Pattern>,
    #[serde(skip_serializing_if = "is_false")]
    pub default: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Expression {
    pub node: ExpressionKind,
}

#[derive(Serialize, Deserialize)]
pub enum ExpressionKind {
    IdentifierExpression(String),
}

#[derive(Serialize, Deserialize)]
pub struct TextElement(pub String);

#[derive(Serialize, Deserialize)]
pub struct Placeable(pub Vec<Expression>);

#[derive(Serialize, Deserialize)]
pub enum PatternElement {
    TextElement(TextElement),
    Placeable(Placeable),
}

#[derive(Deserialize)]
pub struct Pattern {
    pub source: String,

    #[serde(skip_serializing)]
    pub elements: Vec<PatternElement>,
}

impl Serialize for Pattern {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.source)
    }
}
