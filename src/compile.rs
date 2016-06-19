// Stage 4 in compilation.
// Converts the generated IR into machine code.
// The IR is low level so this stage can be made very simple.
// The machine code generation is easy, so it also does the work of interpreting tags.
// An important step here is to organize the resulting code, which means it:
//
//   - reserves space for variables and symbol blocks
//   - inserts code for dynamic lookup of symbol and variable addresses

use token::Token;
use tree::build_token_tree;
use error::*;
use utils::*;
use compile_utils::*;
use ir::*;

use std::collections::HashMap;

const SEGMENT_SETUP: &'static str = "
12 8 9
10 0 $0
16 8 0
12 8 5

10 0 $1
16 8 0
12 7 5

29 $2
";

const CLEANUP: &'static [i32] = &[
    10, 7, 0,
    10, 8, 0,
    35
];

pub fn compile(prog: &str) -> Result<Vec<i32>, BlocksError> {
    let tree = build_token_tree(prog.to_string())?;
    let mut ir = build_ir(TokenWrapper::Tree(tree), 0)?;

    remove_dead_code(&mut ir.ir);

    println!("IR:\nmain:");

    for i in &ir.ir {
        println!("{:?}", i);
    }

    for block in ir.blocks.keys() {
        println!("\n{:?}:", block);

        for i in ir.blocks.get(block).unwrap() {
            println!("{:?}", i);
        }
    }

    println!("\n");

    let (mut compiled, data_section_size, symbol_section_size) = compile_ir(ir, &mut HashMap::new(), &mut 0, &mut 0)?;
    
    for _ in 0..data_section_size {
        compiled.insert(0, 0);
    }

    let setup_size = SEGMENT_SETUP.split_whitespace().collect::<Vec<_>>().len(); 
    let setup = SEGMENT_SETUP.to_string()
                             .replace("$0", &format!("{}", setup_size))
                             .replace("$1", &format!("{}", data_section_size))
                             .replace("$2", &format!("{}", symbol_section_size))
                             .split_whitespace().map(|x| x.parse().unwrap()).collect::<Vec<_>>();
    
    for x in setup.iter().rev() {
        compiled.insert(0, *x);
    }

    compiled.extend_from_slice(CLEANUP);

    Ok(compiled)
}

pub fn compile_ir(ir: IrResult, vars: &mut HashMap<String, i32>,
                  var_addr: &mut i32, symbol_addr: &mut i32) -> Result<(Vec<i32>, usize, usize), BlocksError> {

    let mut result = Vec::new();

    for (key, value) in ir.blocks.iter() {
        vars.insert(key.clone(), *symbol_addr);

        let ir = IrResult {
            ir: value.clone(),
            blocks: HashMap::new(),
            address: Address::Static(-1),
            var_addr: Address::Static(-1),
            register: None,
            deref: false,
            math: false
        };

        let temp = compile_ir(ir, vars, var_addr, symbol_addr)?.0;

        result.extend_from_slice(&temp);
        *symbol_addr += temp.len() as i32;
    }

    for item in ir.ir {
        match item {
            Ir::Write(addr_a, data) => {
                let addr_a = get_var_or_new(addr_a, vars, var_addr);
                let data = get_static_addr(data);
                result.extend_from_slice(&[0, addr_a, data]);
            },
            Ir::Copy(addr_a, addr_b) => {
                let addr_a = get_var_or_new(addr_a, vars, var_addr);
                let addr_b = get_addr(addr_b, vars)?;
                result.extend_from_slice(&[1, addr_a, addr_b]);
            },
            Ir::IndirWrite(addr_a, data) => {
                let addr_a = get_var_or_new(addr_a, vars, var_addr);
                let data = get_static_addr(data);
                result.extend_from_slice(&[2, addr_a, data]);
            },
            Ir::IndirCopy(addr_a, addr_b) => {
                let addr_a = get_var_or_new(addr_a, vars, var_addr);
                let addr_b = get_addr(addr_b, vars)?;
                result.extend_from_slice(&[3, addr_a, addr_b]);
            },
            Ir::IndirCopy3(addr_a, addr_b) => {
                let addr_a = get_var_or_new(addr_a, vars, var_addr);
                let addr_b = get_addr(addr_b, vars)?;
                result.extend_from_slice(&[5, addr_a, addr_b]);
            },
            Ir::RegWrite(reg, data) => {
                let reg = reg as i32;
                let data = get_static_addr(data);
                result.extend_from_slice(&[10, reg, data]);
            },
            Ir::RegCopy(reg, addr) => {
                let reg = reg as i32;
                let addr = get_var_or_new(addr, vars, var_addr);
                result.extend_from_slice(&[11, reg, addr]);
            },
            Ir::RegMem(reg, addr) => {
                let reg = reg as i32;
                let addr = get_var_or_new(addr, vars, var_addr);
                result.extend_from_slice(&[13, addr, reg]);
            },
            Ir::Add => {
                result.extend_from_slice(&[16, 0, 1]);
            },
            Ir::Sub => {
                result.extend_from_slice(&[17, 0, 1]);
            },
            Ir::Mul => {
                result.extend_from_slice(&[18, 0, 1]);
            },
            Ir::Div => {
                result.extend_from_slice(&[19, 0, 1]);
            },
            Ir::Equals => {
                result.extend_from_slice(&[20, 0, 1]);
            },
            Ir::Less => {
                result.extend_from_slice(&[21, 0, 1]);
            },
            Ir::Greater => {
                result.extend_from_slice(&[22, 0, 1]);
            },
            Ir::LessEqual => {
                result.extend_from_slice(&[23, 0, 1]);
            },
            Ir::GreaterEqual => {
                result.extend_from_slice(&[24, 0, 1]);
            },
            Ir::Or => {
                result.extend_from_slice(&[25, 0, 1]);
            },
            Ir::And => {
                result.extend_from_slice(&[26, 0, 1]);
            },
            Ir::Not => {
                result.extend_from_slice(&[27, 0]);
            },
            Ir::Xor => {
                result.extend_from_slice(&[28, 0, 1]);
            },
            Ir::Branch(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[29, addr]);
            },
            Ir::CondBranch(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[30, addr]);
            },
            Ir::IndirBranch(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[32, addr]);
            },
            Ir::Call(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[33, addr]);
            },
            Ir::Return => {
                result.push(35);
            },
            Ir::Raw(raw) => {
                result.extend_from_slice(&raw);
            },
            Ir::Tag(name, value) => {
                match &name as &_ {
                    "var_addr" => *var_addr = if let Ok(v) = value.parse() {
                        v
                    } else {
                        return Err(BlocksError::new(ErrorKind::TagError,
                                                    Token::Other(format!("var_addr must be a number (found `{}`)", value))));
                    },
                    _ => return Err(BlocksError::new(ErrorKind::UnknownTag, Token::Other(name)))
                }
            }
        }
    }

    let data_section_size = vars.len() - ir.blocks.len();
    let symbol_section_size = *symbol_addr as usize;

    Ok((result, data_section_size, symbol_section_size))
}
