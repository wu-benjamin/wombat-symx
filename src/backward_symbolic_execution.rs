use std::collections::{HashMap};

use inkwell::values::FunctionValue;
use inkwell::module::Module as InkwellModule;
use z3::ast::Bool;
use z3::Solver;

use crate::codegen::function_codegen;
use crate::pretty_print::pretty_print_function;

pub fn backward_symbolic_execution(function: &FunctionValue, _arg_names: &HashMap<String, String>, solver: &Solver, _module: &InkwellModule, _namespace: &String) -> bool {
    pretty_print_function(function);
    function_codegen(function, solver, _namespace);
    
    let start_node = function.get_first_basic_block().unwrap();
    let start_node_var_name = start_node.get_name().to_str().unwrap();
    let start_node_var = Bool::new_const(solver.get_context(), String::from(start_node_var_name));
    solver.assert(&start_node_var.not());
    return true;
}