use std::collections::{HashMap};
use std::path::Path;

use tracing::{debug, error};

use rustc_demangle::demangle;

use inkwell::context::Context as InkwellContext;
use inkwell::module::{Module as InkwellModule};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::values::{FunctionValue, InstructionOpcode, AnyValue, PointerValue};

use z3::{Config, Solver, SatResult};
use z3::Context as Z3Context;
use z3::ast::{Int, Bool};

use crate::codegen_function::codegen_function;
use crate::get_var_name::get_var_name;
use crate::pretty_print::{print_file_functions};


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

fn get_inkwell_module<'a>(context: &'a InkwellContext, file_name: &String) -> Option<InkwellModule<'a>> {
    let path = Path::new(&file_name);
    if !path.is_file() {
        error!("{:?} is an invalid file. Please provide a valid file.", file_name);
        return None;
    }

    let buffer = MemoryBuffer::create_from_file(&path).unwrap();
    let module_result = InkwellModule::parse_bitcode_from_buffer(&buffer, context);

    // Check the module is from a valid bytecode file
    if module_result.is_err() {
        error!("{:?} is not a valid LLVM bitcode file. Please pass in a valid bc file.\nThe module_result is below:\n{:?}", file_name, module_result);
        return None;
    }
    let module = module_result.unwrap();
    return Some(module);
}

fn get_module_name_from_file_name(file_name: &String) -> String {
    let mut start_index = 0;
    if let Some(last_slash_index) = file_name.rfind("/") {
        start_index = last_slash_index + 1;
    }
    let end_index = file_name.rfind(".").unwrap_or(file_name.len());
    return file_name[start_index..end_index].to_string();
}


fn get_function_argument_names<'a>(function: &'a FunctionValue) -> HashMap<String, String> {
    let mut arg_names = HashMap::<String, String>::new();
    for param in &function.get_params() {
        debug!("Func param instr: {:?}", param);
        if param.get_name().len() == 0 {
            // Var name is empty, find in start basic block
            let alias_name = &get_var_name(&param.as_any_value_enum(), &Solver::new(&Z3Context::new(&Config::new())));
            let start_block_option = function.get_first_basic_block();
            if start_block_option.is_none() {
                return arg_names;
            }
            let start_block =start_block_option.unwrap();
            let mut instr = start_block.get_first_instruction();
            while instr.is_some() {
                if instr.unwrap().get_opcode() == InstructionOpcode::Store && alias_name.to_string() == get_var_name(&instr.unwrap().as_any_value_enum(), &Solver::new(&Z3Context::new(&Config::new()))) {
                    let arg_name = get_var_name(&instr.unwrap().get_operand(1).unwrap().left().unwrap().as_any_value_enum(), &Solver::new(&Z3Context::new(&Config::new())));
                    arg_names.insert(arg_name[1..].to_string(), alias_name.to_string());
                }
                instr = instr.unwrap().get_next_instruction();
            }
        } else {
            let arg_name = &param.get_name();
            arg_names.insert(arg_name.to_string(), format!("{}{}", "%", arg_name.to_string()));
        }
    }

    debug!("Function arg names: {:?}", arg_names);
    arg_names
}


fn get_all_function_argument_names(module: &InkwellModule) -> HashMap<String, HashMap<String, String>> {
    let mut all_func_arg_names = HashMap::<String, HashMap<String, String>>::new();

    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        let current_full_function_name = get_function_name(&current_function.as_global_value().as_pointer_value());
        all_func_arg_names.insert(current_full_function_name, get_function_argument_names(&current_function));
        next_function = current_function.get_next_function();
    }
    return all_func_arg_names;
}


pub fn get_function_name(function: &PointerValue) -> String {
    return demangle(&function.get_name().to_str().unwrap()).to_string();
}


fn get_function_by_name<'a>(module: &'a InkwellModule, target_function_name_prefix: &String) -> Option<FunctionValue<'a>> {
    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        let current_full_function_name = get_function_name(&current_function.as_global_value().as_pointer_value());
        if current_full_function_name.find(target_function_name_prefix).is_some() {
            return Some(current_function);
        }
        next_function = current_function.get_next_function();
    }
    return None;
}


fn convert_to_dsa<'a>(module: &InkwellModule) -> () {
    let pass_manager_builder = PassManagerBuilder::create();
    let pass_manager = PassManager::create(module);
    pass_manager.add_promote_memory_to_register_pass();
    pass_manager_builder.populate_function_pass_manager(&pass_manager);

    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        pass_manager.run_on(&current_function);
        next_function = current_function.get_next_function();
    }
}


pub fn symbolic_execution(file_name: &String, function_name: &String) -> Option<bool> {
    let context = InkwellContext::create();
    let module_result = get_inkwell_module(&context, file_name);
    if module_result.is_none() {
        return None;
    }

    let module = module_result.unwrap();
    let module_name = get_module_name_from_file_name(file_name);
    let target_function_name_prefix = format!("{}::{}", module_name, function_name);
    let namespace = String::from("");

    // Initialize the Z3 and Builder objects
    let cfg = Config::new();
    let ctx = Z3Context::new(&cfg);
    let solver = Solver::new(&ctx);

    // Save function argument names before removing store/alloca instructions
    let all_func_arg_names = get_all_function_argument_names(&module);

    // Convert to dynamic single assignment form (DSA)

    print_file_functions(&module);

    convert_to_dsa(&module);

    let function_option = get_function_by_name(&module, &target_function_name_prefix);
    if function_option.is_none() {
        return None;
    }
    let function = function_option.unwrap();

    let func_arg_names_option = all_func_arg_names.get(&get_function_name(&function.as_global_value().as_pointer_value()));
    if func_arg_names_option.is_none() {
        return None;
    }
    let func_arg_names = func_arg_names_option.unwrap();

    codegen_function(&function, &solver, &namespace);

    let start_node = function.get_first_basic_block().unwrap();
    // TODO: Namespace
    let start_node_var_name = start_node.get_name().to_str().unwrap();
    let start_node_var = Bool::new_const(solver.get_context(), String::from(start_node_var_name));
    solver.assert(&start_node_var.not());

    debug!("{:?}", solver);

    // Attempt resolving the model (and obtaining the respective arg values if panic found)
    let satisfiability = solver.check();

    let is_confirmed_safe = satisfiability == SatResult::Unsat;
    let is_confirmed_unsafe = satisfiability == SatResult::Sat;
    println!("\nFunction safety: {}", if is_confirmed_safe {"safe"} else if is_confirmed_unsafe {"unsafe"} else {"unknown"});

    // Exhibit a pathological input if the function is unsafe
    if is_confirmed_unsafe {
        let model = solver.get_model().unwrap();
        debug!("\n{:?}", model);
        println!("\nArgument values:");
        for (arg_name, alias_name) in func_arg_names {
            // TODO: Namespace
            // TODO: Support non-int params
            let arg_z3 = Int::new_const(solver.get_context(), alias_name.as_str());
            println!("\t{:?} = {:?}", &arg_name, model.eval(&arg_z3, true).unwrap());
        };
    }
    return Some(is_confirmed_safe);
}