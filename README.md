Rust implementation of a tree-walk interpreter for Lox.

Feature complete reimplementation of jlox in Rust with some minor changes and additions.
This also means that the design is not perfectly suited to be implemented in Rust. 
There is i.e. some amount of Rc<RecCell<T>> in here because the original was written in a garbage collected language.
I started a Rust reimplementation of clox which should(hopefully) be closer to canonical rust. It can be found [here](https://github.com/froth/rust-lox-vm)

Basically me rusting through Part II of http://craftinginterpreters.com/
