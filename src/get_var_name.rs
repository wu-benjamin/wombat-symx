use inkwell::values::{AnyValue};
use z3::ast::{Ast, Bool, Int};
use z3::Solver;

pub fn get_var_name<'a>(value: &dyn AnyValue, solver: &'a Solver<'_>) -> String {
    // handle const literal
    let llvm_str = value.print_to_string();
    let str = llvm_str.to_str().unwrap();
    if !str.contains("%") {
        let var_name = str.split_whitespace().nth(1).unwrap();
        if var_name.eq("true") {
            let true_const = Bool::new_const(solver.get_context(), format!("const_{}", var_name));
            solver.assert(&true_const._eq(&Bool::from_bool(solver.get_context(), true)));
        } else if var_name.eq("false") {
            let false_const = Bool::new_const(solver.get_context(), format!("const_{}", var_name));
            solver.assert(&false_const._eq(&Bool::from_bool(solver.get_context(), false)));
        } else {
            let parsed_num = var_name.parse::<i64>().unwrap();
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