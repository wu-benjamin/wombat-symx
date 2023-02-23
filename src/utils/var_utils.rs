use inkwell::values::{AnyValue};

use z3::ast::{Ast, Bool, Int};
use z3::Solver;


pub const CONST_NAMESPACE: &str = "const_";


pub fn get_var_name<'a>(value: &dyn AnyValue, solver: &'a Solver<'_>, namespace: &str) -> String {
    let value_llvm_str = value.print_to_string();
    let value_str = value_llvm_str.to_str().unwrap();
    let name = if !value_str.contains("%") {
        // handle const literals
        let const_value_str = value_str.split_whitespace().nth(1).unwrap();
        let var_name_string = format!("{}{}", CONST_NAMESPACE, const_value_str);
        let var_name = var_name_string.as_str();
        if const_value_str.eq("true") {
            let true_const = Bool::new_const(solver.get_context(), var_name);
            solver.assert(&true_const._eq(&Bool::from_bool(solver.get_context(), true)));
        } else if const_value_str.eq("false") {
            let false_const = Bool::new_const(solver.get_context(), var_name);
            solver.assert(&false_const._eq(&Bool::from_bool(solver.get_context(), false)));
        } else {
            let parsed_num = const_value_str.parse::<i64>().unwrap();
            let num_const = Int::new_const(solver.get_context(), var_name);
            solver.assert(&num_const._eq(&Int::from_i64(solver.get_context(), parsed_num.into())));
        }
        String::from(var_name)
    } else {
        let start_index = value_str.find("%").unwrap();
        let end_index = value_str[start_index..].find(|c: char| c == '"' || c == ' ' || c == ',').unwrap_or(value_str[start_index..].len()) + start_index;
        let var_name = String::from(&value_str[start_index..end_index]);
        String::from(format!("{}{}", namespace, var_name))
    };
    return name;
}

pub fn get_min_max_signed_int(size: &str) -> (i64, i64) {
    match size {
        "8" => (i8::MIN as i64, i8::MAX as i64),
        "16" => (i8::MIN as i64, i8::MAX as i64),
        "32" => (i32::MIN as i64, i32::MAX as i64),
        "64" => (i64::MIN, i64::MAX),
        "size" => (isize::MIN as i64, isize::MAX as i64), // isize depends on devices architecture (32 bits or 64 bits)
        "128" => panic!("i128 are unsupported due to Z3 allowing up to 64 bit integers"),
        _ => panic!("Unsupported signed integer type")
    }
}
