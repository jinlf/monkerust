// src/evaluator_test.rs

use super::ast::*;
use super::environment::*;
use super::evaluator::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use std::cell::*;
use std::collections::*;
use std::rc::*;

#[test]
fn test_eval_integer_expression() {
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
        test_integer_object(evaluated, tt.1);
    }
}

fn test_eval(input: &str) -> Option<Object> {
    let env = Rc::new(RefCell::new(new_environment()));
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    eval(Node::Program(program.unwrap()), Rc::clone(&env))
}

fn test_integer_object(obj: Option<Object>, expected: i64) {
    if let Some(Object::Integer(Integer { value })) = obj {
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
fn test_eval_boolean_expression() {
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
        test_boolean_object(evaluated, tt.1);
    }
}

fn test_boolean_object(obj: Option<Object>, expected: bool) {
    if let Some(Object::Boolean(Boolean { value })) = obj {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        assert!(false, "object is not BooleanLiteral. got={:?}", obj);
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
        test_boolean_object(evaluated, tt.1);
    }
}

#[test]
fn test_if_else_expression() {
    let tests: [(&str, Option<Object>); 7] = [
        (
            "if (true) { 10 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
        ("if (false) { 10 }", None),
        (
            "if (1) { 10 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
        (
            "if (1 < 2) { 10 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
        ("if (1 > 2) { 10 }", None),
        (
            "if (1 > 2) { 10 } else { 20 }",
            Some(Object::Integer(Integer { value: 20 })),
        ),
        (
            "if (1 < 2) { 10 } else { 20 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Some(Object::Integer(Integer { value })) = tt.1 {
            test_integer_object(evaluated, value);
        } else {
            test_null_object(evaluated);
        }
    }
}

fn test_null_object(obj: Option<Object>) {
    assert!(
        obj == Some(Object::Null(NULL)),
        "object is not NULL, got={:?}",
        obj
    );
}

#[test]
fn test_return_statements() {
    let tests = [
        ("return 10;", 10),
        ("return 10; 9;", 10),
        ("return 2 * 5; 9;", 10),
        ("9; return 2 * 5; 9;", 10),
        (
            "if (10 > 1) {
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
        test_integer_object(evaluated, tt.1);
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
        (r#""Hello" - "World""#, "unknown operator: STRING - STRING"),
        (
            r#"{"name": "Monkey"}[fn(x){ x }];"#,
            "unusable as hash key: FUNCTION",
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Some(Object::ErrorObj(ErrorObj { message })) = evaluated {
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
fn test_let_statements() {
    let tests = [
        ("let a = 5; a;", 5),
        ("let a = 5 * 5; a;", 25),
        ("let a = 5; let b = a; b;", 5),
        ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
    ];

    for tt in tests.iter() {
        test_integer_object(test_eval(tt.0), tt.1);
    }
}

#[test]
fn test_function_object() {
    let input = "fn(x) { x + 2; };";
    let evaluated = test_eval(input);
    if let Some(Object::Function(Function {
        parameters,
        body,
        env: _,
    })) = evaluated
    {
        assert!(
            parameters.len() == 1,
            "function has wrong parameters. got={}",
            parameters.len()
        );

        assert!(
            parameters[0].string() == "x",
            "parameter is not 'x'. got={:?}",
            parameters[0]
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
fn test_function_application() {
    let tests = [
        ("let identity = fn(x) { x; }; identity(5);", 5),
        ("let identity = fn(x) { return x; }; identity(5);", 5),
        ("let double = fn(x) { x * 2; }; double(5);", 10),
        ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
        ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
        ("fn(x) { x; }(5)", 5),
    ];

    for tt in tests.iter() {
        test_integer_object(test_eval(tt.0), tt.1);
    }
}

#[test]
fn test_closures() {
    let input = "
let newAdder = fn(x) {
fn(y) { x + y };
};
let addTwo = newAdder(2);
addTwo(2);";
    test_integer_object(test_eval(input), 4);
}

#[test]
fn test_string_literal() {
    let input = r#""Hello World!""#;
    let evaluated = test_eval(input);
    if let Some(Object::StringObj(StringObj { value })) = evaluated {
        assert!(
            value == "Hello World!",
            "String has wrong value. got={:?}",
            value
        );
    } else {
        assert!(false, "object is not String. got={:?}", evaluated);
    }
}

#[test]
fn test_string_concatenation() {
    let input = r#""Hello" + " " + "World!""#;
    let evaluated = test_eval(input);
    if let Some(Object::StringObj(StringObj { value })) = evaluated {
        assert!(
            value == "Hello World!",
            "String has wrong value. got={:?}",
            value
        );
    } else {
        assert!(false, "object is not String. got={:?}", evaluated);
    }
}

#[test]
fn test_builtin_functions() {
    let tests: [(&str, Object); 5] = [
        (r#"len("")"#, Object::Integer(Integer { value: 0 })),
        (r#"len("four")"#, Object::Integer(Integer { value: 4 })),
        (
            r#"len("hello world")"#,
            Object::Integer(Integer { value: 11 }),
        ),
        (
            r#"len(1)"#,
            Object::ErrorObj(ErrorObj {
                message: String::from("argument to `len` not supported, got INTEGER"),
            }),
        ),
        (
            r#"len("one", "two")"#,
            Object::ErrorObj(ErrorObj {
                message: String::from("wrong number of arguments. got=2, want=1"),
            }),
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Object::Integer(Integer { value }) = &tt.1 {
            test_integer_object(evaluated, *value);
        } else if let Object::ErrorObj(ErrorObj { message }) = &tt.1 {
            let expected_message = message;
            if let Some(Object::ErrorObj(ErrorObj { message })) = evaluated {
                assert!(
                    message == *expected_message,
                    "wrong error message. expected={:?}, got={:?}",
                    expected_message,
                    message
                );
            } else {
                assert!(false, "object is not Error. got={:?}", evaluated);
            }
        }
    }
}

#[test]
fn test_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";
    let evaluated = test_eval(input);
    if let Some(Object::Array(Array { elements })) = evaluated {
        assert!(
            elements.len() == 3,
            "array has wrong num of elments. got={}",
            elements.len()
        );

        test_integer_object(Some(elements[0].clone()), 1);
        test_integer_object(Some(elements[1].clone()), 4);
        test_integer_object(Some(elements[2].clone()), 6);
    } else {
        assert!(false, "object is not Array. got={:?}", evaluated);
    }
}

#[test]
fn test_array_index_expressions() {
    let tests: [(&str, Object); 10] = [
        ("[1, 2, 3][0]", Object::Integer(Integer { value: 1 })),
        ("[1, 2, 3][1]", Object::Integer(Integer { value: 2 })),
        ("[1, 2, 3][2]", Object::Integer(Integer { value: 3 })),
        ("let i = 0; [1][i];", Object::Integer(Integer { value: 1 })),
        ("[1, 2, 3][1 + 1]", Object::Integer(Integer { value: 3 })),
        (
            "let myArray = [1, 2, 3]; myArray[2];",
            Object::Integer(Integer { value: 3 }),
        ),
        (
            "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
            Object::Integer(Integer { value: 6 }),
        ),
        (
            "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
            Object::Integer(Integer { value: 2 }),
        ),
        ("[1, 2, 3][3]", Object::Null(NULL)),
        ("[1, 2, 3][-1]", Object::Null(NULL)),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Object::Integer(Integer { value }) = tt.1 {
            test_integer_object(evaluated, value);
        } else {
            test_null_object(evaluated);
        }
    }
}

#[test]
fn test_hash_literals() {
    let input = r#"let two = "two";
    {
        "one": 10 - 9,
        two: 1 + 1,
        "thr" + "ee": 6 / 2,
        4: 4,
        true: 5,
        false: 6
    }"#;
    let evaluated = test_eval(input);
    if let Some(Object::Hash(Hash { pairs })) = evaluated {
        let mut expected: HashMap<HashKey, i64> = HashMap::new();
        expected.insert(
            StringObj {
                value: String::from("one"),
            }
            .hash_key(),
            1,
        );
        expected.insert(
            StringObj {
                value: String::from("two"),
            }
            .hash_key(),
            2,
        );
        expected.insert(
            StringObj {
                value: String::from("three"),
            }
            .hash_key(),
            3,
        );
        expected.insert(Integer { value: 4 }.hash_key(), 4);
        expected.insert(TRUE.hash_key(), 5);
        expected.insert(FALSE.hash_key(), 6);

        assert!(
            pairs.len() == expected.len(),
            "Hash has wrong num of pairs. got={}",
            pairs.len()
        );
        for (expected_key, expected_value) in expected.iter() {
            if let Some(pair) = pairs.get(expected_key) {
                test_integer_object(Some(pair.clone()), *expected_value);
            } else {
                assert!(false, "no pair for given key in pairs");
            }
        }
    } else {
        assert!(false, "eval didn't return Hash. got={:?}", evaluated);
    }
}

#[test]
fn test_hash_index_expressions() {
    let tests: [(&str, Option<Object>); 7] = [
        (
            r#"{"foo": 5}["foo"]"#,
            Some(Object::Integer(Integer { value: 5 })),
        ),
        (r#"{"foo": 5}["bar"]"#, Some(Object::Null(NULL))),
        (
            r#"let key = "foo"; {"foo": 5}[key]"#,
            Some(Object::Integer(Integer { value: 5 })),
        ),
        (r#"{}["foo"]"#, Some(Object::Null(NULL))),
        ("{5: 5} [5]", Some(Object::Integer(Integer { value: 5 }))),
        (
            "{true: 5}[true]",
            Some(Object::Integer(Integer { value: 5 })),
        ),
        (
            "{false: 5}[false]",
            Some(Object::Integer(Integer { value: 5 })),
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Some(Object::Integer(integer)) = &tt.1 {
            test_integer_object(evaluated, integer.value);
        } else {
            test_null_object(evaluated);
        }
    }
}
