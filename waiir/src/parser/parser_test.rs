// src/parser_test.rs

use crate::ast::*;
use crate::lexer::*;
use crate::parser::*;
use std::collections::*;

fn panic_with_errors(errors: Vec<String>) {
    let mut messages = Vec::new();
    for msg in errors.into_iter() {
        messages.push(msg);
    }
    panic!("parser error: {}", messages.join("\n"));
}

#[test]
fn test_let_statements() {
    let input = "
let x = 5;
let y = 10;
let foobar = 838383;
";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 3,
                "program.statements does not contain 3 statements. got={}",
                statements.len()
            );

            let tests = ["x", "y", "foobar"];
            for (i, tt) in tests.iter().enumerate() {
                test_let_statement(&statements[i], tt);
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

fn test_let_statement(s: &Statement, expected_name: &str) {
    assert!(
        &s.string()[0..3] == "let",
        "s.token_literal not 'let'. got={}",
        s.string()
    );

    if let Statement::LetStatement(LetStatement {
        token: _,
        name,
        value: _,
    }) = s
    {
        assert!(
            name.value == expected_name,
            "letStmt.name.value not '{}', got={}",
            expected_name,
            name.value
        );

        assert!(
            name.token.literal == expected_name,
            "s.name not '{}'. got={}",
            expected_name,
            name.token.literal
        );
    } else {
        panic!("s not LetStatement. got={:?}", s);
    }
}

#[test]
fn test_return_statement() {
    let input = "
return 5;
return 10;
return 993322;
";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 3,
                "program.statements does not contain 3 statements. got={}",
                statements.len()
            );

            for stmt in statements.iter() {
                if let Statement::ReturnStatement(_) = stmt {
                    assert!(
                        &stmt.string()[..6] == "return",
                        "returnStmt.token_literal not 'return', got={}",
                        stmt.string(),
                    );
                } else {
                    panic!("stmt not ReturnStatement. got={:?}", stmt);
                }
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_identifier_expression() {
    let input = "foobar;";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 1,
                "program has not enough statements. got={}",
                statements.len()
            );
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::Identifier(Identifier { token, value }) = expression {
                    assert!(
                        value == "foobar",
                        "ident.value not {}. got={}",
                        "foobar",
                        value
                    );
                    assert!(
                        token.literal == "foobar",
                        "ident.token_literal not {}. got={}",
                        "foobar",
                        token.literal
                    );
                } else {
                    panic!("exp not Identifier. got={:?}", expression);
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_integer_literal_expression() {
    let input = "5;";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 1,
                "program has not enough statements. got={}",
                statements.len()
            );

            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::IntegerLiteral(IntegerLiteral { token, value }) = expression {
                    assert!(*value == 5, "literal.value not {}. got={}", 5, value);
                    assert!(
                        token.literal == "5",
                        "literal.token_literal not {}. got={}",
                        "5",
                        token.literal
                    );
                } else {
                    panic!("exp not IntegerLiteral. got={:?}", expression);
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

enum ExpectedType {
    Ival(i64),
    Sval(String),
    Bval(bool),
}
impl From<i64> for ExpectedType {
    fn from(v: i64) -> Self {
        ExpectedType::Ival(v)
    }
}
impl From<&str> for ExpectedType {
    fn from(v: &str) -> Self {
        ExpectedType::Sval(v.to_owned())
    }
}
impl From<bool> for ExpectedType {
    fn from(v: bool) -> Self {
        ExpectedType::Bval(v)
    }
}

#[test]
fn test_parsing_prefix_expression() {
    let tests = vec![
        ("!5;", "!", ExpectedType::from(5)),
        ("-15;", "-", ExpectedType::from(15)),
        ("!true", "!", ExpectedType::from(true)),
        ("!false", "!", ExpectedType::from(false)),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        match p.parse_program() {
            Ok(Program { statements }) => {
                assert!(
                    statements.len() == 1,
                    "program.statements does not contain {} statements. got={}",
                    1,
                    statements.len()
                );

                if let Statement::ExpressionStatement(ExpressionStatement {
                    token: _,
                    expression,
                }) = &statements[0]
                {
                    if let Expression::PrefixExpression(PrefixExpression {
                        token: _,
                        operator,
                        right,
                    }) = expression
                    {
                        assert!(
                            operator == tt.1,
                            "exp.operator is not '{}'. got={}",
                            tt.1,
                            operator
                        );

                        test_literal_expression(right, &tt.2);
                    } else {
                        panic!("stmt is not PrefixExpression. got={:?}", expression);
                    }
                } else {
                    assert!(
                        false,
                        "program.statements[0] is not ExpressionStatement. got={:?}",
                        &statements[0]
                    );
                }
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}

fn test_integer_literal(il: &Expression, expected_value: i64) {
    if let Expression::IntegerLiteral(IntegerLiteral { token, value }) = il {
        assert!(
            *value == expected_value,
            "integ.value not {}. got={}",
            expected_value,
            value
        );

        assert!(
            token.literal == format!("{}", expected_value),
            "integ.token_literal not {}. got={}",
            value,
            token.literal
        );
    } else {
        panic!("il not IntegerLiteral. got={:?}", il);
    }
}

#[test]
fn test_parsing_infix_expressions() {
    let tests = vec![
        ("5 + 5;", ExpectedType::from(5), "+", ExpectedType::from(5)),
        ("5 - 5;", ExpectedType::from(5), "-", ExpectedType::from(5)),
        ("5 * 5;", ExpectedType::from(5), "*", ExpectedType::from(5)),
        ("5 / 5;", ExpectedType::from(5), "/", ExpectedType::from(5)),
        ("5 > 5;", ExpectedType::from(5), ">", ExpectedType::from(5)),
        ("5 < 5;", ExpectedType::from(5), "<", ExpectedType::from(5)),
        (
            "5 == 5;",
            ExpectedType::from(5),
            "==",
            ExpectedType::from(5),
        ),
        (
            "5 != 5;",
            ExpectedType::from(5),
            "!=",
            ExpectedType::from(5),
        ),
        (
            "true == true",
            ExpectedType::from(true),
            "==",
            ExpectedType::from(true),
        ),
        (
            "true != false",
            ExpectedType::from(true),
            "!=",
            ExpectedType::from(false),
        ),
        (
            "false == false",
            ExpectedType::from(false),
            "==",
            ExpectedType::from(false),
        ),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        match p.parse_program() {
            Ok(Program { statements }) => {
                assert!(
                    statements.len() == 1,
                    "program.statements does not contain {} statements. got={}",
                    1,
                    statements.len()
                );
                if let Statement::ExpressionStatement(ExpressionStatement {
                    token: _,
                    expression,
                }) = &statements[0]
                {
                    if let Expression::InfixExpression(InfixExpression {
                        token: _,
                        left,
                        operator,
                        right,
                    }) = expression
                    {
                        test_literal_expression(left, &tt.1);

                        assert!(
                            operator == tt.2,
                            "exp.operator is not '{}. got={}",
                            tt.2,
                            operator
                        );

                        test_literal_expression(right, &tt.3);
                    } else {
                        panic!("exp is not InfixExpression. got={:?}", expression);
                    }
                } else {
                    assert!(
                        false,
                        "program.statements[0] is not ExpressionStatement. got={:?}",
                        &statements[0]
                    );
                }
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}

#[test]
fn test_operator_precedence_parsing() {
    let tests = [
        ("-a * b", "((-a) * b)"),
        ("!-a", "(!(-a))"),
        ("a + b + c", "((a + b) + c)"),
        ("a + b - c", "((a + b) - c)"),
        ("a * b * c", "((a * b) * c)"),
        ("a * b / c", "((a * b) / c)"),
        ("a + b / c", "(a + (b / c))"),
        ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
        ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
        ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
        ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
        (
            "3 + 4 * 5 == 3 * 1 +  4 * 5",
            "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
        ),
        ("true", "true"),
        ("false", "false"),
        ("3 > 5 == false", "((3 > 5) == false)"),
        ("3 < 5 == true", "((3 < 5) == true)"),
        ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
        ("(5 + 5) * 2", "((5 + 5) * 2)"),
        ("2 / (5 + 5)", "(2 / (5 + 5))"),
        ("-(5 + 5)", "(-(5 + 5))"),
        ("!(true == true)", "(!(true == true))"),
        ("a + add(b * c) + d", "((a + add((b * c))) + d)"),
        (
            "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
            "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
        ),
        (
            "add(a + b + c * d / f + g)",
            "add((((a + b) + ((c * d) / f)) + g))",
        ),
        (
            "a * [1, 2, 3, 4][b * c] * d",
            "((a * ([1, 2, 3, 4][(b * c)])) * d)",
        ),
        (
            "add(a * b[2], b[1], 2 * [1, 2][1])",
            "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
        ),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        match p.parse_program() {
            Ok(program) => {
                let actual = program.string();
                assert!(actual == tt.1, "expected={:?}, got={:?}", tt.1, actual);
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}

fn test_identifier(exp: &Expression, expected_value: &str) {
    if let Expression::Identifier(Identifier { token, value }) = exp {
        assert!(
            *value == expected_value,
            "ident.value not {}. got={}",
            expected_value,
            value
        );

        assert!(
            token.literal == expected_value,
            "ident.token_literal not {}. got={}",
            expected_value,
            token.literal
        );
    } else {
        panic!("exp not Identifier. got={:?}", exp);
    }
}

fn test_literal_expression(exp: &Expression, expected: &ExpectedType) {
    match expected {
        ExpectedType::Ival(v) => test_integer_literal(exp, *v),
        ExpectedType::Sval(v) => test_identifier(exp, v),
        ExpectedType::Bval(v) => test_boolean_literal(exp, *v),
    }
}

fn test_infix_expression(
    exp: &Expression,
    expected_left: &ExpectedType,
    expected_operator: &str,
    expected_right: &ExpectedType,
) {
    if let Expression::InfixExpression(InfixExpression {
        token: _,
        left,
        operator,
        right,
    }) = exp
    {
        test_literal_expression(left, expected_left);
        assert!(
            *operator == expected_operator,
            "exp.operator is not '{}'. got={:?}",
            expected_operator,
            operator
        );
        test_literal_expression(right, expected_right);
    } else {
        panic!("exp is not InfixExpression. got={:?}", exp);
    }
}

fn test_boolean_literal(exp: &Expression, expected_value: bool) {
    if let Expression::BooleanLiteral(BooleanLiteral { token, value }) = exp {
        assert!(
            *value == expected_value,
            "bo.value not {}. got={}",
            expected_value,
            value
        );
        assert!(
            token.literal == format!("{}", expected_value),
            "bo.token_literal not {}. got={}",
            expected_value,
            token.literal
        );
    } else {
        panic!("exp not BooleanLiteral. got={:?}", exp);
    }
}

#[test]
fn test_if_expression() {
    let input = "if (x < y) { x }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 1,
                "program.body does not contain {} statements. got={}",
                1,
                statements.len()
            );

            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::IfExpression(IfExpression {
                    token: _,
                    condition,
                    consequence,
                    alternative,
                }) = expression
                {
                    test_infix_expression(
                        condition,
                        &ExpectedType::from("x"),
                        "<",
                        &ExpectedType::from("y"),
                    );

                    assert!(
                        consequence.statements.len() == 1,
                        "consequence is not 1 statements. got={}",
                        consequence.statements.len()
                    );

                    if let Statement::ExpressionStatement(ExpressionStatement {
                        token: _,
                        expression,
                    }) = &consequence.statements[0]
                    {
                        test_identifier(expression, "x");

                        assert!(
                            alternative.is_none(),
                            "exp alternative.statements was not None. got={:?}",
                            alternative,
                        );
                    } else {
                        assert!(
                            false,
                            "statements[0] is not ExpressionStatement. got={:?}",
                            &consequence.statements[0]
                        );
                    }
                } else {
                    assert!(
                        false,
                        "stmt.expression is not IfExpression. got={:?}",
                        expression
                    );
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_if_else_expression() {
    let input = "if (x < y) { x } else { y }";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 1,
                "program.body does not contain {} statements. got={}",
                1,
                statements.len()
            );

            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::IfExpression(IfExpression {
                    token: _,
                    condition,
                    consequence,
                    alternative,
                }) = expression
                {
                    test_infix_expression(
                        condition,
                        &ExpectedType::from("x"),
                        "<",
                        &ExpectedType::from("y"),
                    );

                    assert!(
                        consequence.statements.len() == 1,
                        "consequence is not 1 statements. got={}",
                        consequence.statements.len()
                    );

                    if let Statement::ExpressionStatement(ExpressionStatement {
                        token: _,
                        expression,
                    }) = &consequence.statements[0]
                    {
                        test_identifier(expression, "x");

                        if let Some(a) = alternative {
                            assert!(
                                a.statements.len() == 1,
                                "alternative is not 1 statements. got={}",
                                a.statements.len()
                            );
                            if let Statement::ExpressionStatement(ExpressionStatement {
                                token: _,
                                expression,
                            }) = &a.statements[0]
                            {
                                test_identifier(expression, "y");
                            } else {
                                assert!(
                                    false,
                                    "statements[0] is not ExpressionStatement. got={:?}",
                                    &a.statements[0]
                                );
                            }
                        } else {
                            panic!("exp alternative.statements was None");
                        }
                    } else {
                        assert!(
                            false,
                            "statements[0] is not ExpressionStatement. got={:?}",
                            &consequence.statements[0]
                        );
                    }
                } else {
                    assert!(
                        false,
                        "stmt.expression is not IfExpression. got={:?}",
                        expression
                    );
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_function_literal_parsing() {
    let input = "fn(x, y) { x + y; }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 1,
                "program.body does not contain {} statements. got={}",
                1,
                statements.len()
            );

            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::FunctionLiteral(FunctionLiteral {
                    token: _,
                    parameters,
                    body,
                }) = expression
                {
                    assert!(
                        parameters.len() == 2,
                        "function literal parameters wrong. want 2, got={}",
                        parameters.len()
                    );

                    test_literal_expression(
                        &Expression::Identifier(parameters[0].clone()),
                        &ExpectedType::from("x"),
                    );
                    test_literal_expression(
                        &Expression::Identifier(parameters[1].clone()),
                        &ExpectedType::from("y"),
                    );

                    assert!(
                        body.statements.len() == 1,
                        "function.body.statements has not 1 statements. got={}",
                        body.statements.len()
                    );

                    if let Statement::ExpressionStatement(ExpressionStatement {
                        token: _,
                        expression,
                    }) = &body.statements[0]
                    {
                        test_infix_expression(
                            expression,
                            &ExpectedType::from("x"),
                            "+",
                            &ExpectedType::from("y"),
                        );
                    } else {
                        assert!(
                            false,
                            "function body stmt is not ExpressionStatement. got={:?}",
                            &body.statements[0]
                        );
                    }
                } else {
                    assert!(
                        false,
                        "stmt.expression is not FunctionLiteral. got={:?}",
                        expression
                    );
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_function_parameter_parsing() {
    let tests = [
        ("fn() {};", Vec::new()),
        ("fn(x) {};", vec!["x"]),
        ("fn(x, y, z) {};", vec!["x", "y", "z"]),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        match p.parse_program() {
            Ok(Program { statements }) => {
                if let Statement::ExpressionStatement(ExpressionStatement {
                    token: _,
                    expression,
                }) = &statements[0]
                {
                    if let Expression::FunctionLiteral(FunctionLiteral {
                        token: _,
                        parameters,
                        body: _,
                    }) = expression
                    {
                        assert!(
                            parameters.len() == tt.1.len(),
                            "length parameters wrong. want {}, got={}",
                            tt.1.len(),
                            parameters.len()
                        );
                        for (i, ident) in tt.1.iter().enumerate() {
                            test_literal_expression(
                                &Expression::Identifier(parameters[i].clone()),
                                &ExpectedType::from(*ident),
                            );
                        }
                    } else {
                        panic!("parse error");
                    }
                } else {
                    panic!("parse error");
                }
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}

#[test]
fn test_call_expression_parsing() {
    let input = "add(1, 2 * 3, 4 + 5);";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 1,
                "program.statements does not contain {} statements. got={}",
                1,
                statements.len()
            );

            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::CallExpression(CallExpression {
                    token: _,
                    function,
                    arguments,
                }) = expression
                {
                    test_identifier(function, "add");

                    assert!(
                        arguments.len() == 3,
                        "wrong length of arguments. got={}",
                        arguments.len()
                    );

                    test_literal_expression(&arguments[0], &ExpectedType::from(1));
                    test_infix_expression(
                        &arguments[1],
                        &ExpectedType::from(2),
                        "*",
                        &ExpectedType::from(3),
                    );
                    test_infix_expression(
                        &arguments[2],
                        &ExpectedType::from(4),
                        "+",
                        &ExpectedType::from(5),
                    );
                } else {
                    assert!(
                        false,
                        "stmt.expression is not CallExpression. got={:?}",
                        expression
                    );
                }
            } else {
                assert!(
                    false,
                    "stmt is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_string_literal_expression() {
    let input = r#""hello world";"#;
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::StringLiteral(StringLiteral { token: _, value }) = expression {
                    assert!(
                        value == "hello world",
                        "literal.value not {}. got={}",
                        "hello world",
                        value
                    );
                } else {
                    panic!("exp not StringLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_parsing_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::ArrayLiteral(ArrayLiteral { token: _, elements }) = expression {
                    assert!(
                        elements.len() == 3,
                        "len(array.elements) not 3. got={}",
                        elements.len()
                    );
                    test_integer_literal(&elements[0], 1);
                    test_infix_expression(
                        &elements[1],
                        &ExpectedType::from(2),
                        "*",
                        &ExpectedType::from(2),
                    );
                    test_infix_expression(
                        &elements[2],
                        &ExpectedType::from(3),
                        "+",
                        &ExpectedType::from(3),
                    );
                } else {
                    panic!("exp not ArrayLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_parsing_index_expressions() {
    let input = "myArray[1 + 1]";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::IndexExpression(IndexExpression {
                    token: _,
                    left,
                    index,
                }) = expression
                {
                    test_identifier(left, "myArray");
                    test_infix_expression(
                        index,
                        &ExpectedType::from(1),
                        "+",
                        &ExpectedType::from(1),
                    );
                } else {
                    panic!("exp not IndexExpression. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_parsing_hash_literals_string_keys() {
    let input = r#"{"one": 1, "two": 2, "three": 3}"#;
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::HashLiteral(HashLiteral { token: _, pairs }) = expression {
                    assert!(
                        pairs.len() == 3,
                        "hash.pairs has wrong length. got={}",
                        pairs.len()
                    );
                    let mut expected: HashMap<String, i64> = HashMap::new();
                    expected.insert(String::from("one"), 1);
                    expected.insert(String::from("two"), 2);
                    expected.insert(String::from("three"), 3);
                    for (key, value) in pairs.iter() {
                        if let Expression::StringLiteral(literal) = key {
                            let expected_value = expected.get(&literal.string());
                            test_integer_literal(value, *expected_value.unwrap());
                        } else {
                            panic!("key is not StringLiteral. got={:?}", key);
                        }
                    }
                } else {
                    panic!("exp is not HashLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_parsing_empty_hash_literal() {
    let input = "{}";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::HashLiteral(HashLiteral { token: _, pairs }) = expression {
                    assert!(
                        pairs.len() == 0,
                        "hash.pairs has wrong length. got={}",
                        pairs.len()
                    );
                } else {
                    panic!("exp is not HashLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

#[test]
fn test_parsing_hash_literal_with_expressions() {
    let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::HashLiteral(HashLiteral { token: _, pairs }) = expression {
                    assert!(
                        pairs.len() == 3,
                        "hash.pairs has wrong length. got={}",
                        pairs.len()
                    );

                    let mut tests: HashMap<String, fn(&Expression)> = HashMap::new();
                    tests.insert(String::from("one"), |e| {
                        test_infix_expression(
                            e,
                            &ExpectedType::from(0),
                            "+",
                            &ExpectedType::from(1),
                        )
                    });
                    tests.insert(String::from("two"), |e| {
                        test_infix_expression(
                            e,
                            &ExpectedType::from(10),
                            "-",
                            &ExpectedType::from(8),
                        )
                    });
                    tests.insert(String::from("three"), |e| {
                        test_infix_expression(
                            e,
                            &ExpectedType::from(15),
                            "/",
                            &ExpectedType::from(5),
                        )
                    });

                    for (key, value) in pairs {
                        if let Expression::StringLiteral(literal) = key {
                            if let Some(test_func) = tests.get(&literal.string()) {
                                test_func(value);
                            } else {
                                assert!(
                                    false,
                                    "No test function for key {:?} found",
                                    literal.string()
                                );
                            }
                        } else {
                            panic!("key is not StringLiteral. got={:?}", key);
                        }
                    }
                } else {
                    panic!("exp is not HashLiteral. got={:?}", expression);
                }
            } else {
                panic!("parse error");
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
