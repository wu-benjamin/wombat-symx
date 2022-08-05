use std::collections::{HashMap, HashSet};
use std::env;

use inkwell::basic_block::BasicBlock;
use inkwell::values::{FunctionValue, InstructionValue, InstructionOpcode, BasicValueEnum};
use rustc_demangle::demangle;
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

#[derive(Debug)]
#[derive(PartialEq)]
enum IsCleanup {
    YES,
    NO,
    UNKNOWN,
}


fn get_basic_block_by_name<'a>(function: &'a FunctionValue, name: &String) -> BasicBlock<'a> {
    let mut matching_bb: Option<BasicBlock> = None;
    let mut matched = false;
    for bb in function.get_basic_blocks() {
        if name.eq(&String::from(bb.get_name().to_str().unwrap())) {
            if matched {
                println!("Multiple basic blocks matched name {:?}", name);
            }
            matching_bb = Some(bb);
            matched = true;
        }
    }
    return matching_bb.unwrap();
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


fn get_var_name(basic_value_enum: &BasicValueEnum) -> String {
    // TODO: Resolve issue with return values from call not having a name
    // TODO: Support other value types
    return String::from(basic_value_enum.into_int_value().get_name().to_str().unwrap());
}


fn get_entry_condition<'a>(
    solver: &'a Solver<'_>,
    function: &'a FunctionValue,
    predecessor: &str,
    node: &str,
) -> Bool<'a> {
    let mut entry_condition = ast::Bool::from_bool(solver.get_context(), true);
    if let Some(terminator) = get_basic_block_by_name(function, &String::from(predecessor)).get_terminator() {
        let opcode = terminator.get_opcode();
        let num_operands = terminator.get_num_operands();
        match &opcode {
            InstructionOpcode::Br => {
                if num_operands == 1 {
                    // Unconditionally go to node
                } else if num_operands == 3 {
                    let mut target_val = false;
                    let discriminant = terminator.get_operand(0).unwrap().left().unwrap();
                    let successor_basic_block_1 = terminator.get_operand(1).unwrap().right().unwrap();
                    let successor_basic_block_name_1 = String::from(successor_basic_block_1.get_name().to_str().unwrap());
                    if successor_basic_block_name_1.eq(&String::from(node)) {
                        target_val = true;
                    }
                    let target_val_var =
                        ast::Bool::from_bool(solver.get_context(), target_val);
                    let switch_var = ast::Bool::new_const(
                        solver.get_context(),
                        get_var_name(&discriminant),
                    );
                    entry_condition = switch_var._eq(&target_val_var);
                } else {
                    println!("Incorrect number of operators {:?} for opcode {:?} for edge generations", num_operands, opcode);
                }
            }
            InstructionOpcode::Return => {
                // Unconditionally go to node
            }
            InstructionOpcode::Unreachable => {
                // Unconditionally go to node
            }
            _ => {
                println!("Opcode {:?} is not supported as a terminator for computing node entry conditions", opcode);
            },
        }
    } else {
        println!("\tNo terminator found when computing node entry conditions");
    }
    return entry_condition;
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

    for node in backward_sorted_nodes {
        let mut successor_conditions = ast::Bool::from_bool(solver.get_context(), true);
        if let Some(successors) = forward_edges.get(&node) {
            for successor in successors {
                let successor_var =
                    ast::Bool::new_const(solver.get_context(), format!("node_{}", successor));
                successor_conditions =
                    ast::Bool::and(solver.get_context(), &[&successor_conditions, &successor_var]);
            }
        }
        let mut node_var = successor_conditions;

        if node == COMMON_END_NODE_NAME.to_string() {
            let panic_var = ast::Bool::new_const(solver.get_context(), PANIC_VAR_NAME);
            node_var = ast::Bool::and(solver.get_context(), &[&panic_var.not(), &node_var]);
        }

        // Parse statements in the basic block

        // handle assign panic
        if let Some(successors) = forward_edges.get(&node) {
            let mut is_predecessor_of_end_node = false;
            for successor in successors {
                if successor == COMMON_END_NODE_NAME {
                    is_predecessor_of_end_node = true;
                    break;
                }
            }
            if is_predecessor_of_end_node {
                let mut is_panic = false;
                if is_panic_block(&get_basic_block_by_name(&function, &node)) == IsCleanup::YES {
                    is_panic = true;
                }
                let panic_var = ast::Bool::new_const(solver.get_context(), PANIC_VAR_NAME);
                let panic_value = ast::Bool::from_bool(solver.get_context(), is_panic);
                let panic_assignment = panic_var._eq(&panic_value);
                node_var = panic_assignment.implies(&node_var);
            }
        }

        let mut entry_conditions_set = false;
        let mut entry_conditions = ast::Bool::from_bool(solver.get_context(), true);
        if let Some(predecessors) = backward_edges.get(&node) {
            if predecessors.len() > 0 {
                for predecessor in predecessors {
                    // get conditions
                    let entry_condition = get_entry_condition(&solver, &function, &predecessor, &node);
                    entry_conditions = ast::Bool::and(solver.get_context(), &[&entry_conditions, &entry_condition]);
                }
                entry_conditions_set = true;
            }
        }  
        if !entry_conditions_set {
            entry_conditions = ast::Bool::from_bool(solver.get_context(), true);
            entry_conditions_set = true;
        }
        node_var = entry_conditions.implies(&node_var);

        let named_node_var = ast::Bool::new_const(solver.get_context(), format!("node_{}", node));
        solver.assert(&named_node_var._eq(&node_var));
    }

    // // constrain int inputs
    for input in function.get_params() {
        // let arg = ast::Int::new_const(&solver.get_context(), format!("_{}", (i + 1).to_string()));
        // TODO: Support other input types
        let arg = ast::Int::new_const(&solver.get_context(), input.into_int_value().get_name().to_str().unwrap());
        let min_int =
            ast::Int::from_bv(&ast::BV::from_i64(solver.get_context(), i32::MIN.into(), 32), true);
        let max_int =
            ast::Int::from_bv(&ast::BV::from_i64(solver.get_context(), i32::MAX.into(), 32), true);
        solver
            .assert(&ast::Bool::and(solver.get_context(), &[&arg.ge(&min_int), &arg.le(&max_int)]));
    }

    let start_node = function.get_first_basic_block().unwrap();
    let start_node_var_name = start_node.get_name().to_str().unwrap();
    let start_node_var = ast::Bool::new_const(solver.get_context(), String::from(start_node_var_name));
    solver.assert(&start_node_var.not());
    println!("{:?}", solver);

    // Attempt resolving the model (and obtaining the respective arg values if panic found)
    println!("Resolved value: {:?}", solver.check());

    if solver.check() == SatResult::Sat {
        // TODO: Identify concrete function params for Sat case
        println!("\n{:?}", solver.get_model().unwrap());
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
    println!("Basic Blocks:");
    for bb in function.get_basic_blocks() {
        println!("\tBasic Block: {:?}", bb.get_name().to_str().unwrap());
        println!("\t\tis_cleanup: {:?}", is_panic_block(&bb));
        let mut next_instruction = bb.get_first_instruction();
        let has_terminator = bb.get_terminator().is_some();

        while let Some(current_instruction) = next_instruction {
            println!("\t\tStatement: {:?}", current_instruction.to_string());
            next_instruction = current_instruction.get_next_instruction();
        }

        if has_terminator {
            println!("\t\tLast statement is a terminator")
        } else {
            println!("\t\tNo terminator")
        }
    }

    let first_basic_block = function.get_first_basic_block().unwrap();
    println!("Start node: {:?}", first_basic_block.get_name().to_str().unwrap());
    let forward_edges = get_forward_edges(function);
    let successors = forward_edges.get(first_basic_block.get_name().to_str().unwrap()).unwrap();
    for successor in successors {
        println!("\tSuccessor to start node: {:?}", successor);
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
