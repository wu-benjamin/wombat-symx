use std::collections::{HashMap, HashSet};
use std::path::Path;

use clap::Parser;

use inkwell::basic_block::BasicBlock;
use inkwell::passes::{PassManager, PassManagerBuilder};
use inkwell::values::{FunctionValue, InstructionOpcode, AnyValue};
use rustc_demangle::demangle;
use tracing::{debug};
use tracing_core::Level;
use tracing_subscriber::FmtSubscriber;
use z3::ast::{Ast, Bool, Int};
use z3::{Config, Solver};
use z3::Context as Z3Context;

use inkwell::context::Context as InkwellContext;
use inkwell::module::Module as InkwellModule;
use inkwell::memory_buffer::MemoryBuffer;

mod symbolic_execution;
use symbolic_execution::backward_symbolic_execution;

const COMMON_END_NODE_NAME: &str = "common_end";

#[derive(Debug)]
#[derive(PartialEq)]
enum IsCleanup {
    YES,
    NO,
    UNKNOWN,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enable debug printing
    #[clap(short, long)]
    debug: bool,

    /// Enable printing functions at beginning
    #[clap(short, long)]
    print_functions: bool,

    /// Set file name to perform symbolic execution on
    #[clap()]
    file_name: String,
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
                    for operand in 0..num_operands {
                        if operand % 2 == 1 {
                            let successor_basic_block = terminator.get_operand(operand).unwrap().right().unwrap();
                            let successor_basic_block_name = String::from(successor_basic_block.get_name().to_str().unwrap());
                            node_edges.insert(String::from(successor_basic_block_name));
                        }
                    }
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


fn get_function_argument_names<'a>(function: &'a FunctionValue) -> HashMap<String, String> {
    // TODO: only supports i32s
    let mut arg_names = HashMap::<String, String>::new();
    for param in &function.get_params() {
        let param_int_value = param.into_int_value();
        debug!("Func param instr: {:?}", param_int_value);
        if param_int_value.get_name().to_str().unwrap() == "" {
            // Var name is empty, find in start basic block
            let alias_name = &get_var_name(&param_int_value.as_any_value_enum(), &Solver::new(&Z3Context::new(&Config::new())));
            let start_block = function.get_first_basic_block().unwrap();
            let mut instr = start_block.get_first_instruction();
            while instr.is_some() {
                if instr.unwrap().get_opcode() == InstructionOpcode::Store && alias_name.to_string() == get_var_name(&instr.unwrap().as_any_value_enum(), &Solver::new(&Z3Context::new(&Config::new()))) {
                    let arg_name = get_var_name(&instr.unwrap().get_operand(1).unwrap().left().unwrap().as_any_value_enum(), &Solver::new(&Z3Context::new(&Config::new())));
                    arg_names.insert(arg_name[1..].to_string(), alias_name.to_string());
                }
                instr = instr.unwrap().get_next_instruction();
            }
        } else {
            let arg_name = &param_int_value.get_name().to_str().unwrap().to_string();
            arg_names.insert(arg_name.to_string(), format!("{}{}", "%", arg_name.to_string()));
        }
    }

    debug!("Function arg names: {:?}", arg_names);
    arg_names
}


fn print_file_functions(module: &InkwellModule) -> () {
    //! Iterates through all functions in the file and prints the demangled name
    println!("Functions in {:?}:", module.get_name());
    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        println!("\t{:?} == {:?}", demangle(current_function.get_name().to_str().unwrap()).to_string(), current_function.get_name());
        next_function = current_function.get_next_function();
    }
    println!("");
}


fn pretty_print_function(function: &FunctionValue) -> () {
    debug!("Number of Nodes: {}", function.count_basic_blocks());
    debug!("Arg count: {}", function.count_params());
    // No local decl available to print
    debug!("Basic Blocks:");
    for bb in function.get_basic_blocks() {
        debug!("\tBasic Block: {:?}", bb.get_name().to_str().unwrap());
        debug!("\t\tis_cleanup: {:?}", is_panic_block(&bb));
        let mut next_instruction = bb.get_first_instruction();
        let has_terminator = bb.get_terminator().is_some();

        while let Some(current_instruction) = next_instruction {
            debug!("\t\tStatement: {:?}", current_instruction.to_string());
            next_instruction = current_instruction.get_next_instruction();
        }

        if has_terminator {
            debug!("\t\tLast statement is a terminator")
        } else {
            debug!("\t\tNo terminator")
        }
    }
    debug!("");

    let first_basic_block = function.get_first_basic_block().unwrap();
    debug!("Start node: {:?}", first_basic_block.get_name().to_str().unwrap());
    let forward_edges = get_forward_edges(function);
    let successors = forward_edges.get(first_basic_block.get_name().to_str().unwrap()).unwrap();
    for successor in successors {
        debug!("\tSuccessor to start node: {:?}", successor);
    }
}


fn main() {
    let features = Args::parse();
    
    // Set-up the tracing debug level
    let subscriber = if features.debug {
        FmtSubscriber::builder().with_max_level(Level::DEBUG).finish()
    } else {
        FmtSubscriber::builder().with_max_level(Level::WARN).finish()
    };
    let _guard = tracing::subscriber::set_default(subscriber);
    
    let file_name = String::from(&features.file_name);
    let path = Path::new(&file_name);
    if !path.is_file() {
        // TODO: do we want these error printlns to be tracing::errors?
        println!("{:?} is an invalid file. Please provide a valid file.", file_name);
        return;
    }

    let context = InkwellContext::create();
    let buffer = MemoryBuffer::create_from_file(&path).unwrap();
    let module_result = InkwellModule::parse_bitcode_from_buffer(&buffer, &context);
    
    // Ensure that the module is valid (ie. is a valid bitcode file)
    if module_result.is_err() {
        println!("{:?} is not a valid LLVM bitcode file. Please pass in a valid bc file.", file_name);
        println!("{:?}", module_result);
        return;
    }
    let module = module_result.unwrap();

    
    // Only print file functions if `print_function` flag provided
    if features.print_functions {
        print_file_functions(&module);  
    }

    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        let current_function_name = demangle(&current_function.get_name().to_str().unwrap()).to_string();
        if current_function_name.contains(&file_name[file_name.rfind("/").unwrap()+1..file_name.rfind(".").unwrap()])
                && !current_function_name.contains("::main") {
            // Get function argument names before removing store/alloca instructions
            let func_arg_names = get_function_argument_names(&current_function);
            
            let pass_manager_builder = PassManagerBuilder::create();
            let pass_manager = PassManager::create(&module);
            pass_manager.add_promote_memory_to_register_pass();
            pass_manager_builder.populate_function_pass_manager(&pass_manager);
            pass_manager.run_on(&current_function);

            // Do not process main function for now
            println!("Backward Symbolic Execution in {:?}", demangle(current_function.get_name().to_str().unwrap()));
            pretty_print_function(&current_function);
            let forward_edges = get_forward_edges(&current_function);
            debug!("Forward edges:\t{:?}", forward_edges);
            let backward_edges = get_backward_edges(&current_function);
            debug!("Backward edges:\t{:?}", backward_edges);
            let forward_sorted_nodes = forward_topological_sort(&current_function);
            debug!("Forward sorted nodes:\t{:?}", forward_sorted_nodes);
            let backward_sorted_nodes = backward_topological_sort(&current_function);
            debug!("Backward sorted nodes:\t{:?}", backward_sorted_nodes);
            backward_symbolic_execution(&current_function, &func_arg_names);
            println!("\n************************************\n\n");
        }
        next_function = current_function.get_next_function();
    }
}
