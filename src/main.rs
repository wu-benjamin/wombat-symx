use std::{collections::VecDeque};
use std::env;

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

// fn parse_instruction(instr: &Instruction) -> () {

//     match instr {
//         Instruction::Add(add) => {
//             println!("\tAdd operation: {:?}", add)
//         },
//         Instruction::Mul(mul) => {
//             println!("\tMul operation: {:?}", mul)
//         }
//         unknown_opp => {
//             debug!("\tUnknown operation: {:?}", unknown_opp);
//         }
//     }
// }


// fn backward_symbolic_execution(func: &Function) -> () {
//     //! Perform backward symbolic execution on a function given the llvm-ir function object
//     println!("\tBasic Blocks:");
//     for bb in &func.basic_blocks {
//         println!("\t\t{:?}", bb.name);
//         for instr in &bb.instrs {
//             println!("\t\t\t{:?}", instr.to_string());
//             parse_instruction(instr);
//         }
//         println!("\t\t\t{:?}", bb.term);
//     }
// }


fn print_file_functions(module: InkwellModule) -> () {
    //! Iterates through all functions in the file and prints the demangled name
    println!("Functions in {:?}:", module.get_name());
    let mut current_function = module.get_first_function();
    while let Some(function) = current_function {
        println!("\t{:?} == {:?}", demangle(function.get_name().to_str().unwrap()).to_string(), function.get_name());
        current_function = function.get_next_function();
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
    print_file_functions(module);

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
