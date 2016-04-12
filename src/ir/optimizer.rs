use ir::*;

pub fn opt_0(ir: &mut Vec<Ir>) {
}

// Use this to remove ineffiencies made by the IR generator, such as writing to temp then copying
// it to a variable can be reduced to just writing it to the variable
pub fn remove_dead_code(ir: &mut Vec<Ir>) {
}
