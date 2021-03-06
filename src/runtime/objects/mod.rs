use crate::syntax_tree::KodyFunctionData;

mod number;

pub use number::KodyNumber;

#[derive(Debug, Clone, PartialEq)]
pub struct KodyObject {
    pub value: Box<KodyValue>,
}

impl KodyObject {
    pub fn new() -> KodyObject {
        KodyObject {
            value: Box::new(KodyValue::Empty),
        }
    }

    pub fn from(value: KodyValue) -> KodyObject {
        KodyObject {
            value: Box::new(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KodyValue {
    Bool(bool),
    Number(KodyNumber),
    StringLiteral(String),
    Function(KodyFunctionData),
    NativeFunction(fn(Vec<KodyObject>) -> Result<KodyObject, String>),
    Empty,
}
