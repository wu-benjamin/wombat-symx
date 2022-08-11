use std::collections::{HashMap, HashSet};
use std::env;

use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::values::{FunctionValue, InstructionOpcode, AnyValue, InstructionValue, PhiValue};
use rustc_demangle::demangle;
use z3::{
    ast::{Ast, Bool, Int, BV},
    SatResult,
};
use z3::{Config, Solver};
use z3::Context as Z3Context;

use inkwell::context::Context as InkwellContext;
use inkwell::module::Module as InkwellModule;
use inkwell::memory_buffer::MemoryBuffer;
use std::path::Path;

const COMMON_END_NODE_NAME: &str = "common_end";
const PANIC_VAR_NAME: &str = "panic_var";

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
                println!("Multiple basic blocks matched name {:?}", name);
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
                println!("Opcode {:?} is not supported as a terminator for panic detection", opcode);
                return IsCleanup::UNKNOWN;
            }
        }
    } else {
        println!("\tNo terminator found for panic detection");
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
                    node_edges.insert(String::from(COMMON_END_NODE_NAME));
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
                        println!("Incorrect number of operators {:?} for opcode {:?} for edge generations", num_operands, opcode);
                    }
                }
                InstructionOpcode::Switch => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::IndirectBr => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Invoke => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CallBr => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Resume => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchSwitch => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchRet => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CleanupRet => {
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Unreachable => {
                    node_edges.insert(String::from(COMMON_END_NODE_NAME));
                }
                _ => {
                    println!("Opcode {:?} is not supported as a terminator for edge generation", opcode);
                }
            }
        } else {
            println!("\tNo terminator");
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
    all_backward_edges.insert(String::from(COMMON_END_NODE_NAME), HashSet::new());
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
    unsorted.push(String::from(COMMON_END_NODE_NAME));
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
                println!("CFG is cyclic which is not supported");
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


fn get_var_name<'a>(value: &dyn AnyValue, solver: &'a Solver<'_>) -> String {
    // handle const literal
    let llvm_str = value.print_to_string();
    let str = llvm_str.to_str().unwrap();
    // println!("{:?}", str);
    if !str.contains("%") {
        let var_name = str.split_whitespace().nth(1).unwrap();
        if var_name.eq("true") {
            let true_const = Bool::new_const(solver.get_context(), format!("const_{}", var_name));
            solver.assert(&true_const._eq(&Bool::from_bool(solver.get_context(), true)));
        } else if var_name.eq("false") {
            let false_const = Bool::new_const(solver.get_context(), format!("const_{}", var_name));
            solver.assert(&false_const._eq(&Bool::from_bool(solver.get_context(), false)));
        } else {
            let parsed_num = var_name.parse::<i32>().unwrap();
            let num_const = Int::new_const(solver.get_context(), format!("const_{}", var_name));
            solver.assert(&num_const._eq(&Int::from_i64(solver.get_context(), parsed_num.into())));
        }
        return String::from(format!("const_{}", var_name));
    }
    let start_index = str.find("%").unwrap();
    let end_index = str[start_index..].find(|c: char| c == '"' || c == ' ' || c == ',').unwrap_or(str[start_index..].len()) + start_index;
    let var_name = String::from(&str[start_index..end_index]);
    return String::from(var_name);
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
                    // println!("\n--------- {:?}", successor_basic_block_name_1);
                    // println!("--------- {:?}", String::from(terminator.get_operand(2).unwrap().right().unwrap().get_name().to_str().unwrap()));
                    // println!("--------- {:?}", String::from(node));
                    // println!("--------- {:?}\n", terminator);
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
                    println!("Incorrect number of operators {:?} for opcode {:?} for edge generations", num_operands, opcode);
                }
            }
            InstructionOpcode::Return => {
                // Unconditionally go to node
            }
            InstructionOpcode::Unreachable => {
                // Unconditionally go to node
            }
            _ => {
                println!("Opcode {:?} is not supported as a terminator for computing node entry conditions", opcode);
            },
        }
    } else {
        println!("\tNo terminator found when computing node entry conditions");
    }
    return entry_condition;
}


fn backward_symbolic_execution(function: &FunctionValue) -> () {
    //! Perform backward symbolic execution on a function given the llvm-ir function object
    let forward_edges = get_forward_edges(&function);
    let backward_edges = get_backward_edges(&function);
    let backward_sorted_nodes = backward_topological_sort(&function);

    // Initialize the Z3 and Builder objects
    let cfg = Config::new();
    let ctx = Z3Context::new(&cfg);
    let solver = Solver::new(&ctx);

    for node in backward_sorted_nodes {
        let mut successor_conditions = Bool::from_bool(solver.get_context(), true);
        if let Some(successors) = forward_edges.get(&node) {
            for successor in successors {
                let successor_var =
                    Bool::new_const(solver.get_context(), String::from(successor));
                successor_conditions =
                    Bool::and(solver.get_context(), &[&successor_conditions, &successor_var]);
            }
        }
        let mut node_var = successor_conditions;

        if let Some(successors) = forward_edges.get(&node) {
            for successor in successors {
                // look at all phi functions
                if successor.eq(COMMON_END_NODE_NAME) {
                    continue;
                }
                let condition = get_entry_condition(&solver, &function, &node, &successor);
                let mut next_instruction = get_basic_block_by_name(&function, &successor).get_first_instruction();
                while let Some(current_instruction) = next_instruction {
                    let opcode = current_instruction.get_opcode();
                    match &opcode {
                        InstructionOpcode::Phi => {
                            let phi_instruction: PhiValue = current_instruction.try_into().unwrap();
                            for incoming_index in 0..phi_instruction.count_incoming() {
                                let incoming = phi_instruction.get_incoming(incoming_index).unwrap();
                                if incoming.1.get_name().to_str().unwrap().eq(&node) {
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
                                        let assignment = condition.implies(&lvalue_var._eq(&rvalue_var));
                                        node_var = assignment.implies(&node_var);       
                                    } else if current_instruction.get_type().to_string().eq("\"i32\"") {
                                        let lvalue_var = Int::new_const(
                                            solver.get_context(),
                                            lvalue_var_name
                                        );
                                        let rvalue_var = Int::new_const(
                                            solver.get_context(),
                                            rvalue_var_name
                                        );
                                        let assignment = condition.implies(&lvalue_var._eq(&rvalue_var));
                                        // println!("BOOGA LOOGA: {:?}", assignment);
                                        node_var = assignment.implies(&node_var);
                                    } else {
                                        println!("Currently unsuppported type {:?} for extract value", incoming.0.get_type().to_string())
                                    } 
                                }
                            }
                        }
                        _ => {
                            // NO-OP
                        }
                    }
                    next_instruction = current_instruction.get_next_instruction();
                }
            }
        }

        if node == COMMON_END_NODE_NAME.to_string() {
            let panic_var = Bool::new_const(solver.get_context(), PANIC_VAR_NAME);
            node_var = Bool::and(solver.get_context(), &[&panic_var.not(), &node_var]);
        } else {
            // Parse statements in the basic block
            let mut prev_instruction = get_basic_block_by_name(&function, &node).get_last_instruction();

            while let Some(current_instruction) = prev_instruction {
                // TODO: Process current instruction
                let opcode = current_instruction.get_opcode();
                match &opcode {
                    InstructionOpcode::Unreachable => {
                        // NO-OP
                    }
                    InstructionOpcode::Call => {
                        // println!("---------------- Need to Implement------------------\n{:?}", current_instruction);
                        // println!("\tNumber of operands: {:?}", current_instruction.get_num_operands());
                        // for i in 0..current_instruction.get_num_operands() {
                        //     println!("\t\t{:?}", current_instruction.get_operand(i));
                        // }

                        let call_operand = &current_instruction.get_operand(current_instruction.get_num_operands()-1)
                            .unwrap().left().unwrap().into_pointer_value();
                        let call_operation_name = call_operand.get_name().to_str().unwrap();
                        // println!("\tCALL OPERATION: {:?}", call_operation_name);

                        match call_operation_name {
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
                                // println!("{:?}", lvalue_var_name_1);
                                let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                                let rvalue_var_1 = Int::add(solver.get_context(), &[
                                    &Int::new_const(solver.get_context(), operand1_name),
                                    &Int::new_const(solver.get_context(), operand2_name)
                                ]);
                                
                                let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                                let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                                // println!("{:?}", lvalue_var_name_2);
                                let min_int =
                                    Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
                                let max_int =
                                    Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
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
                                // println!("{:?}", lvalue_var_name_1);
                                let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                                let rvalue_var_1 = Int::sub(solver.get_context(), &[
                                    &Int::new_const(solver.get_context(), operand1_name),
                                    &Int::new_const(solver.get_context(), operand2_name)
                                ]);
                                
                                let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                                let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                                // println!("{:?}", lvalue_var_name_2);
                                let min_int =
                                    Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
                                let max_int =
                                    Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
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
                                // println!("{:?}", lvalue_var_name_1);
                                let lvalue_var_1 = Int::new_const(solver.get_context(), lvalue_var_name_1);
                                let rvalue_var_1 = Int::mul(solver.get_context(), &[
                                    &Int::new_const(solver.get_context(), operand1_name),
                                    &Int::new_const(solver.get_context(), operand2_name)
                                ]);
                                
                                let assignment_1 = lvalue_var_1._eq(&rvalue_var_1);

                                let lvalue_var_name_2 = format!("{}.1", get_var_name(&current_instruction, &solver));
                                // println!("{:?}", lvalue_var_name_2);
                                let min_int =
                                    Int::from_bv(&BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
                                let max_int =
                                    Int::from_bv(&BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
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
                            "_ZN4core9panicking5panic17he60bb304466ccbafE" => {
                                // NO-OP
                            }
                            _ => {
                                println!("Unsupported Call function {:?}", call_operation_name);
                            }
                        }
                    }
                    InstructionOpcode::Return => {
                        // NO-OP
                    }
                    InstructionOpcode::Load => {
                        // TODO: Support types other than i32* here
                        let operand = current_instruction.get_operand(0).unwrap().left().unwrap();
                        if !current_instruction.get_type().to_string().eq("\"i32\"") {
                            println!("Currently unsuppported type {:?} for load operand", current_instruction.get_type().to_string())
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
                        // TODO: Support types other than i32* here
                        // TODO: Alloca seems to cause issues with multiple elements accessing this stored value
                        // println!("---------------- Need to Implement------------------\n{:?}", current_instruction);
                        // println!("\tNumber of operands: {:?}", current_instruction.get_num_operands());
                        // for i in 0..current_instruction.get_num_operands() {
                        //     println!("\t\t{:?}", current_instruction.get_operand(i));
                        // }
                        // println!("\t\tptr value: {:?}", get_var_name(&current_instruction.get_operand(1).unwrap().left().unwrap().into_pointer_value().as_any_value_enum(), &solver));

                        let operand1 = current_instruction.get_operand(0).unwrap().left().unwrap();
                        if !operand1.get_type().to_string().eq("\"i32\"") {
                            println!("Currently unsuppported type {:?} for store operand", operand1.get_type().to_string())
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
                        // println!("\tCreated operation: {:?}", assignment);
                        node_var = assignment.implies(&node_var);
                    }
                    InstructionOpcode::Br => {
                        // NO-OP
                    }
                    InstructionOpcode::Xor => {
                        let operand1_var_name = get_var_name(&current_instruction.get_operand(0).unwrap().left().unwrap(), &solver);
                        let operand2_var_name = get_var_name(&current_instruction.get_operand(1).unwrap().left().unwrap(), &solver);
                        if !current_instruction.get_type().to_string().eq("\"i1\"") {
                            println!("Currently unsuppported type {:?} for xor operand", current_instruction.get_type().to_string());
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
                        } else if current_instruction.get_type().to_string().eq("\"i32\"") {
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
                        } else {
                            println!("Currently unsuppported type {:?} for extract value", operand.get_type().to_string())
                        } 
                    }
                    InstructionOpcode::Alloca => {
                        // NO-OP
                    }
                    InstructionOpcode::Phi => {
                        // NO-OP
                    }
                    InstructionOpcode::Trunc => {
                        let lvalue_var_name = get_var_name(&current_instruction, &solver);
                        let operand_var_name = get_var_name(&current_instruction.get_operand(0).unwrap().left().unwrap(), &solver);
                        println!("Trunc: {:?}", lvalue_var_name);
                        println!("Trunc: {:?}", operand_var_name);
                        println!("Trunc: {:?}", current_instruction);
                        let lvalue_var = Bool::new_const(
                            solver.get_context(),
                            lvalue_var_name
                        );
                        let operand_var = Int::new_const(
                            solver.get_context(),
                            operand_var_name
                        );
                        let const_0 = Int::from_bv(&BV::from_i64(solver.get_context(), 0, 32), true);
                        let assignment = lvalue_var._eq(&operand_var._eq(&const_0).not());
                        node_var = assignment.implies(&node_var);                        
                        println!("Trunc is only partially supported (always i1)");
                    }
                    InstructionOpcode::Select => {
                        let discriminant = current_instruction.get_operand(0).unwrap().left().unwrap();
                        let discriminant_name = get_var_name(&discriminant, &solver);
                        let operand_1_var_name = get_var_name(&current_instruction.get_operand(1).unwrap().left().unwrap(), &solver);
                        let operand_2_var_name = get_var_name(&current_instruction.get_operand(2).unwrap().left().unwrap(), &solver);
                        if !discriminant.get_type().to_string().eq("\"i1\"") {
                            println!("Currently unsuppported type {:?} for select discriminant", discriminant.get_type().to_string());
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
                        } else if current_instruction.get_type().to_string().eq("\"i32\"") || current_instruction.get_type().to_string().eq("\"i8\"") {
                            if current_instruction.get_type().to_string().eq("\"i8\"") {
                                println!("i8 is only partially supported for select statements (treated as i32)");
                            }
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
                            println!("Currently unsuppported type {:?} for select", current_instruction.get_type().to_string());
                        }
                    }
                    InstructionOpcode::ZExt => {
                        let lvalue_var_name = get_var_name(&current_instruction, &solver);
                        let operand_var_name = get_var_name(&current_instruction.get_operand(0).unwrap().left().unwrap(), &solver);
                        println!("ZExt: {:?}", lvalue_var_name);
                        println!("ZExt: {:?}", operand_var_name);
                        println!("ZExt: {:?}", current_instruction);
                        let lvalue_var = Int::new_const(
                            solver.get_context(),
                            lvalue_var_name
                        );
                        let operand_var = Bool::new_const(
                            solver.get_context(),
                            operand_var_name
                        );
                        let const_1 = Int::from_bv(&BV::from_i64(solver.get_context(), 1, 32), true);
                        let const_0 = Int::from_bv(&BV::from_i64(solver.get_context(), 0, 32), true);
                        let cast_1 = operand_var.implies(&lvalue_var._eq(&const_1));
                        let cast_2 = operand_var.not().implies(&lvalue_var._eq(&const_0));
                        let assignment = Bool::and(solver.get_context(), &[&cast_1, &cast_2]);
                        node_var = assignment.implies(&node_var);                        
                        println!("ZExt is only partially supported (always i32)");
                    }
                    _ => {
                        println!("Opcode {:?} is not supported as a statement for code gen", opcode);
                    }
                }

                prev_instruction = current_instruction.get_previous_instruction();
            }

            // handle assign panic
            if let Some(successors) = forward_edges.get(&node) {
                let mut is_predecessor_of_end_node = false;
                for successor in successors {
                    if successor == COMMON_END_NODE_NAME {
                        is_predecessor_of_end_node = true;
                        break;
                    }
                }
                if is_predecessor_of_end_node {
                    let mut is_panic = false;
                    if is_panic_block(&get_basic_block_by_name(&function, &node)) == IsCleanup::YES {
                        is_panic = true;
                    }
                    let panic_var = Bool::new_const(solver.get_context(), PANIC_VAR_NAME);
                    let panic_value = Bool::from_bool(solver.get_context(), is_panic);
                    let panic_assignment = panic_var._eq(&panic_value);
                    node_var = panic_assignment.implies(&node_var);
                }
            }
        }

        let mut entry_conditions_set = false;
        let mut entry_conditions = Bool::from_bool(solver.get_context(), true);
        if let Some(predecessors) = backward_edges.get(&node) {
            if predecessors.len() > 0 {
                for predecessor in predecessors {
                    // get conditions
                    let entry_condition = get_entry_condition(&solver, &function, &predecessor, &node);
                    entry_conditions = Bool::and(solver.get_context(), &[&entry_conditions, &entry_condition]);
                }
                entry_conditions_set = true;
            }
        }  
        if !entry_conditions_set {
            entry_conditions = Bool::from_bool(solver.get_context(), true);
        }
        node_var = entry_conditions.implies(&node_var);

        let named_node_var = Bool::new_const(solver.get_context(), String::from(node));
        solver.assert(&named_node_var._eq(&node_var));
    }

    // // constrain int inputs
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
        } else {
            println!("Currently unsuppported type {:?} for input parameter", input.get_type().to_string())
        }
    }

    let start_node = function.get_first_basic_block().unwrap();
    let start_node_var_name = start_node.get_name().to_str().unwrap();
    let start_node_var = Bool::new_const(solver.get_context(), String::from(start_node_var_name));
    solver.assert(&start_node_var.not());
    println!("{:?}", solver);

    // Attempt resolving the model (and obtaining the respective arg values if panic found)
    println!("Function safety: {}", if solver.check() == SatResult::Sat {"unsafe"} else {"safe"});

    if solver.check() == SatResult::Sat {
        // TODO: Identify concrete function params for Sat case
        println!("\n{:?}", solver.get_model().unwrap());
    }
}


fn print_file_functions(module: &InkwellModule) -> () {
    //! Iterates through all functions in the file and prints the demangled name
    println!("Functions in {:?}:", module.get_name());
    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        println!("\t{:?} == {:?}", demangle(current_function.get_name().to_str().unwrap()).to_string(), current_function.get_name());
        next_function = current_function.get_next_function();
    }
}


fn pretty_print_function(function: &FunctionValue) -> () {
    println!("Number of Nodes: {}", function.count_basic_blocks());
    println!("Arg count: {}", function.count_params());
    // No local decl available to print
    println!("Basic Blocks:");
    for bb in function.get_basic_blocks() {
        println!("\tBasic Block: {:?}", bb.get_name().to_str().unwrap());
        println!("\t\tis_cleanup: {:?}", is_panic_block(&bb));
        let mut next_instruction = bb.get_first_instruction();
        let has_terminator = bb.get_terminator().is_some();

        while let Some(current_instruction) = next_instruction {
            println!("\t\tStatement: {:?}", current_instruction.to_string());
            next_instruction = current_instruction.get_next_instruction();
        }

        if has_terminator {
            println!("\t\tLast statement is a terminator")
        } else {
            println!("\t\tNo terminator")
        }
    }

    let first_basic_block = function.get_first_basic_block().unwrap();
    println!("Start node: {:?}", first_basic_block.get_name().to_str().unwrap());
    let forward_edges = get_forward_edges(function);
    let successors = forward_edges.get(first_basic_block.get_name().to_str().unwrap()).unwrap();
    for successor in successors {
        println!("\tSuccessor to start node: {:?}", successor);
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut file_name = String::from("tests/hello_world.bc");
    if args.len() > 1 {
        // Use custom user file
        file_name = args[1].to_string();
    }

    let path = Path::new(&file_name);
    let context = InkwellContext::create();
    let buffer = MemoryBuffer::create_from_file(&path).unwrap();
    let module = InkwellModule::parse_bitcode_from_buffer(&buffer, &context).unwrap();
    print_file_functions(&module);

    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        let current_function_name = demangle(&current_function.get_name().to_str().unwrap()).to_string();
        if current_function_name.contains(&file_name[file_name.rfind("/").unwrap()+1..file_name.rfind(".").unwrap()])
                && !current_function_name.contains("::main") {
            let pass_manager_builder = PassManagerBuilder::create();
            let pass_manager = PassManager::create(&module);
            pass_manager.add_promote_memory_to_register_pass();
            pass_manager_builder.populate_function_pass_manager(&pass_manager);
            pass_manager.run_on(&current_function);

            // Do not process main function for now
            println!("Backward Symbolic Execution in {:?}", demangle(current_function.get_name().to_str().unwrap()));
            pretty_print_function(&current_function);
            let forward_edges = get_forward_edges(&current_function);
            println!("Forward edges:\n\t{:?}", forward_edges);
            let backward_edges = get_backward_edges(&current_function);
            println!("Backward edges:\n\t{:?}", backward_edges);
            let forward_sorted_nodes = forward_topological_sort(&current_function);
            println!("Forward sorted nodes:\n\t{:?}", forward_sorted_nodes);
            let backward_sorted_nodes = backward_topological_sort(&current_function);
            println!("Backward sorted nodes:\n\t{:?}", backward_sorted_nodes);
            backward_symbolic_execution(&current_function);
            println!("\n\n************************************\n\n");
        }
        next_function = current_function.get_next_function();
    }
}
