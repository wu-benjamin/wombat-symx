use std::collections::{HashMap, HashSet};

use inkwell::{IntPredicate};
use inkwell::basic_block::BasicBlock;
use inkwell::values::{FunctionValue, InstructionOpcode, InstructionValue, PhiValue};
use tracing::{warn};
use z3::ast::{Ast, Bool, Int, BV};
use z3::Solver;

use crate::symbolic_execution::get_function_name;
use crate::get_var_name::get_var_name;


#[derive(Debug)]
#[derive(PartialEq)]
enum IsCleanup {
    YES,
    NO,
    UNKNOWN,
}

fn get_basic_block_by_name<'a>(function: &'a FunctionValue, name: &String) -> BasicBlock<'a> {
    let mut matching_bb: Option<BasicBlock> = None;
    let mut matched = false;
    for bb in function.get_basic_blocks() {
        if name.eq(&String::from(bb.get_name().to_str().unwrap())) {
            if matched {
                warn!("Multiple basic blocks matched name {:?}", name);
            }
            matching_bb = Some(bb);
            matched = true;
        }
    }
    return matching_bb.unwrap();
}


fn is_panic_block(bb: &BasicBlock) -> IsCleanup {
    if let Some(terminator) = bb.get_terminator() {
        let opcode = terminator.get_opcode();
        match &opcode {
            InstructionOpcode::Return => {
                return IsCleanup::NO;
            }
            InstructionOpcode::Br => {
                return IsCleanup::NO;
            }
            InstructionOpcode::Switch => {
                return IsCleanup::NO;
            }
            InstructionOpcode::IndirectBr => {
                return IsCleanup::UNKNOWN;
            }
            InstructionOpcode::Invoke => {
                return IsCleanup::UNKNOWN;
            }
            InstructionOpcode::CallBr => {
                return IsCleanup::UNKNOWN;
            }
            InstructionOpcode::Resume => {
                return IsCleanup::UNKNOWN;
            }
            InstructionOpcode::CatchSwitch => {
                return IsCleanup::UNKNOWN;
            }
            InstructionOpcode::CatchRet => {
                return IsCleanup::UNKNOWN;
            }
            InstructionOpcode::CleanupRet => {
                return IsCleanup::UNKNOWN;
            }
            InstructionOpcode::Unreachable => {
                return IsCleanup::YES;
            }
            _ => {
                warn!("Opcode {:?} is not supported as a terminator for panic detection", opcode);
                return IsCleanup::UNKNOWN;
            }
        }
    } else {
        warn!("\tNo terminator found for panic detection");
        return IsCleanup::UNKNOWN;
    }
}


fn get_forward_edges(function: &FunctionValue) -> HashMap<String, HashSet<String>> {
    let mut all_edges = HashMap::new();
    for bb in function.get_basic_blocks() {
        let mut node_edges = HashSet::new();
        let basic_block_name = String::from(bb.get_name().to_str().unwrap());
        if let Some(terminator) = bb.get_terminator() {
            let opcode = terminator.get_opcode();
            let num_operands = terminator.get_num_operands();
            match &opcode {
                InstructionOpcode::Return => {
                    // NO-OP
                }
                InstructionOpcode::Br => {
                    if num_operands == 1 {
                        // Unconditional branch
                        let successor_basic_block = terminator.get_operand(0).unwrap().right().unwrap();
                        let successor_basic_block_name = String::from(successor_basic_block.get_name().to_str().unwrap());
                        node_edges.insert(String::from(successor_basic_block_name));
                    } else if num_operands == 3 {
                        // Conditional branch
                        let successor_basic_block_1 = terminator.get_operand(1).unwrap().right().unwrap();
                        let successor_basic_block_name_1 = String::from(successor_basic_block_1.get_name().to_str().unwrap());
                        node_edges.insert(String::from(successor_basic_block_name_1));
                        let successor_basic_block_2 = terminator.get_operand(2).unwrap().right().unwrap();
                        let successor_basic_block_name_2 = String::from(successor_basic_block_2.get_name().to_str().unwrap());
                        node_edges.insert(String::from(successor_basic_block_name_2));
                    } else {
                        warn!("Incorrect number of operators {:?} for opcode {:?} for edge generations", num_operands, opcode);
                    }
                }
                InstructionOpcode::Switch => {
                    for operand in 0..num_operands {
                        if operand % 2 == 1 {
                            let successor_basic_block = terminator.get_operand(operand).unwrap().right().unwrap();
                            let successor_basic_block_name = String::from(successor_basic_block.get_name().to_str().unwrap());
                            node_edges.insert(String::from(successor_basic_block_name));
                        }
                    }
                }
                InstructionOpcode::IndirectBr => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Invoke => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CallBr => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Resume => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchSwitch => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchRet => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CleanupRet => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Unreachable => {
                    // NO-OP
                }
                _ => {
                    warn!("Opcode {:?} is not supported as a terminator for edge generation", opcode);
                }
            }
        } else {
            warn!("\tNo terminator");
        }
        all_edges.insert(basic_block_name, node_edges);
    }
    return all_edges;
}


fn get_backward_edges(function: &FunctionValue) -> HashMap<String, HashSet<String>> {
    let all_forward_edges = get_forward_edges(function);
    let mut all_backward_edges = HashMap::new();
    for bb in function.get_basic_blocks() {
        let basic_block_name = String::from(bb.get_name().to_str().unwrap());
        all_backward_edges.insert(basic_block_name, HashSet::new());
    }
    for (source, dests) in all_forward_edges {
        for dest in dests {
            if let Some(reverse_dests) = all_backward_edges.get_mut(&dest) {
                reverse_dests.insert(source.clone());
            }
        }
    }
    return all_backward_edges;
}


fn forward_topological_sort(function: &FunctionValue) -> Vec<String> {
    let forward_edges = get_forward_edges(function);
    let backward_edges = get_backward_edges(function);
    let mut sorted = Vec::new();
    let mut unsorted = Vec::new();
    for bb in function.get_basic_blocks() {
        let basic_block_name = String::from(bb.get_name().to_str().unwrap());
        unsorted.push(basic_block_name);
    }
    let num_nodes = unsorted.len();

    let mut indegrees = HashMap::new();
    for node in &unsorted {
        if let Some(reverse_dests) = backward_edges.get(&node.clone()) {
            let mut indegree = 0;
            for _j in 0..reverse_dests.len() {
                indegree += 1;
            }
            indegrees.insert(node, indegree);
        }
    }

    while sorted.len() < num_nodes {
        let mut next_node: Option<String> = None;
        for node in &unsorted {
            if let Some(indegree) = indegrees.get(&node.clone()) {
                if (*indegree) == 0 {
                    indegrees.insert(node, -1);
                    next_node = Some(node.to_string());
                    sorted.push(node.to_string());
                    if let Some(dests) = forward_edges.get(&node.clone()) {
                        for dest in dests.into_iter() {
                            if let Some(prev_indegree) = indegrees.get_mut(&dest.clone()) {
                                *prev_indegree = *prev_indegree - 1;
                            }
                        }
                    }
                }
            }
        }
        match next_node {
            Some(..) => (),
            None => {
                warn!("CFG is cyclic which is not supported");
                break;
            }
        }
    }
    return sorted;
}


fn backward_topological_sort(function: &FunctionValue) -> Vec<String> {
    let mut sorted = forward_topological_sort(function);
    sorted.reverse();
    return sorted;
}


fn get_field_to_extract(instruction: &InstructionValue) -> String {
    let instruction_string = instruction.to_string();
    return String::from(&instruction_string[instruction_string.rfind(" ").unwrap()+1..instruction_string.rfind("\"").unwrap()]);
}


fn get_entry_condition<'a>(
    solver: &'a Solver<'_>,
    function: &'a FunctionValue,
    predecessor: &str,
    node: &str,
) -> Bool<'a> {
    let mut entry_condition = Bool::from_bool(solver.get_context(), true);
    if let Some(terminator) = get_basic_block_by_name(function, &String::from(predecessor)).get_terminator() {
        let opcode = terminator.get_opcode();
        let num_operands = terminator.get_num_operands();
        match &opcode {
            InstructionOpcode::Br => {
                if num_operands == 1 {
                    // Unconditionally go to node
                } else if num_operands == 3 {
                    let mut target_val = true;
                    let discriminant = terminator.get_operand(0).unwrap().left().unwrap();
                    let successor_basic_block_1 = terminator.get_operand(1).unwrap().right().unwrap();
                    let successor_basic_block_name_1 = String::from(successor_basic_block_1.get_name().to_str().unwrap());
                    if successor_basic_block_name_1.eq(&String::from(node)) {
                        target_val = false;
                    }
                    let target_val_var =
                        Bool::from_bool(solver.get_context(), target_val);
                    let switch_var = Bool::new_const(
                        solver.get_context(),
                        get_var_name(&discriminant, &solver),
                    );

                    entry_condition = switch_var._eq(&target_val_var);

                } else {
                    warn!("Incorrect number of operators {:?} for opcode {:?} for edge generations", num_operands, opcode);
                }
            }
            InstructionOpcode::Switch => {
                let discriminant = terminator.get_operand(0).unwrap().left().unwrap();
                let mut target_val = terminator.get_operand(0).unwrap().left().unwrap();
                for i in 0..num_operands {
                    if i % 2 == 1 {
                        let basic_block = terminator.get_operand(i).unwrap().right().unwrap();
                        let basic_block_name = String::from(basic_block.get_name().to_str().unwrap());
                        if basic_block_name.eq(&String::from(node)) {
                            target_val = terminator.get_operand(i-1).unwrap().left().unwrap();
                            break;
                        }
                    }
                }
                let switch_var = Int::new_const(
                    solver.get_context(),
                    get_var_name(&discriminant, &solver),
                );

                if target_val == terminator.get_operand(0).unwrap().left().unwrap() {
                    // default
                    for j in 2..num_operands {
                        if j % 2 == 0 { 
                            let temp_target_val = terminator.get_operand(j).unwrap().left().unwrap();
                            let temp_target_val_var = Int::new_const(solver.get_context(), get_var_name(&temp_target_val, &solver));
                            entry_condition = Bool::and(solver.get_context(), &[&(switch_var._eq(&temp_target_val_var)).not(), &entry_condition]);
                        }
                    }
                } else {
                    let target_val_var = Int::new_const(solver.get_context(), get_var_name(&target_val, &solver));
                    entry_condition = switch_var._eq(&target_val_var);
                }
            }
            InstructionOpcode::Return => {
                // Unconditionally go to node
            }
            InstructionOpcode::Unreachable => {
                // Unconditionally go to node
            }
            _ => {
                warn!("Opcode {:?} is not supported as a terminator for computing node entry conditions", opcode);
            },
        }
    } else {
        warn!("\tNo terminator found when computing node entry conditions");
    }
    return entry_condition;
}
// TODO: Maybe return param names in Z3 space?
pub fn function_codegen(function: &FunctionValue, solver: &Solver, _namespace: &String) -> () {
    //! Perform backward symbolic execution on a function given the llvm-ir function object
    let forward_edges = get_forward_edges(&function);
    let backward_edges = get_backward_edges(&function);
    let backward_sorted_nodes = backward_topological_sort(&function);

    for node in backward_sorted_nodes {
        let mut successors = &HashSet::<String>::new();
        successors = forward_edges.get(&node).unwrap_or(successors);
        let mut node_var = if successors.len() == 0 {
            // handle panic (conceptually assign panic var and assert)
            // (panic <- bool_var) => !panic
            // equivalent to !panic
            let is_panic = is_panic_block(&get_basic_block_by_name(&function, &node)) == IsCleanup::YES;
            Bool::from_bool(solver.get_context(), !is_panic)
        } else {
            let mut successor_conditions = Bool::from_bool(solver.get_context(), true);
            if let Some(successors) = forward_edges.get(&node) {
                for successor in successors {
                    let successor_var =
                        Bool::new_const(solver.get_context(), String::from(successor));
                    successor_conditions =
                        Bool::and(solver.get_context(), &[&successor_conditions, &successor_var]);
                }
            }
            successor_conditions
        };

        // Parse statements in the basic block
        let mut prev_instruction = get_basic_block_by_name(&function, &node).get_last_instruction();

        while let Some(current_instruction) = prev_instruction {
            // Process current instruction
            let opcode = current_instruction.get_opcode();
            match &opcode {
                InstructionOpcode::Unreachable => {
                    // NO-OP
                }
                InstructionOpcode::Call => {
                    let call_operand = &current_instruction.get_operand(current_instruction.get_num_operands()-1)
                        .unwrap().left().unwrap().into_pointer_value();
                    let call_operation_name_string = get_function_name(call_operand);
                    let call_operation_name_str = call_operation_name_string.as_str();

                    match call_operation_name_str {
                        "llvm.sadd.with.overflow.i32" => {
                            let operand1_name = get_var_name(
                                &current_instruction.get_operand(0).unwrap().left().unwrap(),
                                &solver
                            );
                            let operand2_name = get_var_name(
                                &current_instruction.get_operand(1).unwrap().left().unwrap(),
                                &solver
                            );

                            let lvalue_var_name_1 = format!("{}.0", get_var_name(&current_instruction, &solver));
                            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                            let rvalue_var_1 = Int::add(solver.get_context(), &[
                                &Int::new_const(solver.get_context(), operand1_name),
                                &Int::new_const(solver.get_context(), operand2_name)
                            ]);
                            
                            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                            let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                            let min_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
                            let max_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
                            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);
                            
                            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
                            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
                            node_var = assignment.implies(&node_var);
                        }
                        "llvm.sadd.with.overflow.i64" => {
                            let operand1_name = get_var_name(
                                &current_instruction.get_operand(0).unwrap().left().unwrap(),
                                &solver
                            );
                            let operand2_name = get_var_name(
                                &current_instruction.get_operand(1).unwrap().left().unwrap(),
                                &solver
                            );

                            let lvalue_var_name_1 = format!("{}.0", get_var_name(&current_instruction, &solver));
                            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                            let rvalue_var_1 = Int::add(solver.get_context(), &[
                                &Int::new_const(solver.get_context(), operand1_name),
                                &Int::new_const(solver.get_context(), operand2_name)
                            ]);
                            
                            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                            let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                            let min_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MIN.into(), 64), true);
                            let max_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MAX.into(), 64), true);
                            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);
                            
                            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
                            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
                            node_var = assignment.implies(&node_var);
                        }
                        "llvm.ssub.with.overflow.i32" => {
                            let operand1_name = get_var_name(
                                &current_instruction.get_operand(0).unwrap().left().unwrap(),
                                &solver
                            );
                            let operand2_name = get_var_name(
                                &current_instruction.get_operand(1).unwrap().left().unwrap(),
                                &solver
                            );

                            let lvalue_var_name_1 = format!("{}.0", get_var_name(&current_instruction, &solver));
                            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                            let rvalue_var_1 = Int::sub(solver.get_context(), &[
                                &Int::new_const(solver.get_context(), operand1_name),
                                &Int::new_const(solver.get_context(), operand2_name)
                            ]);
                            
                            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                            let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                            let min_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
                            let max_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
                            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);
                            
                            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
                            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
                            node_var = assignment.implies(&node_var);
                        }
                        "llvm.ssub.with.overflow.i64" => {
                            let operand1_name = get_var_name(
                                &current_instruction.get_operand(0).unwrap().left().unwrap(),
                                &solver
                            );
                            let operand2_name = get_var_name(
                                &current_instruction.get_operand(1).unwrap().left().unwrap(),
                                &solver
                            );

                            let lvalue_var_name_1 = format!("{}.0", get_var_name(&current_instruction, &solver));
                            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                            let rvalue_var_1 = Int::sub(solver.get_context(), &[
                                &Int::new_const(solver.get_context(), operand1_name),
                                &Int::new_const(solver.get_context(), operand2_name)
                            ]);
                            
                            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                            let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                            let min_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MIN.into(), 64), true);
                            let max_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MAX.into(), 64), true);
                            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);
                            
                            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
                            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
                            node_var = assignment.implies(&node_var);
                        }
                        "llvm.smul.with.overflow.i32" => {
                            let operand1_name = get_var_name(
                                &current_instruction.get_operand(0).unwrap().left().unwrap(),
                                &solver
                            );
                            let operand2_name = get_var_name(
                                &current_instruction.get_operand(1).unwrap().left().unwrap(),
                                &solver
                            );

                            let lvalue_var_name_1 = format!("{}.0", get_var_name(&current_instruction, &solver));
                            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                            let rvalue_var_1 = Int::mul(solver.get_context(), &[
                                &Int::new_const(solver.get_context(), operand1_name),
                                &Int::new_const(solver.get_context(), operand2_name)
                            ]);
                            
                            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                            let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                            let min_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
                            let max_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
                            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);
                            
                            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
                            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
                            node_var = assignment.implies(&node_var);
                        }
                        "llvm.smul.with.overflow.i64" => {
                            let operand1_name = get_var_name(
                                &current_instruction.get_operand(0).unwrap().left().unwrap(),
                                &solver
                            );
                            let operand2_name = get_var_name(
                                &current_instruction.get_operand(1).unwrap().left().unwrap(),
                                &solver
                            );

                            let lvalue_var_name_1 = format!("{}.0", get_var_name(&current_instruction, &solver));
                            let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                            let rvalue_var_1 = Int::mul(solver.get_context(), &[
                                &Int::new_const(solver.get_context(), operand1_name),
                                &Int::new_const(solver.get_context(), operand2_name)
                            ]);
                            
                            let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                            let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                            let min_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MIN.into(), 64), true);
                            let max_int =
                                Int::from_bv(&BV::from_i64(solver.get_context(), i64::MAX.into(), 64), true);
                            let rvalue_var_2 = Bool::or(solver.get_context(), &[&rvalue_var_1.gt(&max_int), &rvalue_var_1.lt(&min_int)]);
                            
                            let assignment_2 = Bool::new_const(solver.get_context(), lvalue_var_name_2)._eq(&rvalue_var_2);
                            let assignment = Bool::and(solver.get_context(), &[&assignment_1, &assignment_2]);
                            node_var = assignment.implies(&node_var);
                        }
                        "llvm.expect.i1" => {
                            let lvalue_var_name = get_var_name(
                                &current_instruction,
                                &solver
                            );
                            let operand1_name = get_var_name(
                                &current_instruction.get_operand(0).unwrap().left().unwrap(),
                                &solver
                            );
                            let operand2_name = get_var_name(
                                &current_instruction.get_operand(1).unwrap().left().unwrap(),
                                &solver
                            );
                            let rvalue_var = Bool::new_const(solver.get_context(), operand1_name)._eq(&Bool::new_const(solver.get_context(), operand2_name));
                            let assignment = Bool::new_const(solver.get_context(), lvalue_var_name)._eq(&rvalue_var);
                            node_var = assignment.implies(&node_var);
                        }
                        "core::panicking::panic::he60bb304466ccbaf" => {
                            // NO-OP
                        }
                        _ => {
                            warn!("Unsupported Call function {:?}", call_operation_name_str);
                        }
                    }
                }
                InstructionOpcode::Return => {
                    // NO-OP
                }
                InstructionOpcode::Switch => {
                    // NO-OP
                }
                InstructionOpcode::Load => {
                    // TODO: Support non-int types here
                    let operand = current_instruction.get_operand(0).unwrap().left().unwrap();
                    if !current_instruction.get_type().is_int_type() {
                        warn!("Currently unsuppported type {:?} for load operand", current_instruction.get_type().to_string())
                    }
                    let lvalue_var_name = get_var_name(&current_instruction, &solver);
                    let rvalue_var_name = get_var_name(&operand, &solver);
                    let lvalue_var = Int::new_const(
                        solver.get_context(),
                        lvalue_var_name
                    );
                    let rvalue_var = Int::new_const(
                        solver.get_context(),
                        rvalue_var_name
                    );
                    let assignment = lvalue_var._eq(&rvalue_var);
                    node_var = assignment.implies(&node_var);
                }
                InstructionOpcode::Store => {
                    // TODO: Support non-int types here
                    let operand1 = current_instruction.get_operand(0).unwrap().left().unwrap();
                    if !operand1.get_type().is_int_type() {
                        warn!("Currently unsuppported type {:?} for store operand", operand1.get_type().to_string())
                    }
                    let operand2 = current_instruction.get_operand(1).unwrap().left().unwrap().into_pointer_value();
                    
                    let lvalue_var_name = get_var_name(&operand1, &solver);
                    let rvalue_var_name = get_var_name(&operand2, &solver);
                    let lvalue_var = Int::new_const(
                        solver.get_context(),
                        lvalue_var_name
                    );
                    let rvalue_var = Int::new_const(
                        solver.get_context(),
                        rvalue_var_name
                    );
                    let assignment = lvalue_var._eq(&rvalue_var);
                    node_var = assignment.implies(&node_var);
                }
                InstructionOpcode::Br => {
                    // NO-OP
                }
                InstructionOpcode::Xor => {
                    let operand1_var_name = get_var_name(&current_instruction.get_operand(0).unwrap().left().unwrap(), &solver);
                    let operand2_var_name = get_var_name(&current_instruction.get_operand(1).unwrap().left().unwrap(), &solver);
                    if !current_instruction.get_type().to_string().eq("\"i1\"") {
                        warn!("Currently unsuppported type {:?} for xor operand", current_instruction.get_type().to_string());
                    }
                    let operand1_var = Bool::new_const(
                        solver.get_context(),
                        operand1_var_name
                    );
                    let operand2_var = Bool::new_const(
                        solver.get_context(),
                        operand2_var_name
                    );
                    let rvalue_var = operand1_var.xor(&operand2_var);
                    let lvalue_var_name = get_var_name(&current_instruction, &solver);
                    let lvalue_var = Bool::new_const(
                        solver.get_context(),
                        lvalue_var_name
                    );
                    let assignment = lvalue_var._eq(&rvalue_var);
                    node_var = assignment.implies(&node_var);
                }
                InstructionOpcode::ICmp => {
                    let lvalue_var_name = get_var_name(&current_instruction, &solver);
                    let lvalue_var = Bool::new_const(solver.get_context(), lvalue_var_name);
                    let operand1 = get_var_name(&current_instruction.get_operand(0).unwrap().left().unwrap(), &solver);
                    let operand2 = get_var_name(&current_instruction.get_operand(1).unwrap().left().unwrap(), &solver);
                    let rvalue_operation;
                    

                    // Split by the sub-instruction (denoting the type of comparison)
                    // TODO: can signed & unsigned comparisons be combined?
                    let icmp_type = current_instruction.get_icmp_predicate().unwrap();
                    match &icmp_type {
                        IntPredicate::EQ => {
                            rvalue_operation = Int::new_const(&solver.get_context(), operand1)._eq(
                                &Int::new_const(&solver.get_context(), operand2)
                            );
                        }
                        IntPredicate::NE => {
                            rvalue_operation = Int::new_const(&solver.get_context(), operand1)._eq(
                                &Int::new_const(&solver.get_context(), operand2)
                            ).not();
                        }
                        IntPredicate::SGE | IntPredicate::UGE => {
                            rvalue_operation = Int::new_const(&solver.get_context(), operand1).ge(
                                &Int::new_const(&solver.get_context(), operand2)
                            );
                        }
                        IntPredicate::SGT | IntPredicate::UGT => {
                            rvalue_operation = Int::new_const(&solver.get_context(), operand1).gt(
                                &Int::new_const(&solver.get_context(), operand2)
                            );
                        }
                        IntPredicate::SLE | IntPredicate::ULE => {
                            rvalue_operation = Int::new_const(&solver.get_context(), operand1).le(
                                &Int::new_const(&solver.get_context(), operand2)
                            );
                        }
                        IntPredicate::SLT | IntPredicate::ULT => {
                            rvalue_operation = Int::new_const(&solver.get_context(), operand1).lt(
                                &Int::new_const(&solver.get_context(), operand2)
                            );
                        }
                    }

                    let assignment = lvalue_var._eq(&rvalue_operation);
                    node_var = assignment.implies(&node_var);
                }
                InstructionOpcode::ExtractValue => {
                    let lvalue_var_name = get_var_name(&current_instruction, &solver);
                    let operand = current_instruction.get_operand(0).unwrap().left().unwrap();
                    let rvalue_var_name = format!("{}.{}", get_var_name(&operand, &solver), get_field_to_extract(&current_instruction));
                    if current_instruction.get_type().to_string().eq("\"i1\"") {
                        let lvalue_var = Bool::new_const(
                            solver.get_context(),
                            lvalue_var_name
                        );
                        let rvalue_var = Bool::new_const(
                            solver.get_context(),
                            rvalue_var_name
                        );
                        let assignment = lvalue_var._eq(&rvalue_var);
                        node_var = assignment.implies(&node_var);       
                    } else if current_instruction.get_type().is_int_type() {
                        let lvalue_var = Int::new_const(
                            solver.get_context(),
                            lvalue_var_name
                        );
                        let rvalue_var = Int::new_const(
                            solver.get_context(),
                            rvalue_var_name
                        );
                        let assignment = lvalue_var._eq(&rvalue_var);
                        node_var = assignment.implies(&node_var);     
                    }  else {
                        warn!("Currently unsuppported type {:?} for extract value", operand.get_type().to_string())
                    } 
                }
                InstructionOpcode::Alloca => {
                    // NO-OP
                }
                InstructionOpcode::Phi => {
                    let phi_instruction: PhiValue = current_instruction.try_into().unwrap();
                    let mut assignment = Bool::from_bool(solver.get_context(), true);
                    for incoming_index in 0..phi_instruction.count_incoming() {
                        let incoming = phi_instruction.get_incoming(incoming_index).unwrap();
                        let predecessor = incoming.1.get_name().to_str().unwrap();
                        let phi_condition = get_entry_condition(&solver, &function, &predecessor, &node);
                        let lvalue_var_name = get_var_name(&current_instruction, &solver);
                        let rvalue_var_name = get_var_name(&incoming.0, &solver);
                        if current_instruction.get_type().to_string().eq("\"i1\"") {
                            let lvalue_var = Bool::new_const(
                                solver.get_context(),
                                lvalue_var_name
                            );
                            let rvalue_var = Bool::new_const(
                                solver.get_context(),
                                rvalue_var_name
                            );
                            assignment = Bool::and(&solver.get_context(), &[&assignment, &phi_condition.implies(&lvalue_var._eq(&rvalue_var))]);
                        } else if current_instruction.get_type().is_int_type() {
                            let lvalue_var = Int::new_const(
                                solver.get_context(),
                                lvalue_var_name
                            );
                            let rvalue_var = Int::new_const(
                                solver.get_context(),
                                rvalue_var_name
                            );
                            assignment = Bool::and(&solver.get_context(), &[&assignment, &phi_condition.implies(&lvalue_var._eq(&rvalue_var))]);
                        } else {
                            warn!("Currently unsuppported type {:?} for Phi", incoming.0.get_type().to_string());
                        }
                    }
                    node_var = assignment.implies(&node_var);
                }
                InstructionOpcode::Trunc => {
                    let lvalue_var_name = get_var_name(&current_instruction, &solver);
                    let operand_var_name = get_var_name(&current_instruction.get_operand(0).unwrap().left().unwrap(), &solver);
                    let lvalue_var = Bool::new_const(
                        solver.get_context(),
                        lvalue_var_name
                    );
                    let operand_var = Int::new_const(
                        solver.get_context(),
                        operand_var_name
                    );
                    let const_1 = Int::from_i64(solver.get_context(), 1);
                    let const_2 = Int::from_i64(solver.get_context(), 2);
                    let right_most_bit = operand_var.modulo(&const_2);
                    let assignment = lvalue_var._eq(&right_most_bit._eq(&const_1));
                    node_var = assignment.implies(&node_var);
                    warn!("Trunc is only partially supported (always i1)");
                }
                InstructionOpcode::Select => {
                    let discriminant = current_instruction.get_operand(0).unwrap().left().unwrap();
                    let discriminant_name = get_var_name(&discriminant, &solver);
                    let operand_1_var_name = get_var_name(&current_instruction.get_operand(1).unwrap().left().unwrap(), &solver);
                    let operand_2_var_name = get_var_name(&current_instruction.get_operand(2).unwrap().left().unwrap(), &solver);
                    if !discriminant.get_type().to_string().eq("\"i1\"") {
                        warn!("Currently unsuppported type {:?} for select discriminant", discriminant.get_type().to_string());
                        continue;
                    }
                    let discriminant_var = Bool::new_const(
                        solver.get_context(),
                        discriminant_name
                    );
                    if current_instruction.get_type().to_string().eq("\"i1\"") {
                        let operand_1_var = Bool::new_const(
                            solver.get_context(),
                            operand_1_var_name
                        );
                        let operand_2_var = Bool::new_const(
                            solver.get_context(),
                            operand_2_var_name
                        );                            
                        let select_1 = discriminant_var.implies(&Bool::new_const(solver.get_context(), get_var_name(&current_instruction, &solver))._eq(&operand_1_var));
                        let select_2 = discriminant_var.not().implies(&Bool::new_const(solver.get_context(), get_var_name(&current_instruction, &solver))._eq(&operand_2_var));
                        node_var = Bool::and(solver.get_context(), &[&select_1.implies(&node_var), &select_2.implies(&node_var)]);
                    } else if current_instruction.get_type().is_int_type() {
                        let operand_1_var = Int::new_const(
                            solver.get_context(),
                            operand_1_var_name
                        );
                        let operand_2_var = Int::new_const(
                            solver.get_context(),
                            operand_2_var_name
                        );                            
                        let select_1 = discriminant_var.implies(&Int::new_const(solver.get_context(), get_var_name(&current_instruction, &solver))._eq(&operand_1_var));
                        let select_2 = discriminant_var.not().implies(&Int::new_const(solver.get_context(), get_var_name(&current_instruction, &solver))._eq(&operand_2_var));
                        let assignment = Bool::and(solver.get_context(), &[&select_1, &select_2]);
                        node_var = assignment.implies(&node_var);
                    } else {
                        warn!("Currently unsuppported type {:?} for select", current_instruction.get_type().to_string());
                    }
                }
                InstructionOpcode::ZExt => {
                    let lvalue_var_name = get_var_name(&current_instruction, &solver);
                    let operand_var_name = get_var_name(&current_instruction.get_operand(0).unwrap().left().unwrap(), &solver);
                    let lvalue_var = Int::new_const(
                        solver.get_context(),
                        lvalue_var_name
                    );
                    let operand_var = Bool::new_const(
                        solver.get_context(),
                        operand_var_name
                    );
                    let const_1 = Int::from_i64(solver.get_context(), 1);
                    let const_0 = Int::from_i64(solver.get_context(), 0);
                    let cast_1 = operand_var.implies(&lvalue_var._eq(&const_1));
                    let cast_2 = operand_var.not().implies(&lvalue_var._eq(&const_0));
                    let assignment = Bool::and(solver.get_context(), &[&cast_1, &cast_2]);
                    node_var = assignment.implies(&node_var);                        
                    warn!("ZExt is only partially supported (always i32)");
                }
                _ => {
                    warn!("Opcode {:?} is not supported as a statement for code gen", opcode);
                }
            }

            prev_instruction = current_instruction.get_previous_instruction();
        }

        let mut entry_conditions = Bool::from_bool(solver.get_context(), true);
        if let Some(predecessors) = backward_edges.get(&node) {
            if predecessors.len() > 0 {
                for predecessor in predecessors {
                    // get conditions
                    let entry_condition = get_entry_condition(&solver, &function, &predecessor, &node);
                    entry_conditions = Bool::and(solver.get_context(), &[&entry_conditions, &entry_condition]);
                }
            }
        }  
        node_var = entry_conditions.implies(&node_var);

        let named_node_var = Bool::new_const(solver.get_context(), String::from(node));
        solver.assert(&named_node_var._eq(&node_var));
    }

    // constrain int inputs
    for input in function.get_params() {
        // TODO: Support other input types
        if input.get_type().to_string().eq("\"i1\"") {
            continue;
        } else if input.get_type().to_string().eq("\"i32\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver));
            let min_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
            let max_int =
                Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
            solver
                .assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        } else if input.get_type().to_string().eq("\"i64\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver));
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