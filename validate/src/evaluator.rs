// src/evaluator.rs

use super::ast::*;
use super::builtins::*;
use super::environment::*;
use super::object::*;
use std::cell::*;
use std::collections::*;
use std::rc::*;

pub const TRUE: Boolean = Boolean { value: true };
pub const FALSE: Boolean = Boolean { value: false };
pub const NULL: Null = Null {};

pub fn native_bool_to_boolean_object(input: bool) -> Boolean {
    if input {
        TRUE
    } else {
        FALSE
    }
}

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
        Node::Program(_) => eval_program(node, Rc::clone(&env)),
        Node::Statement(Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        })) => eval(Node::Expression(expression), Rc::clone(&env)),
        Node::Expression(Expression::IntegerLiteral(IntegerLiteral { token: _, value })) => {
            Some(Object::Integer(Integer { value: value }))
        }
        Node::Expression(Expression::BooleanLiteral(BooleanLiteral { token: _, value })) => {
            Some(Object::Boolean(native_bool_to_boolean_object(value)))
        }
        Node::Expression(Expression::PrefixExpression(PrefixExpression {
            token: _,
            operator,
            right,
        })) => {
            let right_obj = eval(Node::Expression(*right), Rc::clone(&env));
            if is_error(&right_obj) {
                return right_obj;
            }
            eval_prefix_expression(&operator, right_obj)
        }
        Node::Expression(Expression::InfixExpression(InfixExpression {
            token: _,
            left,
            operator,
            right,
        })) => {
            let left_obj = eval(Node::Expression(*left), Rc::clone(&env));
            if is_error(&left_obj) {
                return left_obj;
            }
            let right_obj = eval(Node::Expression(*right), Rc::clone(&env));
            if is_error(&right_obj) {
                return right_obj;
            }
            eval_infix_expression(&operator, left_obj, right_obj)
        }
        Node::Statement(Statement::BlockStatement(block)) => {
            eval_block_statement(block, Rc::clone(&env))
        }
        Node::Expression(Expression::IfExpression(if_expr)) => {
            eval_if_expression(if_expr, Rc::clone(&env))
        }
        Node::Statement(Statement::ReturnStatement(ReturnStatement {
            token: _,
            return_value,
        })) => {
            let val = eval(Node::Expression(return_value), Rc::clone(&env));
            if is_error(&val) {
                return val;
            }
            if val.is_some() {
                Some(Object::ReturnValue(ReturnValue {
                    value: Box::new(val.unwrap()),
                }))
            } else {
                None
            }
        }
        Node::Statement(Statement::LetStatement(LetStatement {
            token: _,
            name,
            value,
        })) => {
            let val = eval(Node::Expression(value), Rc::clone(&env));
            if is_error(&val) {
                return val;
            }

            env.borrow_mut().set(name.value, val)
        }
        Node::Expression(Expression::Identifier(ident)) => eval_identifier(ident, Rc::clone(&env)),
        Node::Expression(Expression::FunctionLiteral(FunctionLiteral {
            token: _,
            parameters,
            body,
        })) => Some(Object::Function(Function {
            parameters: parameters,
            body: body,
            env: Rc::clone(&env),
        })),
        Node::Expression(Expression::CallExpression(CallExpression {
            token: _,
            function,
            arguments,
        })) => {
            let function_obj = eval(Node::Expression(*function), Rc::clone(&env));
            if is_error(&function_obj) {
                return function_obj;
            }
            let args = eval_expressions(arguments, Rc::clone(&env));
            if args.len() == 1 && is_error(&args[0]) {
                return args[0].clone();
            }
            apply_function(function_obj, args)
        }
        Node::Expression(Expression::StringLiteral(StringLiteral { token: _, value })) => {
            Some(Object::StringObj(StringObj { value: value }))
        }
        Node::Expression(Expression::ArrayLiteral(ArrayLiteral { token: _, elements })) => {
            let elements_obj = eval_expressions(elements, Rc::clone(&env));
            if elements_obj.len() == 1 && is_error(&elements_obj[0]) {
                return elements_obj[0].clone();
            }
            Some(Object::Array(Array {
                elements: elements_obj
                    .iter()
                    .filter(|x| x.is_some())
                    .map(|x| x.as_ref().unwrap().clone())
                    .collect(),
            }))
        }
        Node::Expression(Expression::IndexExpression(IndexExpression {
            token: _,
            left,
            index,
        })) => {
            let left_obj = eval(Node::Expression(*left), Rc::clone(&env));
            if is_error(&left_obj) {
                return left_obj;
            }
            if left_obj.is_none() {
                return None;
            }
            let index_obj = eval(Node::Expression(*index), Rc::clone(&env));
            if is_error(&index_obj) {
                return index_obj;
            }
            if index_obj.is_none() {
                return None;
            }
            eval_index_expression(left_obj.unwrap(), index_obj.unwrap())
        }
        Node::Expression(Expression::HashLiteral(hash_literal)) => {
            eval_hash_literal(hash_literal, Rc::clone(&env))
        } // _ => None,
    }
}

// fn eval_statements(stmts: Vec<Statement>, env: Rc<RefCell<Environment>>) -> Option<Object> {
//     let mut result: Option<Object> = None;
//     for statement in stmts.iter() {
//         result = eval(Node::Statement(statement.clone()), Rc::clone(&env));
//         if let Some(Object::ReturnValue(ReturnValue { value })) = result {
//             return Some(*value);
//         }
//     }
//     result
// }

fn eval_prefix_expression(operator: &str, right: Option<Object>) -> Option<Object> {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => new_error(format!(
            "unknown operator: {}{}",
            operator,
            get_type(&right)
        )),
    }
}

fn eval_bang_operator_expression(right: Option<Object>) -> Option<Object> {
    match right {
        Some(Object::Boolean(TRUE)) => Some(Object::Boolean(FALSE)),
        Some(Object::Boolean(FALSE)) => Some(Object::Boolean(TRUE)),
        Some(Object::Null(NULL)) => Some(Object::Boolean(TRUE)),
        _ => Some(Object::Boolean(FALSE)),
    }
}

fn eval_minus_prefix_operator_expression(right: Option<Object>) -> Option<Object> {
    if let Some(Object::Integer(Integer { value })) = right {
        Some(Object::Integer(Integer { value: -value }))
    } else {
        new_error(format!("unknown operator: -{}", get_type(&right)))
    }
}

fn eval_infix_expression(
    operator: &str,
    left: Option<Object>,
    right: Option<Object>,
) -> Option<Object> {
    if get_type(&left) != get_type(&right) {
        return new_error(format!(
            "type mismatch: {} {} {}",
            get_type(&left),
            operator,
            get_type(&right)
        ));
    }
    if let Some(Object::StringObj(_)) = left {
        if let Some(Object::StringObj(_)) = right {
            return eval_string_infix_expression(operator, &left, &right);
        }
    }
    if let Some(Object::Integer(Integer { value })) = left {
        let left_val = value;
        if let Some(Object::Integer(Integer { value })) = right {
            let right_val = value;
            return eval_integer_infix_expression(operator, left_val, right_val);
        }
    }
    if let Some(left_obj) = left {
        if let Some(right_obj) = right {
            return match operator {
                "==" => Some(Object::Boolean(native_bool_to_boolean_object(
                    left_obj == right_obj,
                ))),
                "!=" => Some(Object::Boolean(native_bool_to_boolean_object(
                    left_obj != right_obj,
                ))),
                _ => new_error(format!(
                    "unknown operator: {} {} {}",
                    left_obj.get_type(),
                    operator,
                    right_obj.get_type(),
                )),
            };
        }
    }
    Some(Object::Null(NULL))
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Option<Object> {
    match operator {
        "+" => Some(Object::Integer(Integer {
            value: left + right,
        })),
        "-" => Some(Object::Integer(Integer {
            value: left - right,
        })),
        "*" => Some(Object::Integer(Integer {
            value: left * right,
        })),
        "/" => Some(Object::Integer(Integer {
            value: left / right,
        })),
        "<" => Some(Object::Boolean(native_bool_to_boolean_object(left < right))),
        ">" => Some(Object::Boolean(native_bool_to_boolean_object(left > right))),
        "==" => Some(Object::Boolean(native_bool_to_boolean_object(
            left == right,
        ))),
        "!=" => Some(Object::Boolean(native_bool_to_boolean_object(
            left != right,
        ))),
        _ => new_error(format!("unknown operator: INTEGER {} INTEGER", operator)),
    }
}

fn eval_if_expression(ie: IfExpression, env: Rc<RefCell<Environment>>) -> Option<Object> {
    let condition = eval(Node::Expression(*ie.condition), Rc::clone(&env));
    if is_error(&condition) {
        return condition;
    }
    if is_truthy(&condition) {
        return eval(
            Node::Statement(Statement::BlockStatement(ie.consequence)),
            Rc::clone(&env),
        );
    } else if ie.alternative.is_some() {
        return eval(
            Node::Statement(Statement::BlockStatement(ie.alternative.unwrap())),
            Rc::clone(&env),
        );
    } else {
        Some(Object::Null(NULL))
    }
}

fn is_truthy(obj: &Option<Object>) -> bool {
    match obj {
        Some(Object::Null(NULL)) => false,
        Some(Object::Boolean(TRUE)) => true,
        Some(Object::Boolean(FALSE)) => false,
        _ => true,
    }
}

fn eval_program(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    let mut result: Option<Object> = None;
    if let Node::Program(Program { statements }) = node {
        for statement in statements.iter() {
            result = eval(Node::Statement(statement.clone()), Rc::clone(&env));
            if let Some(Object::ReturnValue(ReturnValue { value })) = result {
                return Some(*value);
            } else if let Some(Object::ErrorObj(_)) = result {
                return result;
            }
        }
    }
    result
}

fn eval_block_statement(block: BlockStatement, env: Rc<RefCell<Environment>>) -> Option<Object> {
    let mut result: Option<Object> = None;
    for statement in block.statements.iter() {
        result = eval(Node::Statement(statement.clone()), Rc::clone(&env));

        if let Some(Object::ReturnValue(_)) = result {
            return result;
        } else if let Some(Object::ErrorObj(_)) = result {
            return result;
        }
    }
    result
}

pub fn new_error(msg: String) -> Option<Object> {
    Some(Object::ErrorObj(ErrorObj { message: msg }))
}

pub fn get_type(obj: &Option<Object>) -> String {
    if obj.is_some() {
        obj.as_ref().unwrap().get_type()
    } else {
        String::from("None")
    }
}

fn is_error(obj: &Option<Object>) -> bool {
    if let Some(Object::ErrorObj(_)) = obj {
        true
    } else {
        false
    }
}

fn eval_identifier(node: Identifier, env: Rc<RefCell<Environment>>) -> Option<Object> {
    if let Some(val) = env.borrow().get(node.value.clone()) {
        val
    } else if let Some(builtin) = get_builtin(&node.value) {
        Some(builtin)
    } else {
        new_error(format!("identifier not found: {}", node.value))
    }
}

fn eval_expressions(exps: Vec<Expression>, env: Rc<RefCell<Environment>>) -> Vec<Option<Object>> {
    let mut result: Vec<Option<Object>> = Vec::new();
    for e in exps.iter() {
        let evaluated = eval(Node::Expression(e.clone()), Rc::clone(&env));
        if is_error(&evaluated) {
            return vec![evaluated];
        }
        result.push(evaluated);
    }
    result
}

fn apply_function(func: Option<Object>, args: Vec<Option<Object>>) -> Option<Object> {
    if let Some(Object::Function(function)) = func {
        let extended_env = Rc::new(RefCell::new(extend_function_env(function.clone(), args)));
        let evaluated = eval(
            Node::Statement(Statement::BlockStatement(function.body)),
            Rc::clone(&extended_env),
        );
        unwrap_return_value(evaluated)
    } else if let Some(Object::Builtin(Builtin { func })) = func {
        func(&args)
    } else {
        new_error(format!("not a function: {:?}", get_type(&func)))
    }
}

fn extend_function_env(func: Function, args: Vec<Option<Object>>) -> Environment {
    let mut env = new_enclosed_environment(Some(func.env));
    for (param_idx, param) in func.parameters.iter().enumerate() {
        env.set(param.value.clone(), args[param_idx].clone());
    }
    env
}

fn unwrap_return_value(obj: Option<Object>) -> Option<Object> {
    if let Some(Object::ReturnValue(ReturnValue { value })) = obj {
        return Some(*value);
    }
    obj
}

fn eval_string_infix_expression(
    operator: &str,
    left: &Option<Object>,
    right: &Option<Object>,
) -> Option<Object> {
    if operator != "+" {
        return new_error(format!(
            "unknown operator: {} {} {}",
            get_type(&left),
            operator,
            get_type(&right)
        ));
    }
    if let Some(Object::StringObj(StringObj { value })) = left {
        let left_val = value;
        if let Some(Object::StringObj(StringObj { value })) = right {
            let right_val = value;
            return Some(Object::StringObj(StringObj {
                value: format!("{}{}", left_val, right_val),
            }));
        }
    }
    None
}

fn eval_index_expression(left: Object, index: Object) -> Option<Object> {
    if let Object::Array(_) = left {
        if let Object::Integer(_) = index {
            return eval_array_index_expression(left, index);
        }
    } else if let Object::Hash(hash_obj) = left {
        return eval_hash_index_expression(hash_obj, index);
    }
    new_error(format!("index operator not supported: {}", left.get_type()))
}

fn eval_array_index_expression(array: Object, index: Object) -> Option<Object> {
    if let Object::Array(Array { elements }) = array {
        if let Object::Integer(Integer { value }) = index {
            let idx = value;
            let max = elements.len() as i64 - 1;
            if idx < 0 || idx > max {
                return Some(Object::Null(NULL));
            }
            return Some(elements[idx as usize].clone());
        }
    }
    None
}

fn eval_hash_literal(node: HashLiteral, env: Rc<RefCell<Environment>>) -> Option<Object> {
    let mut pairs: HashMap<HashKey, Object> = HashMap::new();

    for (key_node, value_node) in node.pairs.iter() {
        let key = eval(Node::Expression(key_node.clone()), Rc::clone(&env));
        if is_error(&key) {
            return key;
        }
        if key.is_none() {
            return None;
        }

        if let Some(hash_key) = key.as_ref().unwrap().as_hashable() {
            let value = eval(Node::Expression(value_node.clone()), Rc::clone(&env));
            if is_error(&value) {
                return value;
            }
            if value.is_none() {
                return None;
            }
            let hashed = hash_key.hash_key();
            pairs.insert(hashed, value.unwrap());
        } else {
            assert!(false, "unusable as hash key: {}", get_type(&key));
        }
    }
    Some(Object::Hash(Hash { pairs: pairs }))
}

fn eval_hash_index_expression(hash: Hash, index: Object) -> Option<Object> {
    if let Some(key) = index.as_hashable() {
        if let Some(pair) = hash.pairs.get(&key.hash_key()) {
            return Some(pair.clone());
        }
        return Some(Object::Null(NULL));
    } else {
        return new_error(format!("unusable as hash key: {}", index.get_type()));
    }
}
