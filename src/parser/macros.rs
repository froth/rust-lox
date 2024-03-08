macro_rules! match_token {
    ($self:ident, $pattern:pat $(if $guard:expr)?) => {
        match $self.peek().token_type {
            $pattern $(if $guard)? => Some($self.advance()),
            _ => None
        }
    };
}

macro_rules! consume {
    ($self:ident, $pattern:pat $(if $guard:expr)?, $err_create: expr) => {{
        let peek = $self.peek();
        match peek.token_type {
            $pattern $(if $guard)? => $self.advance(),
            _ => {
                #[allow(clippy::redundant_closure_call)] return Err($err_create(peek));
            }
        }
    }};
}

macro_rules! check {
    ($self:ident, $pattern:pat $(if $guard:expr)?) => {
        matches!($self.peek().token_type, $pattern $(if $guard)?)
    };
}

pub(super) use check;
pub(super) use consume;
pub(super) use match_token;
