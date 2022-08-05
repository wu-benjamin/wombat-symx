use std::collections::{VecDeque, HashMap, HashSet};
use std::env;
use either;

use inkwell::values::{FunctionValue, InstructionValue, InstructionOpcode};
use rustc_demangle::demangle;
use tracing::debug;
use z3::{
    ast::{self, Ast, Bool, Int},
    SatResult,
};
use z3::{Config, Solver};
use z3::Context as Z3Context;

use inkwell::context::Context as InkwellContext;
use inkwell::module::Module as InkwellModule;
use inkwell::memory_buffer::MemoryBuffer;
use std::path::Path;

const COMMON_END_NODE_NAME: &str = "common_end";
const PANIC_VAR_NAME: &str = "panic_var";

fn parse_instruction(instruction: &InstructionValue) -> () {
    match instruction.get_opcode() {
        InstructionOpcode::Add => {
            println!("\t\t\tAdd operation: {:?}", "add")
        },
        InstructionOpcode::Mul => {
            println!("\t\t\tMul operation: {:?}", "mul")
        }
        _ => {
            println!("\t\t\tUnknown operation: {:?}", instruction.get_opcode());
        }
    }
    for operand_index in 0..instruction.get_num_operands() {
        let operand = instruction.get_operand(operand_index).unwrap();
        println!("\t\t\t\tOperand {}: {:?}", operand_index, operand);
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
                    println!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
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


fn backward_symbolic_execution(function: &FunctionValue) -> () {
    // Perform backward symbolic execution on a function given the llvm-ir function object
    let forward_edges = get_forward_edges(&function);
    let backward_edges = get_backward_edges(&function);
    let backward_sorted_nodes = backward_topological_sort(&function);

    // Initialize the Z3 and Builder objects
    let cfg = Config::new();
    let ctx = Z3Context::new(&cfg);
    let solver = Solver::new(&ctx);

    println!("\tBasic Blocks:");
    for bb in function.get_basic_blocks() {
        println!("\t\t{:?}", bb.get_name().to_str().unwrap());
        let mut next_instruction = bb.get_first_instruction();
        while let Some(current_instruction) = next_instruction {
            println!("\t\t\t{:?}", current_instruction.to_string());
            // parse_instruction(&current_instruction);
            next_instruction = current_instruction.get_next_instruction();
        }
        // Terminator is already printed as a regular instruction
        // println!("\t\t\t{:?}", bb.get_terminator());
    }
}


fn print_file_functions(module: &InkwellModule) -> () {
    //! Iterates through all functions in the file and prints the demangled name
    println!("Functions in {:?}:", module.get_name());
    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        println!("\t{:?} == {:?}", demangle(current_function.get_name().to_str().unwrap()).to_string(), current_function.get_name());
        next_function = current_function.get_next_function();
    }
}


fn pretty_print_function(function: &FunctionValue) -> () {
    println!("Number of Nodes: {}", function.count_basic_blocks());
    println!("Arg count: {}", function.count_params());
    // No local decl available to print
    // for i in 0..body.num_nodes() {
    //     // debug!("Node: {:?}", body.basic_blocks()[BasicBlock::from_usize(i)]);
    //     debug!("bb{}", i);
    //     debug!("\tis_cleanup: {}", body.basic_blocks()[BasicBlock::from_usize(i)].is_cleanup);
    //     for j in 0..body.basic_blocks()[BasicBlock::from_usize(i)].statements.len() {
    //         let statement = &body.basic_blocks()[BasicBlock::from_usize(i)].statements[j];
    //         if matches!(statement.kind, StatementKind::Assign(..)) {
    //             debug!("\tStatement: {:?}", statement);
    //         }
    //     }
    //     if let Some(terminator) = &body.basic_blocks()[BasicBlock::from_usize(i)].terminator {
    //         debug!("\tTerminator: {:?}", terminator.kind);
    //     // match &terminator.kind {
    //     //     TerminatorKind::Call{..} => {
    //     //         debug!("is call!");
    //     //     },
    //     //     _ => (),
    //     // }
    //     } else {
    //         debug!("\tNo terminator");
    //     }
    // }
    // debug!("Start Node: {:?}", body.start_node());
    // body.successors(body.start_node()).for_each(|bb| {
    //     debug!("Successor to Start: {:?}", bb);
    // });
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut file_name = String::from("tests/hello_world.bc");
    if args.len() > 1 {
        // Use custom user file
        file_name = args[1].to_string();
    }

    let path = Path::new(&file_name);
    let context = InkwellContext::create();
    let buffer = MemoryBuffer::create_from_file(&path).unwrap();
    let module = InkwellModule::parse_bitcode_from_buffer(&buffer, &context).unwrap();
    print_file_functions(&module);

    let mut next_function = module.get_first_function();
    while let Some(current_function) = next_function {
        let current_function_name = demangle(&current_function.get_name().to_str().unwrap()).to_string();
        if current_function_name.contains(&file_name[file_name.find("/").unwrap()+1..file_name.find(".").unwrap()])
                && !current_function_name.contains("::main") {
            // Do not process main function for now
            println!("Backward Symbolic Execution in {:?}", demangle(current_function.get_name().to_str().unwrap()));
            pretty_print_function(&current_function);
            let forward_edges = get_forward_edges(&current_function);
            println!("Forward edges:\n\t{:?}", forward_edges);
            let backward_edges = get_backward_edges(&current_function);
            println!("Backward edges:\n\t{:?}", backward_edges);
            let forward_sorted_nodes = forward_topological_sort(&current_function);
            println!("Forward sorted nodes:\n\t{:?}", forward_sorted_nodes);
            let backward_sorted_nodes = backward_topological_sort(&current_function);
            println!("Backward sorted nodes:\n\t{:?}", backward_sorted_nodes);
            backward_symbolic_execution(&current_function);
        }
        next_function = current_function.get_next_function();
    }
}
