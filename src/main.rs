mod token;
mod tree;
mod error;

use std::error::Error;

fn main() {
    let prog = "
        set @1 = @2;
    ";

    let tokens = match tree::build_token_tree(prog.to_string()) {
        Ok(v) => Some(v),
        Err(e) => {
            println!("{}", e);
            None
        }
    };

    if let Some(tree::Tree::Block(e)) = tokens {
        for token in e {
            println!("{:?}", token);
        }
    }
}
