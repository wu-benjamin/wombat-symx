use std::fs;
use std::path::Path;
use std::process::Command;

use tracing::{debug, warn, error};

use inkwell::context::Context as InkwellContext;
use inkwell::module::{Module as InkwellModule};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::passes::{PassManager, PassManagerBuilder};

use z3::{Config, Solver, SatResult};
use z3::Context as Z3Context;
use z3::ast::{Int, Bool, Ast};

use crate::codegen_function::codegen_function;
use crate::function_utils::{get_function_name, get_function_by_name, get_all_function_argument_names};
use crate::get_var_name::get_var_name;
use crate::pretty_print::{print_file_functions};


// pub const MAIN_FUNCTION_NAMESPACE: &str = "wombat_symx_";
pub const MAIN_FUNCTION_NAMESPACE: &str = "";
pub const COMMON_END_NODE: &str = "common_end_node";
pub const PANIC_VAR_NAME: &str = "is_panic";
pub const MAIN_FUNCTION_RETURN_REGISTER: &str = "wombat_symx_return_register";


struct FileDropper<'a> {
    file_name: &'a String,
}

impl Drop for FileDropper<'_> {
    fn drop(&mut self) {
        println!("{}", self.file_name);
        fs::remove_file(self.file_name).expect("Failed to delete file.");
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

pub fn get_module_name_from_file_name(file_name: &String) -> String {
    let mut start_index = 0;
    if let Some(last_slash_index) = file_name.rfind("/") {
        start_index = last_slash_index + 1;
    }
    let end_index = file_name.rfind(".").unwrap_or(file_name.len());
    return file_name[start_index..end_index].to_string();
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

    let bytecode_file_name = format!("{}.bc", &file_name[0..file_name.rfind('.').unwrap_or(file_name.len())]);

    Command::new("rustc")
        .args(["--emit=llvm-bc", &file_name, "-o", &bytecode_file_name])
        .status()
        .expect("Failed to generate bytecode file!");

    let _temp_bc_file_dropper = FileDropper {
        file_name: &bytecode_file_name,
    };

    let module_result = get_inkwell_module(&context, &bytecode_file_name);
    if module_result.is_none() {
        return None;
    }

    let module = module_result.unwrap();
    let module_name = get_module_name_from_file_name(&bytecode_file_name);
    let target_function_name_prefix = format!("{}::{}", module_name, function_name);

    // Initialize the Z3 and Builder objects
    let cfg = Config::new();
    let ctx = Z3Context::new(&cfg);
    let solver = Solver::new(&ctx);

    // Save function argument names before removing store/alloca instructions
    let all_func_arg_names = get_all_function_argument_names(&module, &solver, MAIN_FUNCTION_NAMESPACE);

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

    let call_stack = function.get_name().to_str().unwrap();
    codegen_function(&module, &function, &solver, MAIN_FUNCTION_NAMESPACE, call_stack, COMMON_END_NODE, MAIN_FUNCTION_RETURN_REGISTER);

    // constrain int inputs
    for input in function.get_params() {
        // TODO: Support other input types
        if input.get_type().to_string().eq("\"i1\"") {
            continue;
        } else if input.get_type().to_string().eq("\"i32\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver, MAIN_FUNCTION_NAMESPACE));
            let min_int = Int::from_i64(solver.get_context(), i32::MIN.into());
            let max_int = Int::from_i64(solver.get_context(), i32::MAX.into());
            solver.assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        } else if input.get_type().to_string().eq("\"i64\"") {
            let arg = Int::new_const(&solver.get_context(), get_var_name(&input, &solver, MAIN_FUNCTION_NAMESPACE));
            let min_int = Int::from_i64(solver.get_context(), i64::MIN.into());
            let max_int = Int::from_i64(solver.get_context(), i64::MAX.into());
            solver.assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        }  else {
            warn!("Currently unsuppported type {:?} for input parameter to {}", input.get_type().to_string(), function_name);
        }
    }

    let common_end_node_var = Bool::new_const(solver.get_context(), String::from(COMMON_END_NODE));
    let panic_var = Bool::new_const(solver.get_context(), String::from(PANIC_VAR_NAME));
    solver.assert(&common_end_node_var._eq(&panic_var.not()));

    let start_node = function.get_first_basic_block().unwrap();
    let start_node_var_name = format!("{}{}", MAIN_FUNCTION_NAMESPACE, start_node.get_name().to_str().unwrap());
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
        let mut argument_values = std::vec::Vec::<String>::new();
        for (arg_name, z3_name) in func_arg_names {
            // TODO: Support non-int params
            let arg_name_without_namespace = &arg_name[MAIN_FUNCTION_NAMESPACE.len()..];
            let arg_name_without_namespace_and_percent = arg_name_without_namespace.replace("%", "");
            let model_string = format!("{:?}", model);
            // Find line of model for variable and extract line
            let mut value = &model_string[model_string.find(z3_name).unwrap()..model_string.len()];
            value = &value[value.find("->").unwrap()..value.find("\n").unwrap_or(value.len())];
            let cleaned_value = &value.replace("(", "").replace(")", "").replace(" ", "").replace("->", "");
            println!("\t{:?} = {}", &arg_name_without_namespace_and_percent, cleaned_value);
            argument_values.push(String::from(cleaned_value));
        };

        let mut source_file_content = fs::read_to_string(file_name).unwrap();
        source_file_content = source_file_content.replace("fn main", "fn _main");
        source_file_content = format!("{}\nfn main() {{{}(", source_file_content, function_name);
        for argument_value in argument_values {
            source_file_content = format!("{}{},", source_file_content, argument_value);
        }
        source_file_content = format!("{});}}", source_file_content);
        debug!("{}", source_file_content);

        let mut temp_file_path_base_end_index = 0;
        if file_name.rfind('/').is_some() {
            temp_file_path_base_end_index = file_name.rfind('/').unwrap() + 1;
        }
        let temp_source_file_name = format!("{}temp_wombat_symx_{}", &file_name[0..temp_file_path_base_end_index], &file_name[temp_file_path_base_end_index..file_name.len()]);
        fs::write(&temp_source_file_name, format!("{}", source_file_content)).expect("Failed to write file!");

        let _temp_source_file_dropper = FileDropper {
            file_name: &temp_source_file_name,
        };

        let temp_executable_file_name = &temp_source_file_name[0..temp_source_file_name.rfind('.').unwrap()];

        Command::new("rustc")
        .args([&temp_source_file_name, "-o", &temp_executable_file_name])
        .status()
        .expect("Failed to generate executable file!");

        let _temp_executable_file_dropper = FileDropper {
            file_name: &String::from(temp_executable_file_name),
        };

        println!("{}", std::str::from_utf8(&Command::new(temp_executable_file_name).output().ok().unwrap().stderr).unwrap());
    }

    return Some(is_confirmed_safe);
}