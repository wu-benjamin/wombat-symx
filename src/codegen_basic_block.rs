use std::collections::{HashMap, HashSet};

use inkwell::basic_block::BasicBlock;
use inkwell::values::{FunctionValue, InstructionOpcode};
use tracing::{warn};
use z3::ast::{Ast, Bool, Int};
use z3::Solver;

use crate::get_var_name::get_var_name;
use crate::codegen_instruction::codegen_instruction;


type EdgeSet = HashMap<String, HashSet<String>>;


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


pub fn is_panic_block(bb: &BasicBlock) -> Option<bool> {
    if let Some(terminator) = bb.get_terminator() {
        let opcode = terminator.get_opcode();
        match &opcode {
            InstructionOpcode::Return => {
                return Some(false);
            }
            InstructionOpcode::Br => {
                return Some(false);
            }
            InstructionOpcode::Switch => {
                return Some(false);
            }
            InstructionOpcode::IndirectBr => {
                warn!("Unsure if opcode {:?} implies a panicking block.", opcode);
                return None;
            }
            InstructionOpcode::Invoke => {
                warn!("Unsure if opcode {:?} implies a panicking block.", opcode);
                return None;
            }
            InstructionOpcode::CallBr => {
                warn!("Unsure if opcode {:?} implies a panicking block.", opcode);
                return None;
            }
            InstructionOpcode::Resume => {
                warn!("Unsure if opcode {:?} implies a panicking block.", opcode);
                return None;
            }
            InstructionOpcode::CatchSwitch => {
                warn!("Unsure if opcode {:?} implies a panicking block.", opcode);
                return None;
            }
            InstructionOpcode::CatchRet => {
                warn!("Unsure if opcode {:?} implies a panicking block.", opcode);
                return None;
            }
            InstructionOpcode::CleanupRet => {
                warn!("Unsure if opcode {:?} implies a panicking block.", opcode);
                return None;
            }
            InstructionOpcode::Unreachable => {
                return Some(true);
            }
            _ => {
                warn!("Opcode {:?} is not supported as a terminator for panic detection", opcode);
                return None;
            }
        }
    } else {
        warn!("\tNo terminator found for panic detection");
        return None;
    }
}


pub fn get_entry_condition<'a>(
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
pub fn codegen_basic_block(node: String, forward_edges: &EdgeSet, backward_edges: &EdgeSet, function: &FunctionValue, solver: &Solver, _namespace: &String) -> () {
    let mut successors = &HashSet::<String>::new();
    successors = forward_edges.get(&node).unwrap_or(successors);
    let mut node_var = if successors.len() == 0 {
        // handle panic (conceptually assign panic var and assert)
        // (panic <- bool_var) => !panic
        // equivalent to !panic
        // Default is_panic true to be cautious in our analysis
        let is_panic = is_panic_block(&get_basic_block_by_name(&function, &node)).unwrap_or(true);
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
        node_var = codegen_instruction(&node, node_var, current_instruction, function, solver, _namespace);
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

    let named_node_var = Bool::new_const(solver.get_context(), String::from(&node));
    solver.assert(&named_node_var._eq(&node_var));
}