#[derive(Debug, Clone, PartialEq)]
pub enum KodyObject {
    Bool(bool),
    Number(String),
    StringLiteral(String),
    Empty,
}
