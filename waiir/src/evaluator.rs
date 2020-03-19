use super::ast::*;
use super::env::*;
use super::object::*;
use std::cell::*;
use std::collections::*;
use std::rc::*;

pub const TRUE: Object = Object::Bool(Bool { value: true });
pub const FALSE: Object = Object::Bool(Bool { value: false });
pub const NULL: Object = Object::Null {};

pub fn eval(node: Node, env: Rc<RefCell<Env>>) -> Option<Object> {
    println!("eval: {:?}", node);
    match node {
        Node::Program(program) => eval_program(program, env),
        Node::Stmt(stmt) => match stmt {
            Stmt::ExprStmt { token: _, expr } => eval(Node::Expr(expr), env),
            Stmt::BlockStmt(block_stmt) => eval_block_stmt(block_stmt, env),
            Stmt::ReturnStmt { token: _, value } => {
                let val = eval(Node::Expr(value), env);
                if is_error(&val) {
                    return val;
                }
                Some(Object::ReturnValue {
                    value: Box::new(val), //TODO ?
                })
            }
            Stmt::LetStmt {
                token: _,
                name,
                value,
            } => {
                let val = eval(Node::Expr(value), Rc::clone(&env));
                if is_error(&val) {
                    return val;
                }
                env.borrow_mut().set(name.value, val) //TODO ?
            }
        },
        Node::Expr(expr) => match expr {
            Expr::IntLiteral(int_lite) => Some(Object::Int(Int {
                value: int_lite.value,
            })),
            Expr::Bool { token: _, value } => Some(native_bool_to_boolean_obj(value)),
            Expr::PrefixExpr(PrefixExpr {
                token: _,
                operator,
                right,
            }) => {
                let right_obj = eval(Node::Expr(*right), env);
                if is_error(&right_obj) {
                    return right_obj;
                }
                eval_prefix_expr(&operator, right_obj)
            }
            Expr::InfixExpr(InfixExpr {
                token: _,
                left,
                operator,
                right,
            }) => {
                let left_obj = eval(Node::Expr(*left), Rc::clone(&env));
                if is_error(&left_obj) {
                    return left_obj;
                }
                let right_obj = eval(Node::Expr(*right), env);
                if is_error(&right_obj) {
                    return right_obj;
                }
                eval_infix_expr(&operator, left_obj, right_obj)
            }
            Expr::IfExpr {
                token: _,
                condition: _,
                consequence: _,
                alternative: _,
            } => eval_if_expr(expr, env),
            Expr::Ident(ident) => eval_ident(ident, env),
            Expr::FuncLite {
                token: _,
                parameters,
                body,
            } => Some(Object::Func(Func {
                parameters: parameters,
                body: body,
                env: Some(env),
            })),

            Expr::CallExpr {
                token: _,
                func,
                arguments,
            } => {
                let func = eval(Node::Expr(*func), Rc::clone(&env));
                if is_error(&func) {
                    return func;
                }
                let args = eval_exprs(arguments, env);
                if args.len() == 1 && is_error(&args[0]) {
                    return args[0].clone();
                }
                apply_func(func, &args)
            }
            Expr::StrLite { token: _, value } => Some(Object::Str(Str { value: value })),
            Expr::ArrayLite { token: _, elements } => {
                let elemts = eval_exprs(elements, env);
                if elemts.len() == 1 && is_error(&elemts[0]) {
                    return elemts[0].clone();
                }
                Some(Object::Array { elements: elemts })
            }
            Expr::IndexExpr {
                token: _,
                left,
                index,
            } => {
                let left_obj = eval(Node::Expr(*left), Rc::clone(&env));
                if is_error(&left_obj) {
                    return left_obj;
                }
                let index_obj = eval(Node::Expr(*index), Rc::clone(&env));
                if is_error(&index_obj) {
                    return index_obj;
                }
                eval_index_expr(left_obj, index_obj)
            }
            Expr::HashLite(hash_lite) => eval_hash_lite(&hash_lite, Rc::clone(&env)),
        },
    }
}

fn eval_program(program: Program, env: Rc<RefCell<Env>>) -> Option<Object> {
    println!("eval_program: {:?}", program);
    let mut result: Option<Object> = None;
    for stmt in program.stmts.iter() {
        result = eval(Node::Stmt(stmt.clone()), Rc::clone(&env));

        if let Some(Object::ReturnValue { value }) = result {
            return *value;
        }
        if let Some(Object::Error { message }) = result {
            return Some(Object::Error { message: message });
        }
    }
    result
}

fn native_bool_to_boolean_obj(input: bool) -> Object {
    if input {
        return TRUE;
    }
    return FALSE;
}

fn eval_prefix_expr(operator: &str, right: Option<Object>) -> Option<Object> {
    println!("eval_prefix_expr: {} {:?}", operator, right);
    match operator {
        "!" => eval_bang_operator_expr(right),
        "-" => eval_minus_operator_expr(right),
        _ => new_error(format!(
            "unknown operator: {}{}",
            operator,
            right.unwrap().get_type()
        )),
    }
}

fn eval_bang_operator_expr(right: Option<Object>) -> Option<Object> {
    println!("eval_bang_operator_expr: {:?}", right);
    match right {
        Some(Object::Bool(Bool { value })) => {
            if value {
                Some(FALSE)
            } else {
                Some(TRUE)
            }
        }
        Some(Object::Null {}) => Some(TRUE),
        _ => Some(FALSE),
    }
}

fn eval_minus_operator_expr(right: Option<Object>) -> Option<Object> {
    println!("eval_minus_operator_expr: {:?}", right);
    if let Some(Object::Int(Int { value })) = right {
        return Some(Object::Int(Int { value: -value }));
    } else {
        return new_error(format!("unknown operator: -{}", right.unwrap().get_type()));
    }
}

fn eval_infix_expr(operator: &str, left: Option<Object>, right: Option<Object>) -> Option<Object> {
    println!("eval_infix_expr: {} {:?} {:?}", operator, left, right);
    if let Some(Object::Int(Int { value: _ })) = left {
        if let Some(Object::Int(Int { value: _ })) = right {
            return eval_int_infix_expr(&operator, left, right);
        }
    }

    if let Some(Object::Str(Str { value: _ })) = left {
        if let Some(Object::Str(Str { value: _ })) = right {
            return eval_str_infix_expr(&operator, left, right);
        }
    }

    return match operator {
        "==" => Some(native_bool_to_boolean_obj(left == right)),
        "!=" => Some(native_bool_to_boolean_obj(left != right)),
        _ => {
            if left.as_ref().unwrap().get_type() != right.as_ref().unwrap().get_type() {
                new_error(format!(
                    "type mismatch: {} {} {}",
                    left.unwrap().get_type(),
                    operator,
                    right.unwrap().get_type()
                ))
            } else {
                new_error(format!(
                    "unknown operator: {} {} {}",
                    left.unwrap().get_type(),
                    operator,
                    right.unwrap().get_type()
                ))
            }
        }
    };
}

fn eval_int_infix_expr(
    operator: &str,
    left: Option<Object>,
    right: Option<Object>,
) -> Option<Object> {
    println!("eval_int_infix_expr: {} {:?} {:?}", operator, left, right);
    if let Some(Object::Int(Int { value })) = left {
        let left_val = value;
        if let Some(Object::Int(Int { value })) = right {
            let right_val = value;
            return match operator {
                "+" => Some(Object::Int(Int {
                    value: left_val + right_val,
                })),
                "-" => Some(Object::Int(Int {
                    value: left_val - right_val,
                })),
                "*" => Some(Object::Int(Int {
                    value: left_val * right_val,
                })),
                "/" => Some(Object::Int(Int {
                    value: left_val / right_val,
                })),
                "<" => Some(native_bool_to_boolean_obj(left_val < right_val)),
                ">" => Some(native_bool_to_boolean_obj(left_val > right_val)),
                "==" => Some(native_bool_to_boolean_obj(left_val == right_val)),
                "!=" => Some(native_bool_to_boolean_obj(left_val != right_val)),
                _ => new_error(format!(
                    "unknown operator: {} {} {}",
                    left.unwrap().get_type(),
                    operator,
                    right.unwrap().get_type()
                )),
            };
        }
    }
    None
}

fn eval_if_expr(expr: Expr, env: Rc<RefCell<Env>>) -> Option<Object> {
    println!("eval_if_expr: {:?}", expr);
    if let Expr::IfExpr {
        token: _,
        condition,
        consequence,
        alternative,
    } = expr
    {
        let condition_obj = eval(Node::Expr(*condition), Rc::clone(&env));
        if is_error(&condition_obj) {
            return condition_obj;
        }

        if is_truthy(condition_obj) {
            return eval(Node::Stmt(Stmt::BlockStmt(consequence)), env);
        } else {
            if let Some(a) = alternative {
                return eval(Node::Stmt(Stmt::BlockStmt(a)), env);
            } else {
                return Some(NULL);
            }
        }
    }
    None
}

fn is_truthy(obj: Option<Object>) -> bool {
    println!("is_truthy: {:?}", obj);
    match obj {
        Some(NULL) | Some(FALSE) => false,
        _ => true,
    }
}

fn eval_block_stmt(block: BlockStmt, env: Rc<RefCell<Env>>) -> Option<Object> {
    println!("eval_block_stmt: {:?}", block);
    let mut result: Option<Object> = None;
    for stmt in block.stmts {
        result = eval(Node::Stmt(stmt), Rc::clone(&env));

        if let Some(Object::ReturnValue { value }) = result {
            return Some(Object::ReturnValue { value });
        }
        if let Some(Object::Error { message }) = result {
            return Some(Object::Error { message: message });
        }
    }
    result
}

fn new_error(message: String) -> Option<Object> {
    Some(Object::Error { message: message })
}

fn is_error(obj: &Option<Object>) -> bool {
    if let Some(Object::Error { message: _ }) = &obj {
        return true;
    }
    false
}

fn eval_ident(ident: Ident, env: Rc<RefCell<Env>>) -> Option<Object> {
    println!("eval_ident: {:?}", ident);
    let (val, ok) = env.borrow().get(ident.value.clone());
    if ok {
        return val;
    } else {
        if let Some(builtin) = get_builtin(&ident.value) {
            return Some(builtin);
        }
    }
    new_error(format!("identifier not found: {}", ident.value))
}

fn eval_exprs(exps: Vec<Option<Expr>>, env: Rc<RefCell<Env>>) -> Vec<Option<Object>> {
    println!("eval_exprs: {:?}", exps);
    let mut result: Vec<Option<Object>> = Vec::new();
    for e in exps.iter() {
        let evaluated = eval(Node::Expr(e.as_ref().unwrap().clone()), Rc::clone(&env));
        if is_error(&evaluated) {
            return vec![evaluated];
        }
        result.push(evaluated);
    }
    result
}

fn apply_func(func: Option<Object>, args: &Vec<Option<Object>>) -> Option<Object> {
    println!("apply_func: {:?} {:?}", func, args);
    if let Some(Object::Func(function)) = func {
        let extended_env = extend_func_env(function.clone(), args);
        let evaluated = eval(Node::Stmt(Stmt::BlockStmt(function.body)), extended_env);
        unwrap_return_value(evaluated)
    } else if let Some(Object::Builtin(Builtin { func })) = func {
        func(args)
    } else {
        new_error(format!("not a function: {:?}", func))
    }
}

fn extend_func_env(func: Func, args: &Vec<Option<Object>>) -> Rc<RefCell<Env>> {
    let env = Rc::new(RefCell::new(new_enclosed_env(func.env)));
    for (param_idx, param) in func.parameters.iter().enumerate() {
        env.borrow_mut()
            .set(param.value.clone(), args[param_idx].clone());
    }
    env
}

fn unwrap_return_value(obj: Option<Object>) -> Option<Object> {
    if let Some(Object::ReturnValue { value }) = obj {
        return *value;
    }
    obj
}

fn eval_str_infix_expr(
    operator: &str,
    left: Option<Object>,
    right: Option<Object>,
) -> Option<Object> {
    if operator != "+" {
        return new_error(format!(
            "unknown operator: {:?} {} {:?}",
            left, operator, right
        ));
    }

    if let Some(Object::Str(Str { value })) = left {
        let left_val = value;
        if let Some(Object::Str(Str { value })) = right {
            let right_val = value;
            return Some(Object::Str(Str {
                value: format!("{}{}", left_val, right_val),
            }));
        }
    }

    new_error(format!("unknown error"))
}

fn get_builtin(val: &str) -> Option<Object> {
    match val {
        "len" => {
            let func: fn(&Vec<Option<Object>>) -> Option<Object> = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }

                if let Some(Object::Array { elements }) = &args[0] {
                    Some(Object::Int(Int {
                        value: elements.len() as i64,
                    }))
                } else if let Some(Object::Str(Str { value })) = &args[0] {
                    Some(Object::Int(Int {
                        value: value.len() as i64,
                    }))
                } else {
                    new_error(format!(
                        "argument to `len` not supported. got {}",
                        args[0].as_ref().unwrap().get_type()
                    ))
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "first" => {
            let func: fn(&Vec<Option<Object>>) -> Option<Object> = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }

                if let Some(Object::Array { elements }) = &args[0] {
                    if elements.len() > 0 {
                        return elements[0].clone();
                    }
                    return Some(NULL);
                } else {
                    return new_error(format!(
                        "argument to `first` must be ARRAY, got {}",
                        args[0].as_ref().unwrap().get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "last" => {
            let func: fn(&Vec<Option<Object>>) -> Option<Object> = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }

                if let Some(Object::Array { elements }) = &args[0] {
                    if elements.len() > 0 {
                        return elements[elements.len() - 1].clone();
                    }
                    return Some(NULL);
                } else {
                    return new_error(format!(
                        "argument to `last` must be ARRAY, got {}",
                        args[0].as_ref().unwrap().get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "rest" => {
            let func: fn(&Vec<Option<Object>>) -> Option<Object> = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }

                if let Some(Object::Array { elements }) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        let mut new_elemts = Vec::new();
                        new_elemts.clone_from_slice(&elements[1..length]);
                        return Some(Object::Array {
                            elements: new_elemts,
                        });
                    }
                    return Some(NULL);
                } else {
                    return new_error(format!(
                        "argument to `rest` must be ARRAY, got {}",
                        args[0].as_ref().unwrap().get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "push" => {
            let func: fn(&Vec<Option<Object>>) -> Option<Object> = |args| {
                if args.len() != 2 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=2",
                        args.len()
                    ));
                }

                if let Some(Object::Array { elements }) = &args[0] {
                    let mut new_elements: Vec<Option<Object>> = Vec::new();
                    new_elements.clone_from(elements);
                    new_elements.push(args[1].clone());

                    return Some(Object::Array {
                        elements: new_elements,
                    });
                } else {
                    return new_error(format!(
                        "argument to `push` must be ARRAY, got {}",
                        args[0].as_ref().unwrap().get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "puts" => {
            let func: fn(&Vec<Option<Object>>) -> Option<Object> = |args| {
                for arg in args.iter() {
                    if let Some(a) = arg {
                        println!("{}", a.inspect());
                    }
                }
                Some(NULL)
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}

fn eval_index_expr(left: Option<Object>, index: Option<Object>) -> Option<Object> {
    if let Some(Object::Array { elements }) = left.clone() {
        if let Some(Object::Int(Int { value })) = index {
            return eval_array_index_expr(elements, value);
        }
    }
    if let Some(Object::Hash(_)) = left {
        return eval_hash_index_expr(left, index);
    }
    new_error(format!("index operator not supported: {:?}", left))
}

fn eval_array_index_expr(elements: Vec<Option<Object>>, idx: i64) -> Option<Object> {
    let max = (elements.len() as i64) - 1;
    if idx < 0 || idx > max {
        return Some(NULL);
    }
    elements[idx as usize].clone()
}

fn eval_hash_lite(node: &HashLite, env: Rc<RefCell<Env>>) -> Option<Object> {
    let mut pairs: HashMap<HashKey, HashPair> = HashMap::new();

    for (key_node, value_node) in node.pairs.clone() {
        let key = eval(Node::Expr(key_node), Rc::clone(&env));
        if is_error(&key) {
            return key;
        }

        if let Some(hash_key) = &key {
            let value = eval(Node::Expr(value_node), Rc::clone(&env));
            if is_error(&value) {
                return value;
            }
            if let Some(hashed) = hash_key.hash_key() {
                pairs.insert(
                    hashed,
                    HashPair {
                        key: key.unwrap(),
                        value: value.unwrap(),
                    },
                );
            } else {
                return new_error(format!("error"));
            }
        } else {
            return new_error(format!("unusable as hash key: {:?}", key));
        }
    }
    Some(Object::Hash(Hash { pairs: pairs }))
}

fn eval_hash_index_expr(hash: Option<Object>, index: Option<Object>) -> Option<Object> {
    if let Some(Object::Hash(hash_obj)) = hash {
        if let Some(index_obj) = index {
            if let Some(key) = index_obj.hash_key() {
                if let Some(pair) = hash_obj.pairs.get(&key) {
                    return Some(pair.value.clone());
                } else {
                    return Some(NULL);
                }
            } else {
                return new_error(format!("unusable as hash key: {}", index_obj.get_type()));
            }
        } else {
            return new_error(format!("error"));
        }
    } else {
        return new_error(format!("unusable as hash object: {:?}", hash));
    }
}

#[cfg(test)]
mod tests {
    use super::super::lexer::*;
    use super::super::parser::*;
    use super::*;

    #[test]
    fn test_eval_int_expr() {
        let tests = [
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];
        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            test_int_obj(evaluated, tt.1);
        }
    }

    fn test_eval(input: &str) -> Option<Object> {
        let env = Rc::new(RefCell::new(new_env()));
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();

        eval(program.unwrap(), env)
    }

    fn test_int_obj(obj: Option<Object>, expected: i64) {
        if let Some(Object::Int(Int { value })) = obj {
            assert!(
                value == expected,
                "object has wrong value. got={}, want={}",
                value,
                expected
            );
        } else {
            assert!(false, "object is not Int. got={:?}", obj);
        }
    }

    #[test]
    fn test_eval_boolean_expr() {
        let tests = [
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            test_boolean_obj(evaluated, tt.1);
        }
    }

    fn test_boolean_obj(obj: Option<Object>, expected: bool) {
        if let Some(Object::Bool(Bool { value })) = obj {
            assert!(
                value == expected,
                "object has wrong value. got={}, want={}",
                value,
                expected
            );
        } else {
            assert!(false, "object is not Bool. got={:?}", obj);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = [
            ("!true", false),
            ("!false", true),
            ("!5", false),
            ("!!true", true),
            ("!!false", false),
            ("!!5", true),
        ];
        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            test_boolean_obj(evaluated, tt.1);
        }
    }

    #[test]
    fn test_if_else_expr() {
        let tests: [(&str, Object); 7] = [
            ("if (true) { 10 }", Object::Int(Int { value: 10 })),
            ("if (false) { 10 }", NULL),
            ("if (1) { 10 }", Object::Int(Int { value: 10 })),
            ("if (1 < 2) { 10 }", Object::Int(Int { value: 10 })),
            ("if (1 > 2) { 10 }", NULL),
            (
                "if (1 > 2) { 10 } else { 20 }",
                Object::Int(Int { value: 20 }),
            ),
            (
                "if (1 < 2) { 10 } else { 20 }",
                Object::Int(Int { value: 10 }),
            ),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            match tt.1 {
                Object::Int(Int { value }) => test_int_obj(evaluated, value),
                _ => test_null_obj(evaluated),
            }
        }
    }

    fn test_null_obj(obj: Option<Object>) {
        if let Some(Object::Null {}) = obj {
        } else {
            assert!(false, "object is not NULL. got={:?}", obj);
        }
    }

    #[test]
    fn test_return_stmts() {
        let tests = [
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            (
                "
if (10 > 1) {
    if (10 > 1) {
        return 10;
    }
    
    return 1;
}",
                10,
            ),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            test_int_obj(evaluated, tt.1);
        }
    }

    #[test]
    fn test_error_handling() {
        let tests = [
            ("5 + true;", "type mismatch: INTEGER + BOOLEAN"),
            ("5 + true; 5;", "type mismatch: INTEGER + BOOLEAN"),
            ("-true", "unknown operator: -BOOLEAN"),
            ("true + false;", "unknown operator: BOOLEAN + BOOLEAN"),
            ("5; true + false; 5", "unknown operator: BOOLEAN + BOOLEAN"),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            (
                "
if (10 > 1) { 
    if (10 > 1) {
        return true + false;
    }
    return 1;
}",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            ("foobar", "identifier not found: foobar"),
            (
                r#"{"name": "Monkey"}[fn(x){x}];"#,
                "unusable as hash key: FUNCTION",
            ),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            if let Some(Object::Error { message }) = evaluated {
                assert!(
                    message == tt.1,
                    "wrong error message. expected={}, got={}",
                    tt.1,
                    message
                );
            } else {
                assert!(false, "no error object returned. got={:?}", evaluated);
            }
        }
    }

    #[test]
    fn test_let_stmts() {
        let tests = [
            ("let a = 5; a;", 5),
            ("let a = 5 * 5; a;", 25),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
        ];
        for tt in tests.iter() {
            test_int_obj(test_eval(tt.0), tt.1);
        }
    }

    #[test]
    fn test_func_obj() {
        let input = "fn(x) { x + 2; };";
        let evaluated = test_eval(input);
        if let Some(Object::Func(Func {
            parameters,
            body,
            env: _,
        })) = evaluated
        {
            assert!(
                parameters.len() == 1,
                "function has wrong parameters. parameters={:?}",
                parameters
            );
            assert!(
                parameters[0].string() == "x",
                "parameter is not 'x'. got={}",
                parameters[0].string()
            );
            let expected_body = "(x + 2)";
            assert!(
                body.string() == expected_body,
                "body is not {}, got={}",
                expected_body,
                body.string()
            );
        } else {
            assert!(false, "object is not Function. got={:?}", evaluated);
        }
    }

    #[test]
    fn test_func_application() {
        let tests = [
            ("let identity = fn(x) { x; }; identity(5);", 5),
            ("let identity = fn(x) { return x; }; identity(5);", 5),
            ("let double = fn(x) { x * 2; }; double(5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
            ("fn(x) {x;}(5)", 5),
        ];

        for tt in tests.iter() {
            test_int_obj(test_eval(tt.0), tt.1);
        }
    }

    #[test]
    fn test_closures() {
        let input = "
let newAdder = fn(x) { 
    fn(y) { x + y };
};
let addTwo = newAdder(2); 
addTwo(2);
";
        test_int_obj(test_eval(input), 4);
    }

    #[test]
    fn test_str_lite() {
        let input = r#""Hello World!""#;
        let evaluated = test_eval(input);
        if let Some(Object::Str(Str { value })) = evaluated {
            assert!(
                value == "Hello World!",
                "String has wrong value. got={}",
                value
            );
        } else {
            assert!(false, "object id not String. got={:?}", evaluated);
        }
    }

    #[test]
    fn test_str_concat() {
        let input = r#""Hello" + " " + "World!""#;
        let evaluated = test_eval(input);
        if let Some(Object::Str(Str { value })) = evaluated {
            assert!(
                value == "Hello World!",
                "String has wrong value. got={}",
                value
            );
        } else {
            assert!(false, "object is not String. got={:?}", evaluated);
        }
    }

    #[test]
    fn test_builtin_funcs() {
        let tests: [(&str, Object); 5] = [
            (r#"len("")"#, Object::Int(Int { value: 0 })),
            (r#"len("four")"#, Object::Int(Int { value: 4 })),
            (r#"len("hello world")"#, Object::Int(Int { value: 11 })),
            (
                r#"len(1)"#,
                Object::Str(Str {
                    value: String::from("argument to `len` not supported. got INTEGER"),
                }),
            ),
            (
                r#"len("one", "two")"#,
                Object::Str(Str {
                    value: String::from("wrong number of arguments. got=2, want=1"),
                }),
            ),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            if let Object::Int(Int { value }) = tt.1.clone() {
                test_int_obj(evaluated, value);
            } else if let Object::Str(Str { value }) = tt.1.clone() {
                if let Some(Object::Error { message }) = evaluated {
                    assert!(
                        message == value,
                        "wrong error message. expected={}, got={}",
                        value,
                        message
                    );
                } else {
                    assert!(false, "object is not Error. got={:?}", evaluated);
                }
            }
        }
    }

    #[test]
    fn test_array_lites() {
        let input = "[1, 2 * 2, 3 + 3]";
        let evaluated = test_eval(input);
        if let Some(Object::Array { elements }) = evaluated {
            assert!(
                elements.len() == 3,
                "array has wrong num of elements. got={}",
                elements.len()
            );

            test_int_obj(elements[0].clone(), 1);
            test_int_obj(elements[1].clone(), 4);
            test_int_obj(elements[2].clone(), 6);
        } else {
            assert!(false, "object is not Array. got={:?}", evaluated);
        }
    }

    #[test]
    fn test_array_index_expr() {
        let tests: [(&str, Object); 10] = [
            ("[1, 2, 3][0]", Object::Int(Int { value: 1 })),
            ("[1, 2, 3][1]", Object::Int(Int { value: 2 })),
            ("[1, 2, 3][2]", Object::Int(Int { value: 3 })),
            ("let i = 0; [1][i];", Object::Int(Int { value: 1 })),
            ("[1, 2, 3][1 + 1]", Object::Int(Int { value: 3 })),
            (
                "let myArray = [1, 2, 3]; myArray[2];",
                Object::Int(Int { value: 3 }),
            ),
            (
                "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
                Object::Int(Int { value: 6 }),
            ),
            (
                "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
                Object::Int(Int { value: 2 }),
            ),
            ("[1, 2, 3][3]", NULL),
            ("[1, 2, 3][-1]", NULL),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            if let Some(Object::Int(Int { value })) = evaluated {
                test_int_obj(evaluated, value);
            } else {
                test_null_obj(evaluated);
            }
        }
    }

    #[test]
    fn test_hash_lites() {
        let input = r#"let two = "two";
        {
            "one": 10 - 9,
            two: 1 + 1,
            "thr" + "ee": 6 / 2,
            4: 4,
            true: 5,
            false:6
        }"#;

        let evaluated = test_eval(input);
        if let Some(Object::Hash(result)) = evaluated {
            let mut expected: HashMap<HashKey, i64> = HashMap::new();
            expected.insert(
                Hashtable::Str(Str {
                    value: String::from("one"),
                })
                .hash_key(),
                1,
            );
            expected.insert(
                Hashtable::Str(Str {
                    value: String::from("two"),
                })
                .hash_key(),
                2,
            );
            expected.insert(
                Hashtable::Str(Str {
                    value: String::from("three"),
                })
                .hash_key(),
                3,
            );
            expected.insert(Hashtable::Int(Int { value: 4 }).hash_key(), 4);
            expected.insert(TRUE.hash_key().unwrap(), 5);
            expected.insert(FALSE.hash_key().unwrap(), 6);

            assert!(
                result.pairs.len() == expected.len(),
                "Hash has wrong num of pairs. got={}",
                result.pairs.len()
            );

            for (expected_key, expected_value) in expected.iter() {
                if let Some(pair) = result.pairs.get(expected_key) {
                    test_int_obj(Some(pair.value.clone()), *expected_value);
                } else {
                    assert!(false, "no pair for given key in pairs");
                }
            }
        } else {
            assert!(false, "Eval didn't return Hash. got={:?}", evaluated);
        }
    }

    #[test]
    fn test_hash_index_expr() {
        let tests: [(&str, Object); 7] = [
            (r#"{"foo":5}["foo"]"#, Object::Int(Int { value: 5 })),
            (r#"{"foo":5}["bar"]"#, NULL),
            (
                r#"let key="foo"; {"foo":5}[key]"#,
                Object::Int(Int { value: 5 }),
            ),
            (r#"{}["foo"]"#, NULL),
            ("{5:5}[5]", Object::Int(Int { value: 5 })),
            ("{true:5}[true]", Object::Int(Int { value: 5 })),
            ("{false:5}[false]", Object::Int(Int { value: 5 })),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            if let Object::Int(Int { value }) = tt.1 {
                test_int_obj(evaluated, value);
            } else {
                test_null_obj(evaluated);
            }
        }
    }
}
