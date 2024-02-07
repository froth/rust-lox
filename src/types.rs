use crate::value::Value;

#[derive(Debug, strum::Display)]
pub enum Type {
    String,
    Number,
    Boolean,
    Nil,
}

impl From<Value> for Type {
    fn from(value: Value) -> Self {
        match value {
            Value::String(_) => Type::String,
            Value::Number(_) => Type::Number,
            Value::Boolean(_) => Type::Boolean,
            Value::Nil => Type::Nil,
        }
    }
}
