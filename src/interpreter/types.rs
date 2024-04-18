#[derive(Debug, strum::Display)]
pub enum Type {
    Function,
    NativeFunction,
    Class,
    Instance,
    String,
    Number,
    Boolean,
    Nil,
}
