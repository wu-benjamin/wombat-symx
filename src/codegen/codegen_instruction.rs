use tracing::{warn};

use inkwell::module::{Module as InkwellModule};
use inkwell::IntPredicate;
use inkwell::values::{InstructionOpcode, InstructionValue};

use z3::Solver;
use z3::ast::{Ast, Bool, Int};

use crate::codegen::codegen_call::{codegen_call};
use crate::utils::var_utils::get_var_name;


fn get_field_to_extract(instruction: &InstructionValue) -> String {
    let instruction_string = instruction.to_string();
    return String::from(&instruction_string[instruction_string.rfind(" ").unwrap()+1..instruction_string.rfind("\"").unwrap()]);
}


pub fn codegen_instruction<'a>(
    module: &InkwellModule,
    mut node_var: Bool<'a>,
    instruction: InstructionValue,
    solver: &'a Solver,
    namespace: &'a str,
    call_stack: &str,
    return_register: &str
) -> Bool<'a> {
    let opcode = instruction.get_opcode();
    match &opcode {
        InstructionOpcode::Unreachable => {
            // NO-OP
        }
        InstructionOpcode::Call => {
            // Code gen function with return to POST_NODE and request to assign return value to new return register
            node_var = codegen_call(module, node_var, instruction, solver, namespace, call_stack);
        }
        InstructionOpcode::Return => {
            if instruction.get_num_operands() == 0 {
                // NO-OP
            } else if instruction.get_num_operands() == 1 {    
                let operand = instruction.get_operand(0).unwrap().left().unwrap();        
                let rvalue_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), solver, namespace);
                if operand.get_type().to_string().eq("\"i1\"") {
                    let lvalue_var = Bool::new_const(
                        solver.get_context(),
                        return_register
                    );
                    let rvalue_var = Bool::new_const(
                        solver.get_context(),
                        rvalue_name
                    );                            
                    let assignment = lvalue_var._eq(&rvalue_var);
                    node_var = assignment.implies(&node_var);
                } else if operand.get_type().is_int_type() {
                        let lvalue_var = Int::new_const(
                        solver.get_context(),
                        return_register
                    );
                    let rvalue_var = Int::new_const(
                        solver.get_context(),
                        rvalue_name
                    );                            
                    let assignment = lvalue_var._eq(&rvalue_var);
                    node_var = assignment.implies(&node_var);
                } else {
                    warn!("Currently unsupported type {:?} for return {:?}", operand.get_type().to_string(), instruction);
                }
            } else {
                warn!("Currently unsupported number of operands {:?} for return {:?}", instruction.get_num_operands(), instruction);
            }
        }
        InstructionOpcode::Switch => {
            // NO-OP
        }
        InstructionOpcode::Load => {
            // TODO: Support non-int types here
            let operand = instruction.get_operand(0).unwrap().left().unwrap();
            if !instruction.get_type().is_int_type() {
                warn!("Currently unsupported type {:?} for load operand", instruction.get_type().to_string())
            }
            let lvalue_var_name = get_var_name(&instruction, &solver, namespace);
            let rvalue_var_name = get_var_name(&operand, &solver, namespace);
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
            let operand1 = instruction.get_operand(0).unwrap().left().unwrap();
            if !operand1.get_type().is_int_type() {
                warn!("Currently unsupported type {:?} for store operand", operand1.get_type().to_string())
            }
            let operand2 = instruction.get_operand(1).unwrap().left().unwrap().into_pointer_value();
            
            let lvalue_var_name = get_var_name(&operand1, &solver, namespace);
            let rvalue_var_name = get_var_name(&operand2, &solver, namespace);
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
            let operand1_var_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), &solver, namespace);
            let operand2_var_name = get_var_name(&instruction.get_operand(1).unwrap().left().unwrap(), &solver, namespace);
            if !instruction.get_type().to_string().eq("\"i1\"") {
                warn!("Currently unsupported type {:?} for xor operand", instruction.get_type().to_string());
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
            let lvalue_var_name = get_var_name(&instruction, &solver, namespace);
            let lvalue_var = Bool::new_const(
                solver.get_context(),
                lvalue_var_name
            );
            let assignment = lvalue_var._eq(&rvalue_var);
            node_var = assignment.implies(&node_var);
        }
        InstructionOpcode::ICmp => {
            let lvalue_var_name = get_var_name(&instruction, &solver, namespace);
            let lvalue_var = Bool::new_const(solver.get_context(), lvalue_var_name);
            let operand1 = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), &solver, namespace);
            let operand2 = get_var_name(&instruction.get_operand(1).unwrap().left().unwrap(), &solver, namespace);
            let rvalue_operation;
            

            // Split by the sub-instruction (denoting the type of comparison)
            // TODO: can signed & unsigned comparisons be combined?
            let icmp_type = instruction.get_icmp_predicate().unwrap();
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
            let lvalue_var_name = get_var_name(&instruction, &solver, namespace);
            let operand = instruction.get_operand(0).unwrap().left().unwrap();
            let rvalue_var_name = format!("{}.{}", get_var_name(&operand, &solver, namespace), get_field_to_extract(&instruction));
            if instruction.get_type().to_string().eq("\"i1\"") {
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
            } else if instruction.get_type().is_int_type() {
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
                warn!("Currently unsupported type {:?} for extract value", operand.get_type().to_string())
            } 
        }
        InstructionOpcode::Alloca => {
            // NO-OP
        }
        InstructionOpcode::Phi => {
            warn!("Phi instruction should be resolved and not exist during instruction codegen")
        }
        InstructionOpcode::Trunc => {
            if instruction.get_type().to_string().eq("\"i1\"") {
                let lvalue_var_name = get_var_name(&instruction, &solver, namespace);
                let operand_var_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), &solver, namespace);
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
            } else {
                warn!("Type {} is not a supported target type for the Trunc instruction!", instruction.get_type().to_string());
            }
        }
        InstructionOpcode::Select => {
            let discriminant = instruction.get_operand(0).unwrap().left().unwrap();
            let discriminant_name = get_var_name(&discriminant, &solver, namespace);
            let operand_1_var_name = get_var_name(&instruction.get_operand(1).unwrap().left().unwrap(), &solver, namespace);
            let operand_2_var_name = get_var_name(&instruction.get_operand(2).unwrap().left().unwrap(), &solver, namespace);
            if !discriminant.get_type().to_string().eq("\"i1\"") {
                warn!("Currently unsupported type {:?} for select discriminant", discriminant.get_type().to_string());
            }
            let discriminant_var = Bool::new_const(
                solver.get_context(),
                discriminant_name
            );
            if instruction.get_type().to_string().eq("\"i1\"") {
                let operand_1_var = Bool::new_const(
                    solver.get_context(),
                    operand_1_var_name
                );
                let operand_2_var = Bool::new_const(
                    solver.get_context(),
                    operand_2_var_name
                );                            
                let select_1 = discriminant_var.implies(&Bool::new_const(solver.get_context(), get_var_name(&instruction, &solver, namespace))._eq(&operand_1_var));
                let select_2 = discriminant_var.not().implies(&Bool::new_const(solver.get_context(), get_var_name(&instruction, &solver, namespace))._eq(&operand_2_var));
                node_var = Bool::and(solver.get_context(), &[&select_1.implies(&node_var), &select_2.implies(&node_var)]);
            } else if instruction.get_type().is_int_type() {
                let operand_1_var = Int::new_const(
                    solver.get_context(),
                    operand_1_var_name
                );
                let operand_2_var = Int::new_const(
                    solver.get_context(),
                    operand_2_var_name
                );                            
                let select_1 = discriminant_var.implies(&Int::new_const(solver.get_context(), get_var_name(&instruction, &solver, namespace))._eq(&operand_1_var));
                let select_2 = discriminant_var.not().implies(&Int::new_const(solver.get_context(), get_var_name(&instruction, &solver, namespace))._eq(&operand_2_var));
                let assignment = Bool::and(solver.get_context(), &[&select_1, &select_2]);
                node_var = assignment.implies(&node_var);
            } else {
                warn!("Currently unsupported type {:?} for select", instruction.get_type().to_string());
            }
        }
        InstructionOpcode::ZExt => {
            if instruction.get_operand(0).unwrap().left().unwrap().get_type().to_string().eq("\"i1\"") {
                let lvalue_var_name = get_var_name(&instruction, &solver, namespace);
                let operand_var_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), &solver, namespace);
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
            } else if instruction.get_operand(0).unwrap().left().unwrap().get_type().is_int_type() {
                let lvalue_var_name = get_var_name(&instruction, &solver, namespace);
                let operand_var_name = get_var_name(&instruction.get_operand(0).unwrap().left().unwrap(), &solver, namespace);
                let lvalue_var = Int::new_const(
                    solver.get_context(),
                    lvalue_var_name
                );
                let operand_var = Int::new_const(
                    solver.get_context(),
                    operand_var_name
                );
                let assignment = lvalue_var._eq(&operand_var);
                node_var = assignment.implies(&node_var);
            } else {
                warn!("Type {} is not a supported target type for the ZExt instruction!", instruction.get_type().to_string());
            }
        }
        _ => {
            warn!("Opcode {:?} is not supported as a statement for code gen", opcode);
        }
    }
    return node_var;
}