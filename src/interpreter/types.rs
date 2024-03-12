#[derive(Debug, strum::Display)]
pub enum Type {
    Callable,
    String,
    Number,
    Boolean,
    Nil,
}
