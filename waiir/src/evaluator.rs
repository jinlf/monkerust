use super::ast::*;
use super::object::*;

const TRUE: Object = Object::Bool { value: true };
const FALSE: Object = Object::Bool { value: false };
const NULL: Object = Object::Null {};

pub fn eval(node: Node) -> Option<Object> {
    println!("eval: {:?}", node);
    match node {
        Node::Program(program) => eval_program(program),
        Node::Stmt(stmt) => match stmt {
            Stmt::ExprStmt { token: _, expr } => eval(Node::Expr(expr)),
            Stmt::BlockStmt(block_stmt) => eval_block_stmt(block_stmt),
            Stmt::ReturnStmt { token: _, value } => {
                let val = eval(Node::Expr(value));
                if is_error(&val) {
                    return val;
                }
                Some(Object::ReturnValue {
                    value: Box::new(val.unwrap()),
                })
            }
            _ => None,
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
                let right_obj = eval(Node::Expr(*right));
                if is_error(&right_obj) {
                    return right_obj;
                }
                eval_prefix_expr(&operator, right_obj.unwrap())
            }
            Expr::InfixExpr(InfixExpr {
                token: _,
                left,
                operator,
                right,
            }) => {
                let left_obj = eval(Node::Expr(*left));
                if is_error(&left_obj) {
                    return left_obj;
                }
                let right_obj = eval(Node::Expr(*right));
                if is_error(&right_obj) {
                    return right_obj;
                }
                eval_infix_expr(&operator, left_obj.unwrap(), right_obj.unwrap())
            }
            Expr::IfExpr {
                token: _,
                condition: _,
                consequence: _,
                alternative: _,
            } => eval_if_expr(expr),
            _ => None,
        },
    }
}

fn eval_program(program: Program) -> Option<Object> {
    println!("eval_program: {:?}", program);
    let mut result: Option<Object> = None;
    for stmt in program.stmts.iter() {
        result = eval(Node::Stmt(stmt.clone()));

        if let Some(Object::ReturnValue { value }) = result {
            return Some(*value);
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

fn eval_prefix_expr(operator: &str, right: Object) -> Option<Object> {
    println!("eval_prefix_expr: {} {:?}", operator, right);
    match operator {
        "!" => eval_bang_operator_expr(right),
        "-" => eval_minus_operator_expr(right),
        _ => new_error(format!(
            "unknown operator: {}{}",
            operator,
            right.get_type()
        )),
    }
}

fn eval_bang_operator_expr(right: Object) -> Option<Object> {
    println!("eval_bang_operator_expr: {:?}", right);
    match right {
        Object::Bool { value } => {
            if value {
                Some(FALSE)
            } else {
                Some(TRUE)
            }
        }
        Object::Null {} => Some(TRUE),
        _ => Some(FALSE),
    }
}

fn eval_minus_operator_expr(right: Object) -> Option<Object> {
    println!("eval_minus_operator_expr: {:?}", right);
    if right.get_type() != "INTEGER" {
        return new_error(format!("unknown operator: -{}", right.get_type()));
    }

    if let Object::Int { value } = right {
        return Some(Object::Int { value: -value });
    }
    None
}

fn eval_infix_expr(operator: &str, left: Object, right: Object) -> Option<Object> {
    println!("eval_infix_expr: {} {:?} {:?}", operator, left, right);
    if left.get_type() == "INTEGER" && right.get_type() == "INTEGER" {
        return eval_int_infix_expr(&operator, left, right);
    }
    return match operator {
        "==" => Some(native_bool_to_boolean_obj(left == right)),
        "!=" => Some(native_bool_to_boolean_obj(left != right)),
        _ => {
            if left.get_type() != right.get_type() {
                new_error(format!(
                    "type mismatch: {} {} {}",
                    left.get_type(),
                    operator,
                    right.get_type()
                ))
            } else {
                new_error(format!(
                    "unknown operator: {} {} {}",
                    left.get_type(),
                    operator,
                    right.get_type()
                ))
            }
        }
    };
}

fn eval_int_infix_expr(operator: &str, left: Object, right: Object) -> Option<Object> {
    println!("eval_int_infix_expr: {} {:?} {:?}", operator, left, right);
    if let Object::Int { value } = left {
        let left_val = value;
        if let Object::Int { value } = right {
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
                    left.get_type(),
                    operator,
                    right.get_type()
                )),
            };
        }
    }
    None
}

fn eval_if_expr(expr: Expr) -> Option<Object> {
    println!("eval_if_expr: {:?}", expr);
    if let Expr::IfExpr {
        token: _,
        condition,
        consequence,
        alternative,
    } = expr
    {
        let condition_obj = eval(Node::Expr(*condition));
        if is_error(&condition_obj) {
            return condition_obj;
        }

        if is_truthy(condition_obj.unwrap()) {
            return eval(Node::Stmt(Stmt::BlockStmt(consequence)));
        } else if alternative.is_some() {
            return eval(Node::Stmt(Stmt::BlockStmt(alternative.unwrap())));
        } else {
            return Some(NULL);
        }
    }
    None
}

fn is_truthy(obj: Object) -> bool {
    println!("is_truthy: {:?}", obj);
    match obj {
        NULL | FALSE => false,
        _ => true,
    }
}

fn eval_block_stmt(block: BlockStmt) -> Option<Object> {
    let mut result: Option<Object> = None;
    for stmt in block.stmts {
        result = eval(Node::Stmt(stmt));

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
    if let Some(Object::Error { message }) = &obj {
        return true;
    }
    false
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
            test_int_obj(evaluated.unwrap(), tt.1);
        }
    }

    fn test_eval(input: &str) -> Option<Object> {
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();

        eval(program.unwrap())
    }

    fn test_int_obj(obj: Object, expected: i64) {
        if let Object::Int { value } = obj {
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
            test_boolean_obj(evaluated.unwrap(), tt.1);
        }
    }

    fn test_boolean_obj(obj: Object, expected: bool) {
        if let Object::Bool { value } = obj {
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
            test_boolean_obj(evaluated.unwrap(), tt.1);
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
                Object::Int { value } => test_int_obj(evaluated.unwrap(), value),
                _ => test_null_obj(evaluated.unwrap()),
            }
        }
    }

    fn test_null_obj(obj: Object) {
        if let Object::Null {} = obj {
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
            test_int_obj(evaluated.unwrap(), tt.1);
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
}
