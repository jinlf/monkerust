use super::ast::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use super::token::*;

const TRUE: Object = Object::Boolean { value: true };
const FALSE: Object = Object::Boolean { value: false };
const NULL: Object = Object::Null {};

pub fn eval(node: Node) -> Option<Object> {
    println!("eval: {:?}", node);
    match node {
        Node::Program(program) => eval_stmts(program.stmts),
        Node::Stmt(stmt) => match stmt {
            Stmt::ExprStmt { token: _, expr } => eval(Node::Expr(expr)),
            _ => None,
        },
        Node::Expr(expr) => match expr {
            Expr::IntegerLiteral(int_lite) => Some(Object::Integer {
                value: int_lite.value,
            }),
            Expr::Boolean { token: _, value } => Some(native_bool_to_boolean_obj(value)),
            Expr::PrefixExpr(PrefixExpr {
                token,
                operator,
                right,
            }) => {
                let right_obj = eval(Node::Expr(*right));
                eval_prefix_expr(&operator, right_obj.unwrap())
            }
            _ => None,
        },
    }
}

fn eval_stmts(stmts: Vec<Stmt>) -> Option<Object> {
    println!("eval: {:?}", stmts);
    let mut result: Option<Object> = None;
    for stmt in stmts.iter() {
        result = eval(Node::Stmt(stmt.clone()));
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
        _ => None,
    }
}

fn eval_bang_operator_expr(right: Object) -> Option<Object> {
    println!("eval_bang_operator_expr: {:?}", right);
    match right {
        Object::Boolean { value } => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_int_expr() {
        let tests = [("5", 5), ("10", 10)];
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
        if let Object::Integer { value } = obj {
            assert!(
                value == expected,
                "object has wrong value. got={}, want={}",
                value,
                expected
            );
        } else {
            assert!(false, "object is not Integer. got={:?}", obj);
        }
    }

    #[test]
    fn test_eval_boolean_expr() {
        let tests = [("true", true), ("false", false)];

        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            test_boolean_obj(evaluated.unwrap(), tt.1);
        }
    }

    fn test_boolean_obj(obj: Object, expected: bool) {
        if let Object::Boolean { value } = obj {
            assert!(
                value == expected,
                "object has wrong value. got={}, want={}",
                value,
                expected
            );
        } else {
            assert!(false, "object is not Boolean. got={:?}", obj);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = [
            ("!true", false),
            // ("!false", true),
            // ("!5", false),
            // ("!!true", true),
            // ("!!false", false),
            // ("!!5", true),
        ];
        for tt in tests.iter() {
            let evaluated = test_eval(tt.0);
            test_boolean_obj(evaluated.unwrap(), tt.1);
        }
    }
}
