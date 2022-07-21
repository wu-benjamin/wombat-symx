use std::collections::VecDeque;

use llvm_ir::Module;
use llvm_ir_analysis::{ModuleAnalysis, FunctionAnalysis, CFGNode};
use rustc_demangle::demangle;


fn backward_symbolic_execution(function: &FunctionAnalysis) -> () {
    let cfg = function.control_flow_graph();

    let mut blocks = Vec::<CFGNode>::new();
    
    let mut stack = VecDeque::<CFGNode>::new();
    stack.extend(cfg.succs(cfg.entry()));

    while stack.len() > 0 {
        let current_block = stack.pop_front().unwrap();
        if !blocks.contains(&current_block) {
            blocks.push(current_block);
            match current_block {
                CFGNode::Block(name) => {
                    println!("\tvalue: {}", name);
                    stack.extend(cfg.succs(name));
                },
                _ => println!("\tReturn block"),
            }
        }
    }


    println!("\tNumber of blocks: {:?}", blocks.len());
    for block in blocks {
        println!("\t\t{:?}", block.to_string());
    }
}


fn print_file_functions(module: &Module) -> () {
    println!("Functions in {:?}:", module.name);
    for func in &module.functions {
        println!("\t{:?}", demangle(&func.name.as_str()).to_string());
    }
}


fn main() {
    println!("Hello, world!");
    let module = Module::from_bc_path("./tests/hello_world.bc").unwrap();
    
    print_file_functions(&module);

    let ma = ModuleAnalysis::new(&module);
    let fa = ma.fn_analysis("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E");
    println!("Backward Symbolic Execution in {:?}", demangle("_ZN11hello_world7neg_abs17h8bd18ec7b7f1f032E"));
    backward_symbolic_execution(fa);

}
