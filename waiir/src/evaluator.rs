use super::ast::*;
use super::env::*;
use super::object::*;
use std::cell::*;
use std::rc::*;

pub const TRUE: Object = Object::Bool { value: true };
pub const FALSE: Object = Object::Bool { value: false };
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
            Expr::IntLiteral(int_lite) => Some(Object::Int {
                value: int_lite.value,
            }),
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
        Some(Object::Bool { value }) => {
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
    if let Some(Object::Int { value }) = right {
        return Some(Object::Int { value: -value });
    } else {
        return new_error(format!("unknown operator: -{}", right.unwrap().get_type()));
    }
}

fn eval_infix_expr(operator: &str, left: Option<Object>, right: Option<Object>) -> Option<Object> {
    println!("eval_infix_expr: {} {:?} {:?}", operator, left, right);
    if let Some(Object::Int { value: _ }) = left {
        if let Some(Object::Int { value: _ }) = right {
            return eval_int_infix_expr(&operator, left, right);
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
    if let Some(Object::Int { value }) = left {
        let left_val = value;
        if let Some(Object::Int { value }) = right {
            let right_val = value;
            return match operator {
                "+" => Some(Object::Int {
                    value: left_val + right_val,
                }),
                "-" => Some(Object::Int {
                    value: left_val - right_val,
                }),
                "*" => Some(Object::Int {
                    value: left_val * right_val,
                }),
                "/" => Some(Object::Int {
                    value: left_val / right_val,
                }),
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
    if !ok {
        new_error(format!("identifier not found: {}", ident.value))
    } else {
        val
    }
}

fn eval_exprs(exps: Vec<Expr>, env: Rc<RefCell<Env>>) -> Vec<Option<Object>> {
    println!("eval_exprs: {:?}", exps);
    let mut result: Vec<Option<Object>> = Vec::new();
    for e in exps.iter() {
        let evaluated = eval(Node::Expr(e.clone()), Rc::clone(&env));
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
        if let Some(Object::Int { value }) = obj {
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
        if let Some(Object::Bool { value }) = obj {
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
            ("if (true) { 10 }", Object::Int { value: 10 }),
            ("if (false) { 10 }", NULL),
            ("if (1) { 10 }", Object::Int { value: 10 }),
            ("if (1 < 2) { 10 }", Object::Int { value: 10 }),
            ("if (1 > 2) { 10 }", NULL),
            ("if (1 > 2) { 10 } else { 20 }", Object::Int { value: 20 }),
            ("if (1 < 2) { 10 } else { 20 }", Object::Int { value: 10 }),
        ];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            match tt.1 {
                Object::Int { value } => test_int_obj(evaluated, value),
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
}
