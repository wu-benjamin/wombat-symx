use tracing::{warn};

use inkwell::module::{Module as InkwellModule};
use inkwell::values::{FunctionValue};

use z3::Solver;

use crate::codegen::codegen_basic_block::codegen_basic_block;
use crate::control_flow_graph::{get_forward_edges, get_backward_edges, backward_topological_sort};
use crate::utils::pretty_print::pretty_print_function;


pub fn codegen_function(module: &InkwellModule, function: &FunctionValue, solver: &Solver, namespace: &str, call_stack: &str, return_target_node: &str, return_register: &str) -> () {
    //! Perform backward symbolic execution on a function given the llvm-ir function object
    
    let call_stack_vec = std::vec::Vec::from_iter(call_stack.split(","));
    if call_stack_vec.len() >= 2 {
        let current_call = *call_stack_vec.last().unwrap();
        for i in 0..call_stack_vec.len()-1 {
            if call_stack_vec.get(i).unwrap().eq(&current_call) {
                warn!("Recursive call to {} in call stack {:?} cannot be analyzed!", current_call, call_stack);
                return;
            }
        }
    }

    pretty_print_function(&function, namespace);

    let forward_edges = get_forward_edges(&function, namespace, return_target_node);
    let backward_edges = get_backward_edges(&function, namespace, return_target_node);
    let backward_sorted_nodes = backward_topological_sort(&function, namespace, return_target_node);

    for node in backward_sorted_nodes {
        codegen_basic_block(&module, node, &forward_edges, &backward_edges, function, solver, namespace, call_stack, return_register);
    }
}