use std::path::Path;

use tracing::{debug, error};

use inkwell::context::Context as InkwellContext;
use inkwell::module::{Module as InkwellModule};
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::passes::{PassManager, PassManagerBuilder};

use z3::{Config, Solver, SatResult};
use z3::Context as Z3Context;
use z3::ast::{Int, Bool};

use crate::codegen_function::codegen_function;
use crate::function_utils::{get_function_name, get_function_by_name, get_all_function_argument_names};
use crate::pretty_print::{print_file_functions, pretty_print_function};


const MAIN_FUNCTION_NAMESPACE: &str = "wombat_symx_";


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

    pretty_print_function(&function);

    codegen_function(&function, &solver, MAIN_FUNCTION_NAMESPACE);

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
        for (arg_name, z3_name) in func_arg_names {
            // TODO: Support non-int params
            let arg_z3 = Int::new_const(solver.get_context(), z3_name.as_str());
            let arg_name_without_namespace = &arg_name[MAIN_FUNCTION_NAMESPACE.len()..];
            let arg_name_without_namespace_and_percent = arg_name_without_namespace.replace("%", "");
            println!("\t{:?} = {:?}", &arg_name_without_namespace_and_percent, model.eval(&arg_z3, true).unwrap());
        };
    }
    return Some(is_confirmed_safe);
}