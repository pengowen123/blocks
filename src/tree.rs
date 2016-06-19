// Stage 2 in compilation.
// Organizes tokens into a tree format.
// Catches some errors in operator and keyword use but most errors are caught by the IR generator.

use utils::*;
use token::*;
use error::BlocksError;
use error::ErrorKind::*;

const IGNORED_TOKENS: [Token; 3] = [
    Token::AssignSymbol,
    Token::LineEnd,
    Token::Null
];

pub type Boxed = Box<TokenWrapper>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tree {
    Block(Vec<TokenWrapper>),
    Assign(Boxed, Boxed),
    Dereference(Boxed),
    Goto(Boxed),
    IfGoto(Boxed),
    Call(Boxed),
    Return,
    Multiply(Boxed, Boxed),
    Divide(Boxed, Boxed),
    Add(Boxed, Boxed),
    Subtract(Boxed, Boxed),
    Address(Boxed),
    Compare(Boxed),
    Greater(Boxed, Boxed),
    Less(Boxed, Boxed),
    GreaterEqual(Boxed, Boxed),
    LessEqual(Boxed, Boxed),
    Equals(Boxed, Boxed),
    Not(Boxed),
    And(Boxed, Boxed),
    Or(Boxed, Boxed),
    Xor(Boxed, Boxed),
    Symbol(String, Boxed),
    Tag(String, String),
    Raw(Vec<i32>)
}

pub fn build_token_tree<'a>(prog: String) -> Result<Tree, BlocksError> {
    let mut tree = Vec::new();
    let mut stack = Stack::new();
    let tokens = build_tokens(prog);

    let mut block_data = Stack::new();
    let mut block = false;

    for token in tokens.iter().rev() {
        if is_element_token(token, &IGNORED_TOKENS) {
            continue;
        }

        if block {
            block_data.push(TokenWrapper::Token(token.clone()));
        } else {
            stack.push(TokenWrapper::Token(token.clone()));
        }

        let inputs = get_input_count(token);
        let mut node_data;

        if block {
            node_data = if let Some(v) = block_data.pop(inputs) {
                v
            } else {
                return Err(BlocksError::new(NotEnoughArgs, token.clone()));
            }
        } else {
            node_data = if let Some(v) = stack.pop(inputs) {
                v
            } else {
                return Err(BlocksError::new(NotEnoughArgs, token.clone()));
            };
        }

        if node_data.is_empty() {
            match token {
                o @ &Token::OpenBrace => node_data = vec![TokenWrapper::Token(o.clone())],
                c @ &Token::CloseBrace => node_data = vec![TokenWrapper::Token(c.clone())],
                _ => continue
            }
        }

        match node_data[0] {
            TokenWrapper::Token(Token::Assign) => {
                let new = TokenWrapper::Tree(
                    Tree::Assign(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );

                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Dereference) => {
                let new = TokenWrapper::Tree(Tree::Dereference(Box::new(node_data[1].clone())));
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Address) => {
                let new = TokenWrapper::Tree(Tree::Address(Box::new(node_data[1].clone())));
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Goto) => {
                let new = TokenWrapper::Tree(Tree::Goto(Box::new(node_data[1].clone())));
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::IfGoto) => {
                let new = TokenWrapper::Tree(Tree::IfGoto(Box::new(node_data[1].clone())));
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Call) => {
                let new = TokenWrapper::Tree(Tree::Call(Box::new(node_data[1].clone())));
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Return) => {
                let new = TokenWrapper::Tree(Tree::Return);
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Multiply) => {
                let new = TokenWrapper::Tree(
                    Tree::Multiply(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Divide) => {
                let new = TokenWrapper::Tree(
                    Tree::Divide(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Add) => {
                let new = TokenWrapper::Tree(
                    Tree::Add(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Subtract) => {
                let new = TokenWrapper::Tree(
                    Tree::Subtract(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Compare) => {
                let new = TokenWrapper::Tree(Tree::Compare(Box::new(node_data[1].clone())));

                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Greater) => {
                let new = TokenWrapper::Tree(
                    Tree::Greater(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Equals) => {
                let new = TokenWrapper::Tree(
                    Tree::Equals(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Less) => {
                let new = TokenWrapper::Tree(
                    Tree::Less(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::GreaterEqual) => {
                let new = TokenWrapper::Tree(
                    Tree::GreaterEqual(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::LessEqual) => {
                let new = TokenWrapper::Tree(
                    Tree::LessEqual(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Not) => {
                let new = TokenWrapper::Tree(Tree::Not(Box::new(node_data[1].clone())));
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::And) => {
                let new = TokenWrapper::Tree(
                    Tree::And(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Or) => {
                let new = TokenWrapper::Tree(
                    Tree::Or(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Xor) => {
                let new = TokenWrapper::Tree(
                    Tree::Xor(
                        Box::new(node_data[1].clone()),
                        Box::new(node_data[2].clone())
                    )
                );

                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Raw) => {
                let temp = if let TokenWrapper::Token(Token::Identifier(string)) = node_data[1].clone() {
                    string.split_whitespace().map(|i| match i.parse::<i32>() {
                        Ok(v) => Ok(v),
                        Err(..) => Err(BlocksError::new(InvalidRaw, Token::Identifier(string.clone())))
                    }).collect::<Vec<Result<_, _>>>()
                } else {
                    return Err(BlocksError::new(InvalidRaw, Token::Null))
                };

                let mut raw = Vec::new();

                for x in &temp {
                    match x {
                        &Ok(v) => raw.push(v),
                        &Err(ref e) => return Err(e.clone())
                    }
                }

                let new = TokenWrapper::Tree(
                    Tree::Raw(
                        raw
                    )
                );

                if block { block_data.push(new) } else { stack.push(new) }
            }
            TokenWrapper::Token(Token::Tag) => {
                let a = if let TokenWrapper::Token(ref a) = node_data[1].clone() {
                    a.clone()
                } else {
                    return Err(BlocksError::new(TagNameType, Token::Null));
                };

                let b = if let TokenWrapper::Token(ref b) = node_data[2].clone() {
                    b.clone()
                } else {
                    return Err(BlocksError::new(TagValueType, Token::Null));
                };

                let name = match a {
                    Token::Identifier(ident) => ident,
                    Token::Number(num) => format!("{}", num),
                    _ => return Err(BlocksError::new(TagNameType, a))
                };

                let value = match b {
                    Token::Identifier(ident) => ident,
                    Token::Number(num) => format!("{}", num),
                    _ => return Err(BlocksError::new(TagNameType, b))
                };

                let new = TokenWrapper::Tree(Tree::Tag(name, value));

                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::Symbol) => {
                let token = if let TokenWrapper::Token(ref t) = node_data[1] {
                    t.clone()
                } else {
                    return Err(BlocksError::new(SymbolNameType, Token::Null));
                };

                let name = if let Token::Identifier(ident) = token {
                    ident
                } else {
                    return Err(BlocksError::new(SymbolNameType, token));
                };

                let new = TokenWrapper::Tree(Tree::Symbol(name, Box::new(node_data[2].clone())));
                
                if block { block_data.push(new) } else { stack.push(new) }
            },
            TokenWrapper::Token(Token::OpenBrace) => {
                block_data.pop(1);
                stack.push(TokenWrapper::Tree(Tree::Block(block_data.data.iter().cloned().rev().collect())));
                block_data = Stack::new();

                if !block {
                    return Err(BlocksError::new(UnexpectedToken, token.clone()));
                }

                block = false;
            },
            TokenWrapper::Token(Token::CloseBrace) => {
                if block {
                    return Err(BlocksError::new(UnexpectedToken, token.clone()));
                }

                stack.pop(1);
                block = true;
            },
            _ => {
                return Err(BlocksError::new(Other, token.clone()));
            }
        }
    }

    for x in stack.data {
        tree.insert(0, x);
    }

    Ok(Tree::Block(tree))
}
