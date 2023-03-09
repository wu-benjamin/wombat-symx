use std::collections::{HashMap};

use tracing::{error};

use inkwell::module::{Module as InkwellModule};
use inkwell::context::Context;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{PhiValue, InstructionOpcode, InstructionValue, BasicValueEnum, IntValue, AnyValue};

use super::pretty_print::pretty_print_function;

pub const MAGIC_STRING_TERMINATOR: &str = "WHYISLIFEPAIN";

struct Assignment<'a> {
    lvalue: String,
    rvalue: BasicValueEnum<'a>
}


fn get_instructions(bb: BasicBlock) -> Vec<InstructionValue> {
    let mut instructions = Vec::new();
    let mut next_instruction = bb.get_first_instruction();
    while let Some(current_instruction) = next_instruction {
        instructions.push(current_instruction);
        next_instruction = current_instruction.get_next_instruction();
    }
    return instructions;
}


pub fn resolve_phi<'a>(context: &Context, module: &InkwellModule) -> () {
    let mut next_function = module.get_first_function();
    let builder = context.create_builder();
    while let Some(current_function) = next_function {
        if current_function.get_name().to_str().unwrap().contains("test_") && !current_function.get_name().to_str().unwrap().contains("main") {
            pretty_print_function(&current_function, "");
        }
        for bb in current_function.get_basic_blocks() {
            let instructions = get_instructions(bb);
            
            let mut phi_predecessor_assignments = HashMap::<BasicBlock, Vec<Assignment>>::new();

            // Add lvalue and rvalue pairs for each phi predecessor 

            for instruction in instructions {
                if instruction.get_opcode() == InstructionOpcode::Phi {
                    let phi_instruction: PhiValue = instruction.try_into().unwrap();

                    for incoming_index in 0..phi_instruction.count_incoming() {
                        let (phi_rvalue, phi_predecessor) = phi_instruction.get_incoming(incoming_index).unwrap();
                        if !phi_predecessor_assignments.contains_key(&phi_predecessor) {
                            phi_predecessor_assignments.insert(phi_predecessor, Vec::<Assignment>::new());
                        }

                        let value_llvm_str = &phi_instruction.print_to_string();
                        let value_str = value_llvm_str.to_str().unwrap();
                        let start_index = value_str.find("%").unwrap();
                        let end_index = value_str[start_index..].find(|c: char| c == '"' || c == ' ' || c == ',').unwrap_or(value_str[start_index..].len()) + start_index;
                        let var_name = String::from(&value_str[start_index..end_index]);

                        let assignment = Assignment{
                            lvalue: var_name,
                            rvalue: phi_rvalue
                        };

                        phi_predecessor_assignments.get_mut(&phi_predecessor).unwrap().push(assignment);
                    }
                    instruction.remove_from_basic_block();
                }
            }
            // For each phi predecessor, create edge node to current node
            // Add assignment instructions to edge node
            for (phi_predecessor, assignments) in &phi_predecessor_assignments {
                let phi_predecessor_name = phi_predecessor.get_name().to_str().unwrap();
                let current_node_name = bb.get_name().to_str().unwrap();
                let edge_node_name = format!("{}_{}", phi_predecessor_name, current_node_name);
                let edge_node = context.append_basic_block(current_function, edge_node_name.as_str());
                builder.position_at_end(edge_node);
                for assignment in assignments {
                    let alloca = builder.build_alloca(assignment.rvalue.get_type(), &assignment.lvalue);
                    alloca.set_name(&format!("{}{}", &assignment.lvalue[1..], MAGIC_STRING_TERMINATOR));
                    builder.build_store(alloca, assignment.rvalue);
                    // let i64_type = context.i64_type();
                    // let i64_zero = i64_type.const_int(7, false);
                    // builder.build_int_add(assignment.rvalue.into_int_value(), i64_zero, &assignment.lvalue[1..]);
                }
                builder.build_unconditional_branch(bb);
                
                let phi_predecessor_terminator = phi_predecessor.get_last_instruction().unwrap();
                phi_predecessor_terminator.remove_from_basic_block();
                builder.position_at_end(*phi_predecessor);

                if phi_predecessor_terminator.get_opcode() == InstructionOpcode::Br {
                    if phi_predecessor_terminator.get_num_operands() == 1 {
                        let mut dest_block = phi_predecessor_terminator.get_operand(0).unwrap().right().unwrap();
                        if dest_block == bb {
                            dest_block = edge_node;
                        }
                        builder.build_unconditional_branch(dest_block);
                    } else if phi_predecessor_terminator.get_num_operands() == 3 {
                        // TODO: Check order of then and else block
                        let mut dest_then_block = phi_predecessor_terminator.get_operand(2).unwrap().right().unwrap();
                        if dest_then_block == bb {
                            dest_then_block = edge_node;
                        }
                        let mut dest_else_block = phi_predecessor_terminator.get_operand(1).unwrap().right().unwrap();
                        if dest_else_block == bb {
                            dest_else_block = edge_node;
                        }
                        builder.build_conditional_branch(phi_predecessor_terminator.get_operand(0).unwrap().left().unwrap().into_int_value(), dest_then_block, dest_else_block);
                    } else {
                        error!("Invalid number of operands {:?} for phi_predecessor_terminator {:?}", phi_predecessor_terminator.get_num_operands(), phi_predecessor_terminator);
                    }
                } else if phi_predecessor_terminator.get_opcode() == InstructionOpcode::Switch {
                    let else_value = phi_predecessor_terminator.get_operand(0).unwrap().left().unwrap().into_int_value();
                    let else_block = phi_predecessor_terminator.get_operand(1).unwrap().right().unwrap();
                    let mut cases = Vec::<(IntValue, BasicBlock)>::new();
                    for i in 2..phi_predecessor_terminator.get_num_operands() {
                        if i % 2 == 0 {
                            let mut conditional_block = phi_predecessor_terminator.get_operand(i + 1).unwrap().right().unwrap();
                            if conditional_block == bb {
                                conditional_block = edge_node;
                            }
                            let case = (phi_predecessor_terminator.get_operand(i).unwrap().left().unwrap().into_int_value(), conditional_block);
                            cases.push(case);
                        }
                    }
                    builder.build_switch(else_value, else_block, &cases);
                } else {
                    error!("Unsupported phi_predecessor_terminator opcode {:?}", phi_predecessor_terminator.get_opcode());
                }
            }
        }
        next_function = current_function.get_next_function();
    }
}