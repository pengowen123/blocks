#[cfg(test)]
mod tests {
    use ir::*;
    use utils::*;
    use tree::build_token_tree;

    #[test]
    fn test_write() {
        let prog = "
            set 0 = 1;
        ";
        
        let tree = build_token_tree(prog.to_string()).unwrap();

        let mut ir = build_ir(TokenWrapper::Tree(tree), 0).unwrap().ir;

        remove_dead_temp(&mut ir);

        let expected = [
            Ir::Write(Address::Variable("__temp_0__".to_string()), Address::Static(0)),
            Ir::Write(Address::Variable("__temp_1__".to_string()), Address::Static(1)),
            Ir::Copy(Address::Static(0), Address::Variable("__temp_1__".to_string())),
        ];

        assert_eq!(&expected, &ir as &[_]);
    }
}
