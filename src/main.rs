// TODO: add file i/o and command line tool
//       add tags and write some libraries
//       add optimizations

extern crate blocks;

fn main() {
    let prog = "
        set #0 = 322;

        raw `10 7 0 10 8 0 29 -1`;
    ";

    match blocks::compile(prog) {
        Ok(v) => println!("Compiled:\n{:?}", v),
        Err(e) => println!("{}", e)
    }
}
