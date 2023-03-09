use std::collections::{HashMap};

use inkwell::types::BasicTypeEnum;
// use tracing::{debug};

use rustc_demangle::demangle;

use inkwell::module::{Module as InkwellModule};
use inkwell::values::{FunctionValue, InstructionOpcode, AnyValue, PointerValue};

use z3::{Solver};

use crate::utils::var_utils::get_var_name;


trait Named {
    fn get_name(&self) -> String;
}

impl Named for inkwell::values::BasicValueEnum<'_> {
    fn get_name(&self) -> String {
        if self.is_array_value() {
            self.into_array_value().get_name().to_str().unwrap().to_string()
        } else if self.is_int_value() {
            self.into_int_value().get_name().to_str().unwrap().to_string()
        } else if self.is_float_value() {
            self.into_float_value().get_name().to_str().unwrap().to_string()
        } else if self.is_pointer_value() {
            self.into_pointer_value().get_name().to_str().unwrap().to_string()
        } else if self.is_struct_value() {
            self.into_struct_value().get_name().to_str().unwrap().to_string()
        } else {
            self.into_vector_value().get_name().to_str().unwrap().to_string()
        }
    }
}

// Returns a map of source code function argument names to Z3 module variable names
pub fn get_function_argument_names<'a>(function: FunctionValue<'a>, solver: &Solver, namespace: &str) -> Vec<(String, String, BasicTypeEnum<'a>)> {
    let mut arg_names = Vec::<(String, String, BasicTypeEnum)>::new();
    for param in &function.get_params() {
        // debug!("Func param instr: {:?}", param);
        if param.get_name().is_empty() {
            // Var name is empty, find in start basic block
            let alias_name = &get_var_name(&param.as_any_value_enum(), solver, namespace);
            
            let start_block_option = function.get_first_basic_block();
            if start_block_option.is_none() {
                return arg_names;
            }
            let start_block =start_block_option.unwrap();
            let mut instr = start_block.get_first_instruction();
            while instr.is_some() {
                if instr.unwrap().get_opcode() == InstructionOpcode::Store && *alias_name == get_var_name(&instr.unwrap().as_any_value_enum(), solver, namespace) {
                    let arg_name = get_var_name(&instr.unwrap().get_operand(1).unwrap().left().unwrap().as_any_value_enum(), solver, namespace);
                    arg_names.push((arg_name.to_string(), alias_name.to_string(), param.get_type()));
                }
                instr = instr.unwrap().get_next_instruction();
            }
        } else {
            let arg_name_string = format!("{}{}{}", namespace, "%", &param.get_name());
            let arg_name = arg_name_string.to_string();
            arg_names.push((arg_name.to_string(), arg_name.to_string(), param.get_type()));
        }
    }

    // debug!("Function arg names: {:?}", arg_names);
    arg_names
}


pub fn get_all_function_argument_names<'a>(module: &'a InkwellModule, solver: &'a Solver, namespace: &'a str) -> HashMap<String, Vec<(String, String, BasicTypeEnum<'a>)>> {
    let mut all_func_arg_names = HashMap::<String, Vec<(String, String, BasicTypeEnum)>>::new();

    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        let current_full_function_name = get_function_name(&current_function.as_global_value().as_pointer_value());
        let function_argument_names = get_function_argument_names(current_function, solver, namespace);
        all_func_arg_names.insert(current_full_function_name, function_argument_names);
        next_function = current_function.get_next_function();
    }
    all_func_arg_names
}


pub fn get_function_name(function: &PointerValue) -> String {
    return demangle(function.get_name().to_str().unwrap()).to_string();
}


pub fn get_function_by_name<'a>(module: &'a InkwellModule, target_function_name_prefix: &String) -> Option<FunctionValue<'a>> {
    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        let current_full_function_name = get_function_name(&current_function.as_global_value().as_pointer_value());
        if current_full_function_name.contains(target_function_name_prefix) {
            return Some(current_function);
        }
        next_function = current_function.get_next_function();
    }
    None
}
