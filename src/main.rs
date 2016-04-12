extern crate blocks;

fn main() {
    let prog = "
        symbol foo { return; return }
        symbol bar { return }

        set x = 0;
    ";

    match blocks::compile(prog) {
        Ok(v) => println!("Compiled:\n{:?}", v),
        Err(e) => println!("{}", e)
    }
}
