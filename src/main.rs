// TODO: add file i/o and command line tool
// TODO: add option to use MASM backend

extern crate blocks;

fn main() {
    let prog = "
        symbol Loop {
            set x = + x 128;
            set i = ~ i 1;

            cmp > i 0;
            ifgoto Loop;
            return;
        }
        
        set x = 0;
        set i = 3;
        call Loop;

        raw `10 7 0 10 8 0 29 -1`;
    ";

    match blocks::compile(prog) {
        Ok(v) => println!("Compiled:\n{:?}", v),
        Err(e) => println!("{}", e)
    }
}
