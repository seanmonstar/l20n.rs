pub enum Entry {
    Comment { content: String },
    Entity {
        id: Identifier,
        value: Value,
        traits: Option<Vec<Member>>,
    },
    Section { key: Keyword, body: Vec<Entry> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
}

pub struct Keyword {
    pub name: String,
}

pub struct Member {
    pub key: Keyword,
    pub value: Value,
    pub default: bool,
}

pub struct Expression {
    pub node: ExpressionKind,
}

pub enum ExpressionKind {
    IdentifierExpression { name: String },
}

pub enum PatternElement {
    TextElement { value: String },
    Placeable { expressions: Vec<Expression> },
}

pub enum Value {
    Pattern {
        source: String,
        elements: Vec<PatternElement>,
    },
}
