// src/evaluator.rs

use super::builtins::*;
use crate::ast::*;
use crate::object::*;
use std::cell::*;
use std::collections::*;
use std::rc::*;

pub const TRUE: Boolean = Boolean { value: true };
pub const FALSE: Boolean = Boolean { value: false };
pub const NULL: Null = Null {};

pub fn evaluate(node: Node, env: Rc<RefCell<Environment>>) -> Object {
    match eval(node, env) {
        Ok(v) => v,
        Err(err) => Object::ErrorObj(ErrorObj { message: err }),
    }
}
fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match node {
        Node::Program(program) => eval_program(program, Rc::clone(&env)),

        Node::Statement(Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        })) => eval(Node::Expression(expression), Rc::clone(&env)),

        Node::Expression(Expression::IntegerLiteral(IntegerLiteral { token: _, value })) => {
            Ok(Object::Integer(Integer { value: value }))
        }
        Node::Expression(Expression::BooleanLiteral(BooleanLiteral { token: _, value })) => {
            Ok(Object::Boolean(native_bool_to_boolean_object(value)))
        }
        Node::Expression(Expression::PrefixExpression(PrefixExpression {
            token: _,
            operator,
            right,
        })) => {
            let right_obj = eval(Node::Expression(*right), Rc::clone(&env))?;
            eval_prefix_expression(&operator, right_obj)
        }
        Node::Expression(Expression::InfixExpression(InfixExpression {
            token: _,
            left,
            operator,
            right,
        })) => {
            let left_obj = eval(Node::Expression(*left), Rc::clone(&env))?;
            let right_obj = eval(Node::Expression(*right), Rc::clone(&env))?;
            eval_infix_expression(&operator, &left_obj, &right_obj)
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
            let val = eval(Node::Expression(return_value), Rc::clone(&env))?;
            Ok(Object::ReturnValue(ReturnValue {
                value: Box::new(val),
            }))
        }
        Node::Statement(Statement::LetStatement(LetStatement {
            token: _,
            name,
            value,
        })) => {
            let val = eval(Node::Expression(value), Rc::clone(&env))?;
            Ok(env.borrow_mut().set(name.value, val).clone())
        }
        Node::Expression(Expression::Identifier(ident)) => eval_identifier(ident, Rc::clone(&env)),
        Node::Expression(Expression::FunctionLiteral(FunctionLiteral {
            token: _,
            parameters,
            body,
        })) => Ok(Object::Function(Function {
            parameters: parameters,
            body: body,
            env: Rc::clone(&env),
        })),
        Node::Expression(Expression::CallExpression(CallExpression {
            token: _,
            function,
            arguments,
        })) => {
            let function_obj = eval(Node::Expression(*function), Rc::clone(&env))?;
            let mut args = eval_expressions(arguments, Rc::clone(&env))?;
            apply_function(function_obj, &mut args)
        }
        Node::Expression(Expression::StringLiteral(StringLiteral { token: _, value })) => {
            Ok(Object::StringObj(StringObj { value: value }))
        }
        Node::Expression(Expression::ArrayLiteral(ArrayLiteral { token: _, elements })) => {
            let elements_obj = eval_expressions(elements, Rc::clone(&env))?;
            Ok(Object::Array(Array {
                elements: elements_obj,
            }))
        }
        Node::Expression(Expression::IndexExpression(IndexExpression {
            token: _,
            left,
            index,
        })) => {
            let left_obj = eval(Node::Expression(*left), Rc::clone(&env))?;
            let index_obj = eval(Node::Expression(*index), Rc::clone(&env))?;
            eval_index_expression(&left_obj, &index_obj)
        }
        Node::Expression(Expression::HashLiteral(hash_literal)) => {
            eval_hash_literal(hash_literal, Rc::clone(&env))
        }
    }
}

fn eval_program(program: Program, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    let mut result: Object = Object::Null(NULL);
    for statement in program.statements.into_iter() {
        result = eval(Node::Statement(statement), Rc::clone(&env))?;
        if let Object::ReturnValue(ReturnValue { value }) = result {
            return Ok(*value);
        }
    }
    Ok(result)
}

pub fn native_bool_to_boolean_object(input: bool) -> Boolean {
    if input {
        TRUE
    } else {
        FALSE
    }
}

fn eval_prefix_expression(operator: &str, right: Object) -> Result<Object, String> {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => Err(format!(
            "unknown operator: {}{}",
            operator,
            right.get_type()
        )),
    }
}

fn eval_bang_operator_expression(right: Object) -> Result<Object, String> {
    match right {
        Object::Boolean(TRUE) => Ok(Object::Boolean(FALSE)),
        Object::Boolean(FALSE) => Ok(Object::Boolean(TRUE)),
        Object::Null(NULL) => Ok(Object::Boolean(TRUE)),
        _ => Ok(Object::Boolean(FALSE)),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Result<Object, String> {
    if let Object::Integer(Integer { value }) = right {
        Ok(Object::Integer(Integer { value: -value }))
    } else {
        Err(format!("unknown operator: -{}", right.get_type()))
    }
}

fn eval_infix_expression(operator: &str, left: &Object, right: &Object) -> Result<Object, String> {
    if left.get_type() != right.get_type() {
        return Err(format!(
            "type mismatch: {} {} {}",
            left.get_type(),
            operator,
            right.get_type()
        ));
    }
    if let Object::StringObj(value) = left {
        let left_val = value;
        if let Object::StringObj(value) = right {
            let right_val = value;
            return eval_string_infix_expression(operator, left_val, right_val);
        }
    } else if let Object::Integer(Integer { value }) = left {
        let left_val = value;
        if let Object::Integer(Integer { value }) = right {
            let right_val = value;
            return eval_integer_infix_expression(operator, *left_val, *right_val);
        }
    }
    return match operator {
        "==" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left == right,
        ))),
        "!=" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left != right,
        ))),
        _ => Err(format!(
            "unknown operator: {} {} {}",
            left.get_type(),
            operator,
            right.get_type(),
        )),
    };
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Result<Object, String> {
    match operator {
        "+" => Ok(Object::Integer(Integer {
            value: left + right,
        })),
        "-" => Ok(Object::Integer(Integer {
            value: left - right,
        })),
        "*" => Ok(Object::Integer(Integer {
            value: left * right,
        })),
        "/" => Ok(Object::Integer(Integer {
            value: left / right,
        })),
        "<" => Ok(Object::Boolean(native_bool_to_boolean_object(left < right))),
        ">" => Ok(Object::Boolean(native_bool_to_boolean_object(left > right))),
        "==" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left == right,
        ))),
        "!=" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left != right,
        ))),
        _ => Err(format!("unknown operator: INTEGER {} INTEGER", operator)),
    }
}

fn eval_if_expression(ie: IfExpression, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    let condition = eval(Node::Expression(*ie.condition), Rc::clone(&env))?;
    if is_truthy(condition) {
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
        Ok(Object::Null(NULL))
    }
}

fn is_truthy(obj: Object) -> bool {
    match obj {
        Object::Null(NULL) => false,
        Object::Boolean(TRUE) => true,
        Object::Boolean(FALSE) => false,
        _ => true,
    }
}

fn eval_block_statement(
    block: BlockStatement,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let mut result: Object = Object::Null(NULL);
    for statement in block.statements.into_iter() {
        result = eval(Node::Statement(statement), Rc::clone(&env))?;

        if let Object::ReturnValue(_) = result {
            return Ok(result);
        }
    }
    Ok(result)
}

fn eval_identifier(node: Identifier, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    if let Some(val) = env.borrow().get(&node.value) {
        Ok(val.clone())
    } else if let Some(builtin) = get_builtin(&node.value) {
        Ok(builtin)
    } else {
        Err(format!("identifier not found: {}", node.value))
    }
}

fn eval_expressions(
    exps: Vec<Expression>,
    env: Rc<RefCell<Environment>>,
) -> Result<Vec<Object>, String> {
    let mut result: Vec<Object> = Vec::new();
    for e in exps.into_iter() {
        let evaluated = eval(Node::Expression(e), Rc::clone(&env))?;
        result.push(evaluated);
    }
    Ok(result)
}

fn apply_function(func: Object, args: &mut Vec<Object>) -> Result<Object, String> {
    if let Object::Function(function) = func {
        let extended_env = Rc::new(RefCell::new(extend_function_env(&function, args)));
        let evaluated = eval(
            Node::Statement(Statement::BlockStatement(function.body)),
            Rc::clone(&extended_env),
        )?;
        unwrap_return_value(evaluated)
    } else if let Object::Builtin(Builtin { func }) = func {
        func(&args)
    } else {
        Err(format!("not a function: {:?}", func.get_type()))
    }
}

fn extend_function_env(func: &Function, args: &mut Vec<Object>) -> Environment {
    let mut env = new_enclosed_environment(Some(Rc::clone(&func.env)));
    for (param_idx, param) in func.parameters.iter().enumerate() {
        env.set(
            param.value.clone(),
            std::mem::replace(&mut args[param_idx], Object::Null(NULL)),
        );
    }
    env
}

fn unwrap_return_value(obj: Object) -> Result<Object, String> {
    if let Object::ReturnValue(ReturnValue { value }) = obj {
        return Ok(*value);
    }
    Ok(obj)
}

fn eval_string_infix_expression(
    operator: &str,
    left: &StringObj,
    right: &StringObj,
) -> Result<Object, String> {
    if operator != "+" {
        return Err(format!(
            "unknown operator: {} {} {}",
            left.get_type(),
            operator,
            right.get_type()
        ));
    }
    return Ok(Object::StringObj(StringObj {
        value: format!("{}{}", left.value, right.value),
    }));
}

fn eval_index_expression(left: &Object, index: &Object) -> Result<Object, String> {
    if let Object::Array(Array { elements }) = left {
        if let Object::Integer(Integer { value }) = index {
            return eval_array_index_expression(&elements, *value);
        }
    } else if let Object::Hash(hash_obj) = left {
        return eval_hash_index_expression(hash_obj, index);
    }
    Err(format!("index operator not supported: {}", left.get_type()))
}

fn eval_array_index_expression(elements: &Vec<Object>, idx: i64) -> Result<Object, String> {
    let max = elements.len() as i64 - 1;
    if idx < 0 || idx > max {
        return Ok(Object::Null(NULL));
    }
    return Ok(elements[idx as usize].clone());
}

fn eval_hash_literal(node: HashLiteral, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    let mut pairs: HashMap<HashKey, Object> = HashMap::new();

    for (key_node, value_node) in node.pairs.into_iter() {
        let key = eval(Node::Expression(key_node), Rc::clone(&env))?;
        if let Some(hash_key) = key.as_hashable() {
            let value = eval(Node::Expression(value_node), Rc::clone(&env))?;
            let hashed = hash_key.hash_key();
            pairs.insert(hashed, value);
        } else {
            panic!("unusable as hash key: {}", key.get_type());
        }
    }
    Ok(Object::Hash(Hash { pairs: pairs }))
}

fn eval_hash_index_expression(hash: &Hash, index: &Object) -> Result<Object, String> {
    if let Some(key) = index.as_hashable() {
        if let Some(pair) = hash.pairs.get(&key.hash_key()) {
            return Ok(pair.clone());
        }
        Ok(Object::Null(NULL))
    } else {
        Err(format!("unusable as hash key: {}", index.get_type()))
    }
}
