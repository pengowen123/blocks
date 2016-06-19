use error::*;
use token::Token;
use tree::Tree;
use ir::*;

use std::intrinsics::discriminant_value;

// Add one to number of inputs or else bad things
const INPUT_COUNTS: [usize; 32] = [
    3, // Assign,
    0, // AssignSymbol,
    3, // Symbol,
    2, // Goto,
    2, // IfGoto,
    2, // Call,
    1, // Return,
    0, // Identifier(String),
    3, // Tag,
    2, // Dereference,
    2, // Address,
    3, // Equals,
    3, // Multiply,
    3, // Divide,
    3, // Add,
    3, // Subtract,
    2, // Not, 
    2, // Compare,
    3, // And,
    3, // Or,
    3, // Xor,
    3, // Greater,
    3, // Less,
    3, // GreaterEqual,
    3, // LessEqual,
    0, // OpenBrace,
    0, // CloseBrace,
    0, // LineEnd,
    2, // Raw,
    0, // Number
    0, // Register
    0  // Null,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenWrapper {
    Token(Token),
    Tree(Tree)
}

#[derive(Clone,Debug, PartialEq, Eq)]
pub enum Register {
    // These must be in the same order as mybytes ids so it is safe to cast it to i32
    Int1,
    Int2,
    Int3,
    Int4,
    Flag,
    Accum,
    Error,
    FlowSegment,
    DataSegment,
    PCounter
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Static(i32),
    Variable(String)
}

impl Address {
    pub fn new_var(ident: &str) -> Address {
        Address::Variable(ident.to_string())
    }

    pub fn new_temp(id: i32) -> Address {
        Address::Variable(format!("__temp_{}__", id))
    }
}

#[derive(Debug)]
pub struct Stack {
    pub data: Vec<TokenWrapper>
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            data: Vec::new()
        }
    }

    pub fn push(&mut self, item: TokenWrapper) {
        self.data.push(item);
    }

    pub fn pop(&mut self, count: usize) -> Option<Vec<TokenWrapper>> {
        let mut result = Vec::new();

        for _ in 0..count {
            result.push(if let Some(v) = self.data.pop() {
                v
            } else {
                return None;
            })
        }

        Some(result)
    }
}

pub fn get_input_count(operator: &Token) -> usize {
    unsafe {
        INPUT_COUNTS[discriminant_value(operator) as usize]
    }
}

pub fn is_element_token(elem: &Token, slice: &[Token]) -> bool {
    slice.iter().fold(false, |acc, e| {
        if let (&Token::Identifier(..), &Token::Identifier(..)) = (elem, e) {
            true
        } else {
            if elem == e { true } else { acc }
        }
    })
}

pub fn is_element<A: PartialEq<B>, B: Eq>(elem: &A, slice: &[B]) -> bool {
    slice.iter().fold(false, |acc, e| {
        if elem == e { true } else { acc }
    })
}

pub fn is_assigned_to(var: &str, ir: &[Ir]) -> bool {
    let addr = Address::Variable(var.to_string());

    ir.iter().fold(false, |acc, e| {
        match *e {
            Ir::Write(ref address, ref other) if *other != addr && *address == addr  => true,
            Ir::Copy(ref address, ref other) if *other != addr && *address == addr => true,
            Ir::IndirCopy3(ref address, ref other) if *other != addr && *address == addr => true,
            Ir::RegMem(_, ref address) if *address == addr => true,
            _ => acc
        }
    })
}

pub fn get_temp_id(id: i32) -> i32 {
    match id {
        -1 => 0,
        -2 => 1,
        _ => id
    }
}

pub fn register_store(tree: TokenWrapper, register: Register, result: &mut Vec<Ir>, temp_id: i32) -> Result<(), BlocksError> {
    let mut ir = build_ir(tree, temp_id)?;
    let ir_addr;

    if let Some(reg) = ir.register {
        ir_addr = Address::new_temp(temp_id);
        result.push(Ir::RegMem(reg,
                               ir_addr.clone()));
    } else {
        ir_addr = if is_assigned_to("__temp_1__", &ir.ir) {
            Address::new_temp(temp_id)
        } else {
            ir.address
        };
    }

    result.append(&mut ir.ir);
    result.push(Ir::RegCopy(register,
                            ir_addr));

    Ok(())
}

pub fn insert_operator(lhs: TokenWrapper, rhs: TokenWrapper,
                       result: &mut Vec<Ir>, operator: Ir, temp_id: i32) -> Result<(), BlocksError> {

    let has_subtree_lhs = has_subtree(&lhs);
    let has_subtree_rhs = has_subtree(&rhs);

    // makes sure the stores are in the right order to avoid overwriting
    match (has_subtree_lhs, has_subtree_rhs) {
        (_, false) => {
            register_store(lhs, Register::Int1, result, 1)?;
            register_store(rhs, Register::Int2, result, 1)?;
        },
        (false, true) => {
            register_store(rhs, Register::Int2, result, 1)?;
            register_store(lhs, Register::Int1, result, 1)?;
        },
        (true, true) => {
            register_store(lhs, Register::Int2, result, temp_id + 1)?;

            let last = if let Some(v) = result.pop() {
                v
            } else {
                return Err(BlocksError::new(ErrorKind::Other, Token::Other("this might not need to be an error".to_string())));
            };

            register_store(rhs, Register::Int1, result, temp_id + 2)?;

            result.push(last);
        }
    }

    result.push(operator);

    Ok(())
}

pub fn has_subtree(tree: &TokenWrapper) -> bool {
    if let &TokenWrapper::Tree(ref tree) = tree {
        match *tree {
            Tree::Less(_, _) | Tree::Greater(_, _) | Tree::LessEqual(_, _) | Tree::GreaterEqual(_, _) |
            Tree::Equals(_, _) | Tree::Add(_, _) | Tree::Subtract(_, _) | Tree::Multiply(_, _) |
            Tree::Divide(_, _) | Tree::Xor(_, _) | Tree::Not(_) => true,
            _ => false
        }
    } else {
        false
    }
}
