use token::Token;
use error::*;
use ir::Ir;
use utils::Address;

use std::collections::HashMap;

pub fn get_var_or_new(addr: Address, vars: &mut HashMap<String, i32>, var_addr: &mut i32) -> i32 {
    match addr {
        Address::Static(num) => num,
        Address::Variable(ident) => {
            let addr = vars.get(&*ident).map(|a| *a);
            match addr {
                Some(v) => v,
                None => {
                    vars.insert(ident, *var_addr);
                    *var_addr += 1;
                    *var_addr - 1
                }
            }
        }
    }
}

pub fn get_addr(addr: Address, vars: &HashMap<String, i32>) -> Result<i32, BlocksError> {
    match addr {
        Address::Static(num) => Ok(num),
        Address::Variable(ident) => vars.get(&ident)
                                        .map(|a| *a)
                                        .ok_or(BlocksError::new(ErrorKind::UndeclaredVar, Token::Other(ident)))
    }
}

pub fn get_static_addr(addr: Address) -> i32 {
    if let Address::Static(addr) = addr {
        addr
    } else {
        panic!("Address was not Static");
    }
}

pub fn get_code_size(ir: &[Ir]) -> usize {
    ir.iter().fold(0, |accum, x| {
        accum + match *x {
            Ir::Branch(_) | Ir::CondBranch(_) | Ir::IndirBranch(_) | Ir::Call(_) | Ir::Not => 2,
            Ir::Return => 1,
            _ => 3
        }
    })
}
