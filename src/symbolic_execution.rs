use std::fs;
use std::path::Path;
use std::process::Command;

use tracing::{debug, error, warn};

use inkwell::context::Context as InkwellContext;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module as InkwellModule;
use inkwell::passes::{PassManager, PassManagerBuilder};

use z3::ast::{Ast, Bool, Int};
use z3::Context as Z3Context;
use z3::{Config, SatResult, Solver};

use crate::codegen::codegen_function::codegen_function;
use crate::utils::function_utils::{get_all_function_argument_names, get_function_by_name, get_function_name};
use crate::utils::pretty_print::print_file_functions;
use crate::utils::resolve_phi_to_dsa::resolve_phi_to_dsa;
use crate::utils::var_utils::{get_min_max_signed_int, get_var_name};

pub const MAIN_FUNCTION_NAMESPACE: &str = "";
pub const COMMON_END_NODE: &str = "common_end_node";
pub const PANIC_VAR_NAME: &str = "is_panic";
pub const MAIN_FUNCTION_RETURN_REGISTER: &str = "wombat_symx_return_register";

struct FileDropper<'a> {
    file_name: &'a String,
}

impl Drop for FileDropper<'_> {
    fn drop(&mut self) {
        fs::remove_file(self.file_name).expect("Failed to delete file.");
    }
}

fn get_inkwell_module<'a>(context: &'a InkwellContext, file_name: &String) -> Option<InkwellModule<'a>> {
    let path = Path::new(&file_name);
    if !path.is_file() {
        error!("{:?} is an invalid file. Please provide a valid file.", file_name);
        return None;
    }

    let buffer = MemoryBuffer::create_from_file(path).unwrap();
    let module_result = InkwellModule::parse_bitcode_from_buffer(&buffer, context);

    // Check the module is from a valid bytecode file
    if module_result.is_err() {
        error!(
            "{:?} is not a valid LLVM bitcode file. Please pass in a valid bc file.\nThe module_result is below:\n{:?}",
            file_name, module_result
        );
        return None;
    }
    let module = module_result.unwrap();
    Some(module)
}

pub fn get_module_name_from_file_name(file_name: &str) -> String {
    let mut start_index = 0;
    if let Some(last_slash_index) = file_name.rfind('/') {
        start_index = last_slash_index + 1;
    }
    let end_index = file_name.rfind('.').unwrap_or(file_name.len());
    file_name[start_index..end_index].to_string()
}

fn convert_to_ssa(module: &InkwellModule) {
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

pub fn symbolic_execution(file_name: &String, function_name: &String, is_benchmark_mode: bool) -> Option<bool> {
    let context = InkwellContext::create();

    let bytecode_file_name = format!("{}.bc", &file_name[0..file_name.rfind('.').unwrap_or(file_name.len())]);

    if !is_benchmark_mode {
        // Benchmark mode skips compilation and assumes user has already compiled bytecode & executable
        Command::new("rustc")
            .args(["--emit=llvm-bc", &file_name, "-o", &bytecode_file_name])
            .status()
            .expect("Failed to generate bytecode file!");
    }

    let _temp_bc_file_dropper = if !is_benchmark_mode { Some(FileDropper { file_name: &bytecode_file_name }) } else { None };

    let module_result = get_inkwell_module(&context, &bytecode_file_name);
    module_result.as_ref()?;

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

    convert_to_ssa(&module);
    resolve_phi_to_dsa(&context, &module);

    let function_option = get_function_by_name(&module, &target_function_name_prefix);
    function_option?;
    let function = function_option.unwrap();

    let func_arg_names_option = all_func_arg_names.get(&get_function_name(&function.as_global_value().as_pointer_value()));
    func_arg_names_option?;
    let func_arg_names = func_arg_names_option.unwrap();

    let call_stack = function.get_name().to_str().unwrap();
    codegen_function(&module, &function, &solver, MAIN_FUNCTION_NAMESPACE, call_stack, COMMON_END_NODE, MAIN_FUNCTION_RETURN_REGISTER);

    // Constrain int inputs
    // Supports signed int types and booleans
    for input in function.get_params() {
        if input.get_type().to_string().eq("\"i1\"") {
            continue;
        } else if input.get_type().is_int_type() {
            let arg = Int::new_const(solver.get_context(), get_var_name(&input, &solver, MAIN_FUNCTION_NAMESPACE));
            let (min_int_val, max_int_val) = get_min_max_signed_int(&input.get_type().to_string().as_str().replace('\"', "")[1..]);
            let min_int = Int::from_i64(solver.get_context(), min_int_val);
            let max_int = Int::from_i64(solver.get_context(), max_int_val);
            solver.assert(&Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
        } else {
            warn!("Currently unsupported type {:?} for input parameter to {}", input.get_type().to_string(), function_name);
        }
    }

    let common_end_node_var = Bool::new_const(solver.get_context(), String::from(COMMON_END_NODE));
    let panic_var = Bool::new_const(solver.get_context(), String::from(PANIC_VAR_NAME));
    solver.assert(&common_end_node_var._eq(&panic_var.not()));

    let start_node = function.get_first_basic_block().unwrap();
    let start_node_var_name = format!("{}{}", MAIN_FUNCTION_NAMESPACE, start_node.get_name().to_str().unwrap());
    let start_node_var = Bool::new_const(solver.get_context(), start_node_var_name);
    solver.assert(&start_node_var.not());

    debug!("{}", format!("\nSolver:\n{:?}", solver));

    // Attempt resolving the model (and obtaining the respective arg values if panic found)
    let satisfiability = solver.check();

    let is_confirmed_safe = satisfiability == SatResult::Unsat;
    let is_confirmed_unsafe = satisfiability == SatResult::Sat;
    println!(
        "\nFunction safety: {}",
        if is_confirmed_safe {
            "safe"
        } else if is_confirmed_unsafe {
            "unsafe"
        } else {
            "unknown"
        }
    );

    if is_confirmed_unsafe {
        // Exhibit a pathological input if the function is unsafe
        // Supports signed int types and booleans
        let model = solver.get_model().unwrap();
        debug!("\n{:?}", model);
        println!("\nUnsafe values:");
        let mut argument_values = Vec::<String>::new();
        for (arg_name, z3_name, var_type) in func_arg_names {
            let arg_name_without_namespace = &arg_name[MAIN_FUNCTION_NAMESPACE.len()..];
            let arg_name_without_namespace_and_percent = arg_name_without_namespace.replace('%', "");
            let value_string;
            if var_type.to_string().eq("\"i1\"") {
                let value = Bool::new_const(solver.get_context(), z3_name.as_str());
                value_string = format!("{:?}", model.eval(&value, true).unwrap());
                let cleaned_value_string = &value_string.replace('(', "").replace(')', "").replace(' ', "");
                println!("\t{:?} = {}", &arg_name_without_namespace_and_percent, cleaned_value_string);
                argument_values.push(cleaned_value_string.to_string());
            } else if var_type.is_int_type() {
                let value = Int::new_const(solver.get_context(), z3_name.as_str());
                value_string = format!("{:?}", model.eval(&value, true).unwrap());
                let cleaned_value_string = &value_string.replace('(', "").replace(')', "").replace(' ', "");
                println!("\t{:?} = {}", &arg_name_without_namespace_and_percent, cleaned_value_string);
                argument_values.push(cleaned_value_string.to_string());
            } else {
                warn!("{} is not a supported parameter type!", var_type);
            }
        }

        let mut source_file_content = fs::read_to_string(file_name).unwrap();
        if !function_name.eq(&String::from("main")) {
            // Inject custom main function as entry point for test program to generate stack trace
            source_file_content = source_file_content.replace("fn main", "fn _main");
            source_file_content = format!("{}\nfn main() {{{}(", source_file_content, function_name);
            for argument_value in &argument_values {
                source_file_content = format!("{}{},", source_file_content, argument_value);
            }
            source_file_content = format!("{});}}", source_file_content);
        }
        debug!("{}", source_file_content);

        let mut temp_file_path_base_end_index = 0;
        if file_name.rfind('/').is_some() {
            temp_file_path_base_end_index = file_name.rfind('/').unwrap() + 1;
        }
        let temp_source_file_name = format!(
            "{}temp_wombat_symx_{}",
            &file_name[0..temp_file_path_base_end_index],
            &file_name[temp_file_path_base_end_index..file_name.len()]
        );
        fs::write(&temp_source_file_name, &source_file_content).expect("Failed to write file!");

        let _temp_source_file_dropper = FileDropper { file_name: &temp_source_file_name };

        let temp_executable_file_name = &temp_source_file_name[0..temp_source_file_name.rfind('.').unwrap()];

        Command::new("rustc")
            .args([&temp_source_file_name, "-o", temp_executable_file_name])
            .status()
            .expect("Failed to generate executable file!");

        let _temp_executable_file_dropper = FileDropper {
            file_name: &String::from(temp_executable_file_name),
        };

        println!("\nError from calling function {} with unsafe arguments:", function_name);
        println!(
            "\t{}",
            std::str::from_utf8(&Command::new(format!("./{}", temp_executable_file_name)).output().ok().unwrap().stderr)
                .unwrap()
                .replace('\n', "\n\t")
        );
    }

    Some(is_confirmed_safe)
}
