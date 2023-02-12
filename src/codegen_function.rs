use tracing::{warn};

use inkwell::values::{FunctionValue};

use z3::Solver;
use z3::ast::{Bool, Int, BV};

use crate::codegen_basic_block::codegen_basic_block;
use crate::control_flow_graph::{get_forward_edges, get_backward_edges, backward_topological_sort};
use crate::get_var_name::get_var_name;


pub fn codegen_function(function: &FunctionValue, solver: &Solver, namespace: &str) -> () {
    //! Perform backward symbolic execution on a function given the llvm-ir function object
    let forward_edges = get_forward_edges(&function);
    let backward_edges = get_backward_edges(&function);
    let backward_sorted_nodes = backward_topological_sort(&function);

    for node in backward_sorted_nodes {
        codegen_basic_block(node, &forward_edges, &backward_edges, function, solver, namespace);
    }

    // constrain int inputs
    for input in function.get_params() {
        // TODO: Support other input types
        if input.get_type().to_string().eq("\"i1\"") {
            continue;
        } else if input.get_type().to_string().eq("\"i32\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver, namespace));
            let min_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
            let max_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
            solver
                .assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        } else if input.get_type().to_string().eq("\"i64\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver, namespace));
            let min_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MIN.into(), 64), true);
            let max_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MAX.into(), 64), true);
            solver
                .assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        }  else {
            warn!("Currently unsuppported type {:?} for input parameter", input.get_type().to_string())
        }
    }
}