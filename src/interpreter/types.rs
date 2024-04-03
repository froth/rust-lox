#[derive(Debug, strum::Display)]
pub enum Type {
    Callable,
    Instance,
    String,
    Number,
    Boolean,
    Nil,
}
