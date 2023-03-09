use std::collections::HashMap;

use tracing::error;

use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::module::Module as InkwellModule;
use inkwell::values::{
    AnyValue, BasicValueEnum, InstructionOpcode, InstructionValue, IntValue, PhiValue, PointerValue,
};

struct Assignment<'a> {
    lvalue: String,
    rvalue: BasicValueEnum<'a>,
}

fn get_instructions(bb: BasicBlock) -> Vec<InstructionValue> {
    let mut instructions = Vec::new();
    let mut next_instruction = bb.get_first_instruction();
    while let Some(current_instruction) = next_instruction {
        instructions.push(current_instruction);
        next_instruction = current_instruction.get_next_instruction();
    }
    instructions
}

pub fn resolve_phi_to_dsa(context: &Context, module: &InkwellModule) {
    let mut next_function = module.get_first_function();
    let builder = context.create_builder();
    while let Some(current_function) = next_function {
        for cur_node in current_function.get_basic_blocks() {
            let instructions = get_instructions(cur_node);

            let mut phi_predecessor_assignments = HashMap::<BasicBlock, Vec<Assignment>>::new();
            let mut allocas = HashMap::<String, PointerValue>::new();

            // Add lvalue and rvalue pairs for each phi predecessor

            for instruction in instructions {
                if instruction.get_opcode() == InstructionOpcode::Phi {
                    let phi_instruction: PhiValue = instruction.try_into().unwrap();

                    // Adapted from function get_var_name of var_utils.rs
                    // No need for solver as var_name cannot represent a constant here
                    // No need for namespace since there is no notion of instances of function calls in this preprocessing step
                    let value_llvm_str = &phi_instruction.print_to_string();
                    let value_str = value_llvm_str.to_str().unwrap();
                    let start_index = value_str.find('%').unwrap() + 1;
                    let end_index = value_str[start_index..]
                        .find(|c: char| c == '"' || c == ' ' || c == ',')
                        .unwrap_or_else(|| value_str[start_index..].len())
                        + start_index;
                    let var_name = String::from(&value_str[start_index..end_index]);

                    for incoming_index in 0..phi_instruction.count_incoming() {
                        let (phi_rvalue, phi_predecessor) =
                            phi_instruction.get_incoming(incoming_index).unwrap();
                        phi_predecessor_assignments
                            .entry(phi_predecessor)
                            .or_insert_with(Vec::<Assignment>::new);

                        let assignment = Assignment {
                            lvalue: var_name.clone(),
                            rvalue: phi_rvalue,
                        };

                        phi_predecessor_assignments
                            .get_mut(&phi_predecessor)
                            .unwrap()
                            .push(assignment);
                    }

                    // Save required information before removing phi instruction from the basic block
                    let phi_instruction_type = phi_instruction.as_basic_value().get_type();
                    // Instruction has opcode Phi so is non-terminal.
                    // Since all instructions have a terminator, there is at least one more instruction after the phi instruction.
                    let next_instruction = instruction.get_next_instruction().unwrap();

                    instruction.remove_from_basic_block();

                    // Allocate memory to store resolved value of phi instruction
                    builder.position_at_end(current_function.get_first_basic_block().unwrap());
                    if current_function
                        .get_first_basic_block()
                        .unwrap()
                        .get_first_instruction()
                        .is_some()
                    {
                        builder.position_before(
                            &current_function
                                .get_first_basic_block()
                                .unwrap()
                                .get_first_instruction()
                                .unwrap(),
                        );
                    }
                    let alloca =
                        builder.build_alloca(phi_instruction_type, &format!("{}_ptr", var_name));
                    allocas.insert(var_name.clone(), alloca);

                    // Load resolved value of phi instruction
                    builder.position_before(&next_instruction);
                    builder.build_load(alloca, &var_name);
                }
            }

            for (phi_predecessor, assignments) in &phi_predecessor_assignments {
                // For each phi predecessor, create edge node to current node
                // Add assignment instructions to edge node
                let phi_predecessor_name = phi_predecessor.get_name().to_str().unwrap();
                let current_node_name = cur_node.get_name().to_str().unwrap();
                let edge_node_name = format!("{}_{}", phi_predecessor_name, current_node_name);
                let edge_node =
                    context.append_basic_block(current_function, edge_node_name.as_str());
                builder.position_at_end(edge_node);
                for assignment in assignments {
                    builder
                        .build_store(*allocas.get(&assignment.lvalue).unwrap(), assignment.rvalue);
                }
                builder.build_unconditional_branch(cur_node);

                // Replace cur_node with edge_node in terminator of each phi_predecessor
                let phi_predecessor_terminator = phi_predecessor.get_last_instruction().unwrap();
                builder.position_at_end(*phi_predecessor);
                if phi_predecessor_terminator.get_opcode() == InstructionOpcode::Br {
                    if phi_predecessor_terminator.get_num_operands() == 1 {
                        phi_predecessor_terminator.remove_from_basic_block();
                        builder.build_unconditional_branch(edge_node);
                    } else if phi_predecessor_terminator.get_num_operands() == 3 {
                        let mut predecessor_dest_then_block = phi_predecessor_terminator
                            .get_operand(2)
                            .unwrap()
                            .right()
                            .unwrap();
                        let mut predecessor_dest_else_block = phi_predecessor_terminator
                            .get_operand(1)
                            .unwrap()
                            .right()
                            .unwrap();
                        if predecessor_dest_then_block == cur_node {
                            predecessor_dest_then_block = edge_node;
                        } else {
                            predecessor_dest_else_block = edge_node;
                        }
                        let condition = phi_predecessor_terminator
                            .get_operand(0)
                            .unwrap()
                            .left()
                            .unwrap()
                            .into_int_value();
                        phi_predecessor_terminator.remove_from_basic_block();
                        builder.build_conditional_branch(
                            condition,
                            predecessor_dest_then_block,
                            predecessor_dest_else_block,
                        );
                    } else {
                        error!(
                            "Invalid number of operands {:?} for phi_predecessor_terminator {:?}",
                            phi_predecessor_terminator.get_num_operands(),
                            phi_predecessor_terminator
                        );
                    }
                } else if phi_predecessor_terminator.get_opcode() == InstructionOpcode::Switch {
                    let predecessor_dest_else_value = phi_predecessor_terminator
                        .get_operand(0)
                        .unwrap()
                        .left()
                        .unwrap()
                        .into_int_value();
                    let predecessor_dest_else_block = phi_predecessor_terminator
                        .get_operand(1)
                        .unwrap()
                        .right()
                        .unwrap();
                    let mut predecessor_dest_cases = Vec::<(IntValue, BasicBlock)>::new();
                    for i in 2..phi_predecessor_terminator.get_num_operands() {
                        if i % 2 == 0 {
                            let mut predecessor_dest_conditional_block = phi_predecessor_terminator
                                .get_operand(i + 1)
                                .unwrap()
                                .right()
                                .unwrap();
                            if predecessor_dest_conditional_block == cur_node {
                                predecessor_dest_conditional_block = edge_node;
                            }
                            let predecessor_dest_case = (
                                phi_predecessor_terminator
                                    .get_operand(i)
                                    .unwrap()
                                    .left()
                                    .unwrap()
                                    .into_int_value(),
                                predecessor_dest_conditional_block,
                            );
                            predecessor_dest_cases.push(predecessor_dest_case);
                        }
                    }
                    phi_predecessor_terminator.remove_from_basic_block();
                    builder.build_switch(
                        predecessor_dest_else_value,
                        predecessor_dest_else_block,
                        &predecessor_dest_cases,
                    );
                } else {
                    error!(
                        "Unsupported phi_predecessor_terminator opcode {:?}",
                        phi_predecessor_terminator.get_opcode()
                    );
                }
            }
        }

        next_function = current_function.get_next_function();
    }
}
