use std::collections::{HashMap, HashSet};

use tracing::warn;

use inkwell::values::{FunctionValue, InstructionOpcode};

use crate::symbolic_execution::COMMON_END_NODE;

pub fn get_forward_edges(
    function: &FunctionValue,
    namespace: &str,
    return_target_node: &str,
) -> HashMap<String, HashSet<String>> {
    let mut all_edges = HashMap::new();
    for bb in function.get_basic_blocks() {
        let mut node_edges = HashSet::new();
        let basic_block_name = format!("{}{}", namespace, bb.get_name().to_str().unwrap());
        if let Some(terminator) = bb.get_terminator() {
            let opcode = terminator.get_opcode();
            let num_operands = terminator.get_num_operands();
            match &opcode {
                InstructionOpcode::Return => {
                    node_edges.insert(String::from(return_target_node));
                }
                InstructionOpcode::Br => {
                    if num_operands == 1 {
                        // Unconditional branch
                        let successor_basic_block =
                            terminator.get_operand(0).unwrap().right().unwrap();
                        let successor_basic_block_name = format!(
                            "{}{}",
                            namespace,
                            successor_basic_block.get_name().to_str().unwrap()
                        );
                        node_edges.insert(successor_basic_block_name);
                    } else if num_operands == 3 {
                        // Conditional branch
                        let successor_basic_block_1 =
                            terminator.get_operand(1).unwrap().right().unwrap();
                        let successor_basic_block_name_1 = format!(
                            "{}{}",
                            namespace,
                            successor_basic_block_1.get_name().to_str().unwrap()
                        );
                        node_edges.insert(successor_basic_block_name_1);
                        let successor_basic_block_2 =
                            terminator.get_operand(2).unwrap().right().unwrap();
                        let successor_basic_block_name_2 = format!(
                            "{}{}",
                            namespace,
                            successor_basic_block_2.get_name().to_str().unwrap()
                        );
                        node_edges.insert(successor_basic_block_name_2);
                    } else {
                        warn!("Incorrect number of operators {:?} for opcode {:?} for edge generations", num_operands, opcode);
                    }
                }
                InstructionOpcode::Switch => {
                    for operand in 0..num_operands {
                        if operand % 2 == 1 {
                            let successor_basic_block =
                                terminator.get_operand(operand).unwrap().right().unwrap();
                            let successor_basic_block_name = format!(
                                "{}{}",
                                namespace,
                                successor_basic_block.get_name().to_str().unwrap()
                            );
                            node_edges.insert(successor_basic_block_name);
                        }
                    }
                }
                InstructionOpcode::IndirectBr => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Invoke => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CallBr => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Resume => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchSwitch => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CatchRet => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::CleanupRet => {
                    warn!("Support for terminator opcode {:?} is not yet implemented for edge generation", opcode);
                }
                InstructionOpcode::Unreachable => {
                    node_edges.insert(String::from(COMMON_END_NODE));
                }
                _ => {
                    warn!(
                        "Opcode {:?} is not supported as a terminator for edge generation",
                        opcode
                    );
                }
            }
        } else {
            warn!("\tNo terminator");
        }
        all_edges.insert(basic_block_name, node_edges);
    }
    all_edges
}

pub fn get_backward_edges(
    function: &FunctionValue,
    namespace: &str,
    return_target_node: &str,
) -> HashMap<String, HashSet<String>> {
    let all_forward_edges = get_forward_edges(function, namespace, return_target_node);
    let mut all_backward_edges = HashMap::new();
    for bb in function.get_basic_blocks() {
        let basic_block_name = format!("{}{}", namespace, bb.get_name().to_str().unwrap());
        all_backward_edges.insert(basic_block_name, HashSet::new());
    }
    for (source, dests) in all_forward_edges {
        for dest in dests {
            if let Some(reverse_dests) = all_backward_edges.get_mut(&dest) {
                reverse_dests.insert(source.clone());
            }
        }
    }
    all_backward_edges
}

pub fn forward_topological_sort(
    function: &FunctionValue,
    namespace: &str,
    return_target_node: &str,
) -> Vec<String> {
    let forward_edges = get_forward_edges(function, namespace, return_target_node);
    let backward_edges = get_backward_edges(function, namespace, return_target_node);
    let mut sorted = Vec::new();
    let mut unsorted = Vec::new();
    for bb in function.get_basic_blocks() {
        let basic_block_name = format!("{}{}", namespace, bb.get_name().to_str().unwrap());
        unsorted.push(basic_block_name);
    }
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
                        for dest in dests.iter() {
                            if let Some(prev_indegree) = indegrees.get_mut(&dest.clone()) {
                                *prev_indegree -= 1;
                            }
                        }
                    }
                }
            }
        }
        match next_node {
            Some(..) => (),
            None => {
                warn!("CFG is cyclic which is not supported");
                break;
            }
        }
    }
    sorted
}

pub fn backward_topological_sort(
    function: &FunctionValue,
    namespace: &str,
    return_target_node: &str,
) -> Vec<String> {
    let mut sorted = forward_topological_sort(function, namespace, return_target_node);
    sorted.reverse();
    sorted
}
