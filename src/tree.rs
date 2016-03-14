use token::*;
use error::BlocksError;
use error::ErrorKind::*;

const DefaultExpected: [Token; 10] = [Token::Null, Token::Assign,
                         Token::Raw, Token::Symbol,
                         Token::Goto, Token::IfGoto,
                         Token::Call, Token::Return,
                         Token::Tag, Token::Compare];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tree {
    Block(Vec<Tree>),
    Assign(Box<Tree>, Box<Tree>),
    Identifier(String),
    Dereference(Box<Tree>),
    Address(Box<Tree>),
    Null
}

pub fn build_token_tree<'a>(prog: String) -> Result<Tree, BlocksError> {
    let mut tree = Tree::Block(Vec::new());

    let mut previous_token = Token::Null;
    let mut expected = DefaultExpected.to_vec();

    let mut node = Vec::new();

    let tokens = build_tokens(prog);

    for token in tokens {
        //Token::Dereference => {
            //if let Some(&mut Tree::Assign(ref mut i, ref mut x)) = node.last_mut() {
                //if **i == Tree::Null {
                    //*i = Box::new(Tree::Identifier(ident));

                    //expected = vec![Token::AssignSymbol];
                //} else {
                    //*x = Box::new(Tree::Identifier(ident));

                    //expected = vec![Token::LineEnd];
                //}
            //}
        //},
        println!("{:?}", token);
        println!("{:?}", expected);
        println!("{:?}", node);
        let mut node_end = false;

        if token != Token::Null && !is_element_token(&token, &expected) {
            return Err(BlocksError::new(UnexpectedToken, (token, &expected)));
        }

        match token.clone() {
            Token::Assign => {
                expected = vec![Token::Dereference, Token::Address,
                               Token::Identifier(String::new())];
                node.push(Tree::Assign(Box::new(Tree::Null), Box::new(Tree::Null)));
            },
            Token::Identifier(ident) => {
                match previous_token {
                    Token::Symbol => {
                        expected = vec![Token::OpenBrace];
                    },
                    Token::Assign => {
                        if let Some(&mut Tree::Assign(ref mut i, _)) = node.last_mut() {
                            *i = Box::new(Tree::Identifier(ident));
                        }

                        expected = vec![Token::AssignSymbol];
                    },
                    Token::AssignSymbol => {
                        if let Some(&mut Tree::Assign(_, ref mut i)) = node.last_mut() {
                            *i = Box::new(Tree::Identifier(ident));
                        } else {
                            return Err(BlocksError::new(UnexpectedToken, (token, &[Token::Null])));
                        }

                        expected = vec![Token::LineEnd];
                    },
                    Token::Dereference => {
                        if let Some(tree) = node.last_mut() {
                            if let Tree::Assign(ref mut i, ref mut x) = *tree {
                                if **i == Tree::Null {
                                    *i = Box::new(Tree::Identifier(ident));

                                    expected = vec![Token::AssignSymbol];
                                } else {
                                    *x = Box::new(Tree::Identifier(ident));

                                    expected = vec![Token::LineEnd];
                                }
                            } else if let Tree::Dereference(..) = *tree {
                                expected = vec![Token::LineEnd];
                            }

                            *tree = Tree::Dereference(Box::new(tree.clone()));
                        }
                    },
                    Token::Address => {
                        if let Some(tree) = node.last_mut() {
                            if let Tree::Assign(ref mut i, ref mut x) = *tree {
                                if **i == Tree::Null {
                                    *i = Box::new(Tree::Identifier(ident));

                                    expected = vec![Token::AssignSymbol];
                                } else {
                                    *x = Box::new(Tree::Identifier(ident));

                                    expected = vec![Token::LineEnd];
                                }
                            }

                            *tree = Tree::Address(Box::new(tree.clone()));
                        }
                    },
                    _ => {
                        return Err(BlocksError::new(UnexpectedToken, (token, &[Token::Null])));
                    }
                }
            },
            Token::AssignSymbol => {
                expected = vec![Token::Dereference, Token::Address,
                                Token::Identifier(String::new())];
            },
            Token::LineEnd => {
                expected = DefaultExpected.to_vec();
                node_end = true;
            },
            Token::Dereference => {
                expected = vec![Token::Dereference, Token::Address,
                                Token::Identifier(String::new())];
            },
            Token::Address => {
                expected = vec![Token::Dereference, Token::Address,
                                Token::Identifier(String::new())];
            },
            _ => {}
        }

        if node_end {
            if let Tree::Block(ref mut block) = tree {
                block.push(if let Some(n) = node.pop() {
                    n
                } else {
                    return Err(BlocksError::new(UnexpectedToken, (token, &[Token::Null])));
                })
            }
        }
        
        previous_token = token;
    }

    Ok(tree)
}
