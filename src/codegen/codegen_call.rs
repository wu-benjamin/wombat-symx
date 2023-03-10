// Copyright (c) 2023 Benjamin Jialong Wu
// This code is licensed under MIT license (see LICENSE.md for details)

use tracing::{warn};

use inkwell::module::Module as InkwellModule;
use inkwell::values::InstructionValue;

use z3::ast::{Ast, Bool, Int};
use z3::Solver;

use crate::codegen::codegen_function::codegen_function;
use crate::control_flow_graph::forward_topological_sort;
use crate::symbolic_execution::get_module_name_from_file_name;
use crate::utils::function_utils::{get_function_by_name, get_function_name};
use crate::utils::var_utils::{get_min_max_signed_int, get_var_name};

fn codegen_general_call<'a>(module: &InkwellModule, mut node_var: Bool<'a>, instruction: InstructionValue, solver: &'a Solver, namespace: &str, call_stack: &str) -> Bool<'a> {
    // Create named POST_NODE from node_var
    let new_return_register_string = get_var_name(&instruction, solver, namespace);
    let new_return_register_str = new_return_register_string.as_str();
    let post_node_name_string = format!("{}_{}", namespace, new_return_register_str);
    let post_node_name_str = post_node_name_string.as_str();
    let post_node = Bool::new_const(solver.get_context(), post_node_name_str);
    solver.assert(&post_node._eq(&node_var));
    let new_namespace = format!("{}.", post_node);

    // Code gen function with return to POST_NODE and request to assign return value to new return register
    let call_operand = instruction.get_operand(instruction.get_num_operands() - 1).unwrap().left().unwrap().into_pointer_value();
    let call_operation_name_string = get_function_name(&call_operand);
    let function = get_function_by_name(module, &call_operation_name_string).unwrap();
    let new_return_register_string = get_var_name(&instruction, solver, namespace);
    let new_return_register_str = new_return_register_string.as_str();
    let new_call_stack_string = format!("{},{}", call_stack, function.get_name().to_str().unwrap());
    codegen_function(
        module,
        &function,
        solver,
        new_namespace.as_str(),
        new_call_stack_string.as_str(),
        post_node_name_str,
        new_return_register_str,
    );

    // CALL_NODE: Start node of function
    let called_function_forward_sorted_nodes = forward_topological_sort(&function, &new_namespace, post_node_name_str);
    if !called_function_forward_sorted_nodes.is_empty() {
        let call_node_name = called_function_forward_sorted_nodes.first().unwrap();
        node_var = Bool::new_const(solver.get_context(), call_node_name.as_str());
    } else {
        // NO-OP
        node_var = post_node;
    }

    // PRE_NODE with CALL_NODE as successor: Assign call arguments
    // Supports signed int types and booleans
    assert!(u32::try_from(function.get_params().len()).unwrap() == instruction.get_num_operands() - 1);
    for i in 0..function.get_params().len() {
        let params = function.get_params();
        let input = params.get(i).unwrap();
        if input.get_type().to_string().eq("\"i1\"") {
            let lvalue = Bool::new_const(solver.get_context(), get_var_name(input, solver, &new_namespace));
            let rvalue = Bool::new_const(
                solver.get_context(),
                get_var_name(&instruction.get_operand(u32::try_from(i).unwrap()).unwrap().left().unwrap(), solver, namespace),
            );
            let assignment = lvalue._eq(&rvalue);
            node_var = assignment.implies(&node_var);
        } else if input.get_type().is_int_type() {
            let lvalue = Int::new_const(solver.get_context(), get_var_name(input, solver, &new_namespace));
            let rvalue = Int::new_const(
                solver.get_context(),
                get_var_name(&instruction.get_operand(u32::try_from(i).unwrap()).unwrap().left().unwrap(), solver, namespace),
            );
            let assignment = lvalue._eq(&rvalue);
            node_var = assignment.implies(&node_var);
        } else {
            warn!("Currently unsupported type {:?} for input parameter to {}", input.get_type().to_string(), call_operation_name_string);
        }
    }

    // Return PRE_NODE
    node_var
}

pub fn codegen_call<'a>(module: &InkwellModule, mut node_var: Bool<'a>, instruction: InstructionValue, solver: &'a Solver, namespace: &str, call_stack: &str) -> Bool<'a> {
    let call_operand = instruction.get_operand(instruction.get_num_operands() - 1).unwrap().left().unwrap().into_pointer_value();
    let call_operation_name_string = get_function_name(&call_operand);
    let call_operation_name_str = call_operation_name_string.as_str();

    let module_name = get_module_name_from_file_name(&String::from(module.get_name().to_str().unwrap()));
    if call_operation_name_str.contains(&module_name) {
        return codegen_general_call(module, node_var, instruction, solver, namespace, call_stack);
    }

    match call_operation_name_str {
        s if s.starts_with("llvm.sadd.with.overflow.i") => {
            // Translate the intrinsic integer size to an i64 representing the min/max representable numbers
            let s_size = &s[25..];
            let (min_int_val, max_int_val) = get_min_max_signed_int(s_size);

            let operand1_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), solver, namespace);
            let operand2_name = get_var_name(&instruction.get_operand(1).unwrap().left().unwrap(), solver, namespace);

            let lvalue_var_name_1 = format!("{}.0", get_var_name(&instruction, solver, namespace));
            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
            let rvalue_var_1 = Int::add(
                solver.get_context(),
                &[&Int::new_const(solver.get_context(), operand1_name), &Int::new_const(solver.get_context(), operand2_name)],
            );

            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

            let lvalue_var_name_2 = format!("{}.1", get_var_name(&instruction, solver, namespace));
            let min_int = Int::from_i64(solver.get_context(), min_int_val);
            let max_int = Int::from_i64(solver.get_context(), max_int_val);
            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);

            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
            node_var = assignment.implies(&node_var);
        }
        s if s.starts_with("llvm.ssub.with.overflow.i") => {
            // Translate the intrinsic integer size to an i64 representing the min/max representable numbers
            let s_size = &s[25..];
            let (min_int_val, max_int_val) = get_min_max_signed_int(s_size);

            let operand1_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), solver, namespace);
            let operand2_name = get_var_name(&instruction.get_operand(1).unwrap().left().unwrap(), solver, namespace);

            let lvalue_var_name_1 = format!("{}.0", get_var_name(&instruction, solver, namespace));
            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
            let rvalue_var_1 = Int::sub(
                solver.get_context(),
                &[&Int::new_const(solver.get_context(), operand1_name), &Int::new_const(solver.get_context(), operand2_name)],
            );

            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

            let lvalue_var_name_2 = format!("{}.1", get_var_name(&instruction, solver, namespace));
            let min_int = Int::from_i64(solver.get_context(), min_int_val);
            let max_int = Int::from_i64(solver.get_context(), max_int_val);
            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);

            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
            node_var = assignment.implies(&node_var);
        }
        s if s.starts_with("llvm.smul.with.overflow.i") => {
            // Translate the intrinsic integer size to an i64 representing the min/max representable numbers
            let s_size = &s[25..];
            let (min_int_val, max_int_val) = get_min_max_signed_int(s_size);

            let operand1_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), solver, namespace);
            let operand2_name = get_var_name(&instruction.get_operand(1).unwrap().left().unwrap(), solver, namespace);

            let lvalue_var_name_1 = format!("{}.0", get_var_name(&instruction, solver, namespace));
            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
            let rvalue_var_1 = Int::mul(
                solver.get_context(),
                &[&Int::new_const(solver.get_context(), operand1_name), &Int::new_const(solver.get_context(), operand2_name)],
            );

            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

            let lvalue_var_name_2 = format!("{}.1", get_var_name(&instruction, solver, namespace));
            let min_int = Int::from_i64(solver.get_context(), min_int_val);
            let max_int = Int::from_i64(solver.get_context(), max_int_val);
            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);

            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
            node_var = assignment.implies(&node_var);
        }
        "llvm.expect.i1" => {
            let lvalue_var_name = get_var_name(&instruction, solver, namespace);
            let operand1_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), solver, namespace);
            let operand2_name = get_var_name(&instruction.get_operand(1).unwrap().left().unwrap(), solver, namespace);
            let rvalue_var = Bool::new_const(solver.get_context(), operand1_name)._eq(&Bool::new_const(solver.get_context(), operand2_name));
            let assignment = Bool::new_const(solver.get_context(), lvalue_var_name)._eq(&rvalue_var);
            node_var = assignment.implies(&node_var);
        }
        s if s.starts_with("core::panicking::panic") => {
            // NO-OP
        }
        _ => {
            warn!("Unsupported Call function {:?}", call_operation_name_str);
        }
    }
    node_var
}
