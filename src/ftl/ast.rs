extern crate serde;

use self::serde::ser::Serializer;
use self::serde::ser::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Resource(pub Vec<Entry>);

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: Identifier,
    pub value: Option<Pattern>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traits: Option<Vec<Member>>,
}

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

#[derive(Serialize, Deserialize)]
pub struct Member {
    pub key: Keyword,
    pub value: Option<Pattern>,
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
