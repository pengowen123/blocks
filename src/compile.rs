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

pub fn compile(prog: &str) -> Result<Vec<i32>, BlocksError> {
    let tree = build_token_tree(prog.to_string())?;
    let mut ir = build_ir(TokenWrapper::Tree(tree), 0)?;

    remove_dead_code(&mut ir.ir);

    println!("IR:\nmain:");

    for i in &ir.ir {
        println!("{:?}", i);
    }

    for block in ir.blocks.keys() {
        println!("{:?}:", block);

        for i in ir.blocks.get(block).unwrap() {
            println!("{:?}", i);
        }
    }

    println!("\n");

    compile_ir(ir, &mut HashMap::new(),
               &mut 0, &mut 0)
}

pub fn compile_ir(ir: IrResult, vars: &mut HashMap<String, i32>,
                  var_addr: &mut i32, symbol_addr: &mut i32) -> Result<Vec<i32>, BlocksError> {

    let mut result = Vec::new();

    //println!("VARS a: {:?}", vars);

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

        let temp = compile_ir(ir, vars, var_addr, symbol_addr)?;

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
                result.extend_from_slice(&[6, reg, data]);
            },
            Ir::RegCopy(reg, addr) => {
                let reg = reg as i32;
                let addr = get_var_or_new(addr, vars, var_addr);
                result.extend_from_slice(&[7, reg, addr]);
            },
            Ir::RegMem(reg, addr) => {
                let reg = reg as i32;
                let addr = get_var_or_new(addr, vars, var_addr);
                result.extend_from_slice(&[8, addr, reg]);
            },
            Ir::Add => {
                result.extend_from_slice(&[9, 0, 1]);
            },
            Ir::Sub => {
                result.extend_from_slice(&[10, 0, 1]);
            },
            Ir::Mul => {
                result.extend_from_slice(&[11, 0, 1]);
            },
            Ir::Div => {
                result.extend_from_slice(&[12, 0, 1]);
            },
            Ir::Equals => {
                result.extend_from_slice(&[13, 0, 1]);
            },
            Ir::Less => {
                result.extend_from_slice(&[14, 0, 1]);
            },
            Ir::Greater => {
                result.extend_from_slice(&[15, 0, 1]);
            },
            Ir::LessEqual => {
                result.extend_from_slice(&[16, 0, 1]);
            },
            Ir::GreaterEqual => {
                result.extend_from_slice(&[17, 0, 1]);
            },
            Ir::Or => {
                result.extend_from_slice(&[18, 0, 1]);
            },
            Ir::And => {
                result.extend_from_slice(&[19, 0, 1]);
            },
            Ir::Not => {
                result.extend_from_slice(&[20, 0]);
            },
            Ir::Xor => {
                result.extend_from_slice(&[21, 0, 1]);
            },
            Ir::Branch(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[22, addr]);
            },
            Ir::CondBranch(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[23, addr]);
            },
            Ir::IndirBranch(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[24, addr]);
            },
            Ir::Call(addr) => {
                let addr = get_addr(addr, vars)?;
                result.extend_from_slice(&[25, addr]);
            },
            Ir::Return => {
                result.push(26);
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

    println!("vars: {:?}", vars);

    let mut data_section = vec![0; vars.len()];

    // A decision needs to be made regarding the behavior of symbol and variable placement.
    // Problems to solve:
    //   - How should the variable placement deal with the use the var_addr tag?
    //   - How agnostic should the generated code be to the location of the program?
    //   - Similarly, how can the program call symbols if they are moved along with the program (or
    //     should they be unmovable?
    // 
    // Potential solutions:
    //   Dynamic addresses which allows full independency from program location:
    //   - Allow symbols to be moved with the program by setting the segment register to the
    //     program, and reset it to 0 for symbols tagged with sym_addr
    //   - Use a similar system for variables, using dynamic calculation of addresses, while
    //     retaining static addresses for variables tagged with var_addr
    //   Issues:
    //   - Requires an address configurable through tags for storing the program location
    //   - Has a potentially large performance penalty
    //   - Complexifies the generated code, but has a much larger effect in compiler complexity
    
    //println!("variable section size: {:?}", data_section.len());
    //println!("symbol section size: {:?}", symbol_section_size); Ok(result)
}
