use std::{collections::VecDeque};
use std::env;

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


// fn parse_control_flow_graph(func_analysis: &FunctionAnalysis) -> () {
//     //! Parses the llvm-ir-analysis function_analysis object for each basic block
//     let cfg = func_analysis.control_flow_graph();

//     let mut blocks = Vec::<CFGNode>::new();
    
//     let mut stack = VecDeque::<CFGNode>::new();
//     stack.extend(cfg.succs(cfg.entry()));
//     match stack[0] {
//         CFGNode::Block(name) => {
//             let preds = cfg.preds(name);
//             debug!("Start preds:");
//             for pred in preds {
//                 debug!("{:?}", pred);
//             }
//         },
//         _ => debug!("\tReturn block"),
//     }

//     while stack.len() > 0 {
//         let current_block = stack.pop_front().unwrap();
//         if !blocks.contains(&current_block) {
//             blocks.push(current_block);
//             match current_block {
//                 CFGNode::Block(name) => {
//                     debug!("\tvalue: {}", name);
//                     stack.extend(cfg.succs(name));
//                 },
//                 _ => debug!("\tReturn block"),
//             }
//         }
//     }
// }

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


fn backward_symbolic_execution(function: &FunctionValue) -> () {
    //! Perform backward symbolic execution on a function given the llvm-ir function object
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
        if demangle(&current_function.get_name().to_str().unwrap()).to_string().contains(&file_name[file_name.find("/").unwrap()+1..file_name.find(".").unwrap()]) {
            println!("Backward Symbolic Execution in {:?}", demangle(current_function.get_name().to_str().unwrap()));
            backward_symbolic_execution(&current_function);
        }
        next_function = current_function.get_next_function();
    }

    // let ma = ModuleAnalysis::new(&module);
    // for func in &module.functions {
    //     if demangle(&func.name).to_string().contains(&file_name[file_name.find("/").unwrap()+1..file_name.find(".").unwrap()]) {
    //         let func = module.get_func_by_name(&func.name).unwrap();
            
    //         println!("Backward Symbolic Execution in {:?}", demangle(&func.name));
    //         backward_symbolic_execution(func);
    //     }
    // }
    // // let func = module.get_func_by_name("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E").unwrap();
    // // let fa = ma.fn_analysis("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E");
    
    // // debug!("Backward Symbolic Execution in {:?}", demangle("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E"));
    // // backward_symbolic_execution(func);
    // // parse_control_flow_graph(fa);
}
