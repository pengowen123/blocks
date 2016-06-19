// Stage 3 in compilation.
// Converts the recursive AST into a non-recursive, low level IR.
// This makes it easier to compile, while providing most of the error catching.
// Its simplicity makes it ideal for optimizations, so any optimizations are included in this stage
// Most of these optimizations are opt-in, because side effects may be relied on, and they would
// likely be removed in dead code elimination
// Only optimizations related to removing the inefficiencies produced by the IR generator are
// applied by default

use token::Token;
use tree::Tree;
use error::*;
use utils::*;

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ir {
    Write(Address, Address),
    Copy(Address, Address),
    IndirWrite(Address, Address),
    IndirCopy(Address, Address),
    IndirCopy3(Address, Address),
    RegWrite(Register, Address),
    RegCopy(Register, Address),
    RegMem(Register, Address),
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Or,
    And,
    Not,
    Xor,
    Branch(Address),
    CondBranch(Address),
    IndirBranch(Address),
    Call(Address),
    Tag(String, String),
    Return,
    Raw(Vec<i32>)
}

#[derive(Debug)]
pub struct IrResult {
    pub ir: Vec<Ir>,
    pub blocks: HashMap<String, Vec<Ir>>,
    pub address: Address,
    pub var_addr: Address,
    pub register: Option<Register>,
    pub deref: bool,
    pub math: bool
}

pub fn build_ir(tree: TokenWrapper, temp_id: i32) -> Result<IrResult, BlocksError> {
    let mut result = Vec::new();
    let mut blocks = HashMap::new();
    let mut address = Address::Static(-1);
    let mut var_addr = Address::Static(-1);
    let mut register = None;
    let mut deref = false;
    let mut math = false;

    match tree {
        TokenWrapper::Tree(Tree::Block(stmts)) => {
            for s in stmts {
                let mut ir = build_ir(s, 0)?;

                result.append(&mut ir.ir);
                
                for (key, value) in ir.blocks {
                    blocks.insert(key, value);
                }
            }
        },
        TokenWrapper::Tree(Tree::Assign(lhs, rhs)) => {
            let mut temp = Vec::new();

            let mut lhs = build_ir(*lhs, -1)?;
            let rhs = build_ir(*rhs, -2)?;

            temp.append(&mut lhs.ir);
            temp.extend_from_slice(&rhs.ir);

            if let Some(reg) = rhs.register {
                temp.push(Ir::RegMem(reg,
                                     Address::new_temp(1)));
            }

            if let Some(reg) = lhs.register {
                temp.push(Ir::RegCopy(reg,
                                      Address::new_temp(1)));
            } else {
                let lhs_addr = if lhs.address == Address::new_temp(0) {
                    lhs.address
                } else {
                    lhs.var_addr.clone()
                };
                let rhs_addr = if is_assigned_to("__temp_1__", &rhs.ir) {
                    Address::new_temp(1)
                } else {
                    rhs.address
                };

                if let Address::Static(i) = rhs_addr {
                    if i < 0 {
                        println!("foo");
                        return Err(BlocksError::new(ErrorKind::InvalidAddress, Token::Other(format!("{}", i))));
                    }
                } else if let Address::Static(i) = lhs_addr {
                    if i < 0 {
                        return Err(BlocksError::new(ErrorKind::InvalidAddress, Token::Other(format!("{}", i))));
                    }
                }

                if lhs.deref || lhs.math {
                    temp.push(Ir::IndirCopy(lhs_addr,
                                            rhs_addr));
                } else {
                    temp.push(Ir::Copy(lhs_addr,
                                       rhs_addr));
                }
            }

            result.append(&mut temp);
        },
        TokenWrapper::Tree(Tree::Dereference(item)) => {
            let mut temp = Vec::new();
            let item = build_ir(*item, temp_id)?;

            deref = true;
            temp.extend_from_slice(&item.ir);

            let id = match temp_id {
                    -1 => 0,
                    -2 => 1,
                    _ => temp_id
            };

            let addr = Address::new_temp(id);

            if let Some(reg) = item.register {
                temp.push(Ir::RegMem(reg,
                                     addr.clone()));
                temp.push(Ir::IndirCopy3(addr.clone(),
                                         addr.clone()));
            } else {
                deref = true;

                if item.deref {
                    temp.push(Ir::IndirCopy(addr.clone(),
                                            addr.clone()));
                } else {
                    temp.push(Ir::IndirCopy3(addr.clone(),
                                            item.address.clone()));
                }
            }

            var_addr = item.var_addr.clone();
            address = addr.clone();

            result.append(&mut temp);
        },
        TokenWrapper::Tree(Tree::Address(item)) => {
            let id = get_temp_id(temp_id);

            let ident = if let TokenWrapper::Token(Token::Identifier(ident)) = *item {
                ident
            } else {
                return Err(BlocksError::new(ErrorKind::AddressNameType, Token::Null));
            };
            
            address = Address::new_temp(id);

            result.push(Ir::Write(address.clone(),
                                  Address::Variable(ident)));
        },
        TokenWrapper::Tree(Tree::Tag(key, val)) => {
            result.push(Ir::Tag(key, val));
        },
        TokenWrapper::Tree(Tree::Symbol(name, block)) => {
            let ir = build_ir(*block, 0)?.ir;
            blocks.insert(name, ir);
        },
        TokenWrapper::Tree(Tree::Compare(operator)) => {
            result.append(&mut build_ir(*operator, 0)?.ir);
        },
        TokenWrapper::Tree(Tree::Less(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Less, get_temp_id(temp_id))?;
        },
        TokenWrapper::Tree(Tree::Greater(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Greater, get_temp_id(temp_id))?;
        },
        TokenWrapper::Tree(Tree::LessEqual(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::LessEqual, get_temp_id(temp_id))?;
        },
        TokenWrapper::Tree(Tree::GreaterEqual(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::GreaterEqual, get_temp_id(temp_id))?;
        },
        TokenWrapper::Tree(Tree::Equals(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Equals, get_temp_id(temp_id))?;
        },
        TokenWrapper::Tree(Tree::Add(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Add, get_temp_id(temp_id))?;

            let addr = Address::new_temp(get_temp_id(temp_id));

            result.push(Ir::RegMem(Register::Accum,
                                   addr.clone()));

            address = addr;
            math = true;
        },
        TokenWrapper::Tree(Tree::Subtract(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Sub, get_temp_id(temp_id))?;

            let addr = Address::new_temp(get_temp_id(temp_id));

            result.push(Ir::RegMem(Register::Accum,
                                   addr.clone()));

            address = addr;
            math = true;
        },
        TokenWrapper::Tree(Tree::Multiply(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Mul, get_temp_id(temp_id))?;

            let addr = Address::new_temp(get_temp_id(temp_id));

            result.push(Ir::RegMem(Register::Accum,
                                   addr.clone()));

            address = addr;
            math = true;
        },
        TokenWrapper::Tree(Tree::Divide(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Div, get_temp_id(temp_id))?;

            let addr = Address::new_temp(get_temp_id(temp_id));

            result.push(Ir::RegMem(Register::Accum,
                                   addr.clone()));

            address = addr;
            math = true;
        },
        TokenWrapper::Tree(Tree::Xor(lhs, rhs)) => {
            insert_operator(*lhs, *rhs, &mut result, Ir::Xor, get_temp_id(temp_id))?;

            let addr = Address::new_temp(get_temp_id(temp_id));

            result.push(Ir::RegMem(Register::Accum,
                                   addr.clone()));

            address = addr;
            math = true;
        },
        TokenWrapper::Tree(Tree::Not(item)) => {
            register_store(*item, Register::Int1, &mut result, get_temp_id(temp_id))?;
            result.push(Ir::Not);

            let addr = Address::new_temp(get_temp_id(temp_id));

            result.push(Ir::RegMem(Register::Accum,
                                   addr.clone()));

            address = addr;
            math = true;
        },
        TokenWrapper::Token(Token::Register(reg)) => {
            address = Address::new_temp(get_temp_id(temp_id));
            var_addr = address.clone();
            register = Some(reg);
        },
        TokenWrapper::Tree(Tree::Goto(item)) => {
            let mut ir = build_ir(*item.clone(), 0)?;
            result.append(&mut ir.ir);

            let addr = if let TokenWrapper::Token(Token::Identifier(ident)) = *item {
                Address::Variable(ident)
            } else {
                ir.address
            };

            if ir.deref || ir.math {
                result.push(Ir::IndirBranch(addr));
            } else {
                result.push(Ir::Branch(addr));
            }
        },
        TokenWrapper::Tree(Tree::IfGoto(item)) => {
            let addr = match *item {
                TokenWrapper::Token(Token::Identifier(ident)) => Address::Variable(ident),
                TokenWrapper::Token(Token::Number(num)) => Address::Static(num),
                _ => return Err(BlocksError::new(ErrorKind::IfGotoAddressType, Token::Null))
            };

            result.push(Ir::CondBranch(addr));
        },
        TokenWrapper::Tree(Tree::Call(item)) => {
            let addr = match *item {
                TokenWrapper::Token(Token::Identifier(ident)) => Address::Variable(ident),
                TokenWrapper::Token(Token::Number(num)) => Address::Static(num),
                _ => return Err(BlocksError::new(ErrorKind::IfGotoAddressType, Token::Null))
            };

            result.push(Ir::Call(addr));
        },
        TokenWrapper::Tree(Tree::Return) => {
            result.push(Ir::Return);
        },
        TokenWrapper::Tree(Tree::Raw(raw)) => {
            result.push(Ir::Raw(raw));
        },
        TokenWrapper::Token(Token::Identifier(ident)) => {
            if address == Address::Static(-1) {
                address = Address::Variable(ident.clone());
            }

            if var_addr == Address::Static(-1) {
                var_addr = Address::Variable(ident);
            }
        },
        TokenWrapper::Token(Token::Number(num)) => {
            if address == Address::Static(-1) {
                address = Address::Static(num);
            }

            if var_addr == Address::Static(-1) {
                var_addr = Address::Static(num);
            }

            let id = get_temp_id(temp_id);

            result.push(Ir::Write(Address::new_temp(id),
                                  Address::Static(num)));
        },
        _ => {
            return Err(BlocksError::new(ErrorKind::Other, Token::Other("this shouldn't happen".to_string())));
        }
    }

    Ok(IrResult {
        ir: result,
        blocks: blocks,
        address: address,
        var_addr: var_addr,
        register: register,
        deref: deref,
        math: math
    })
}
