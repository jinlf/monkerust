// src/parser_test.rs

extern crate test;

use super::ast::*;
use super::lexer::*;
use super::parser::*;
use std::collections::*;
use test::{black_box, Bencher};

#[test]
fn test_let_statements() {
    let input = "
let x = 5;
let y = 10;
let foobar = 838383;
";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 3,
            "program.statements does not contain 3 statements. got={}",
            statements.len()
        );

        let tests = ["x", "y", "foobar"];
        for (i, tt) in tests.iter().enumerate() {
            test_let_statement(&statements[i], tt);
        }
    } else {
        assert!(false, "parse_program() returned None");
    }
}

fn test_let_statement(s: &Statement, expected_name: &str) {
    assert!(
        s.token_literal() == "let",
        "s.token_literal not 'let'. got={}",
        s.token_literal()
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
            name.token_literal() == expected_name,
            "s.name not '{}'. got={}",
            expected_name,
            name.token_literal()
        );
    } else {
        assert!(false, "s not LetStatement. got={:?}", s);
    }
}

fn check_parser_errors(p: &mut Parser) {
    if p.errors.len() == 0 {
        return;
    }

    let mut msgs = String::from(format!("parser has {} errors\n", p.errors.len()));
    for msg in p.errors.iter() {
        msgs.push_str(&format!("parser error: {:?}\n", msg));
    }
    assert!(false, msgs);
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
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 3,
            "program.statements does not contain 3 statements. got={}",
            statements.len()
        );

        for stmt in statements.iter() {
            if let Statement::ReturnStatement(_) = stmt {
                assert!(
                    stmt.token_literal() == "return",
                    "returnStmt.token_literal not 'return', got={}",
                    stmt.token_literal()
                );
            } else {
                assert!(false, "stmt not ReturnStatement. got={:?}", stmt);
            }
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_identifier_expression() {
    let input = "foobar;";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                assert!(false, "exp not Identifier. got={:?}", expression);
            }
        } else {
            assert!(
                false,
                "program.statements[0] is not ExpressionStatement. got={:?}",
                &statements[0]
            );
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_integer_literal_expression() {
    let input = "5;";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                assert!(false, "exp not IntegerLiteral. got={:?}", expression);
            }
        } else {
            assert!(
                false,
                "program.statements[0] is not ExpressionStatement. got={:?}",
                &statements[0]
            );
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_parsing_prefix_expression() {
    let tests: [(&str, &str, Box<dyn std::any::Any>); 4] = [
        ("!5;", "!", Box::new(5 as i64)),
        ("-15;", "-", Box::new(15 as i64)),
        ("!true", "!", Box::new(true)),
        ("!false", "!", Box::new(false)),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);
        if let Some(Program { statements }) = program {
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

                    test_literal_expression(right, &*tt.2);
                } else {
                    assert!(false, "stmt is not PrefixExpression. got={:?}", expression);
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        } else {
            assert!(false, "parse error");
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
        assert!(false, "il not IntegerLiteral. got={:?}", il);
    }
}

#[test]
fn test_parsing_infix_expressions() {
    let tests: [(&str, Box<dyn std::any::Any>, &str, Box<dyn std::any::Any>); 11] = [
        ("5 + 5;", Box::new(5 as i64), "+", Box::new(5 as i64)),
        ("5 - 5;", Box::new(5 as i64), "-", Box::new(5 as i64)),
        ("5 * 5;", Box::new(5 as i64), "*", Box::new(5 as i64)),
        ("5 / 5;", Box::new(5 as i64), "/", Box::new(5 as i64)),
        ("5 > 5;", Box::new(5 as i64), ">", Box::new(5 as i64)),
        ("5 < 5;", Box::new(5 as i64), "<", Box::new(5 as i64)),
        ("5 == 5;", Box::new(5 as i64), "==", Box::new(5 as i64)),
        ("5 != 5;", Box::new(5 as i64), "!=", Box::new(5 as i64)),
        ("true == true", Box::new(true), "==", Box::new(true)),
        ("true != false", Box::new(true), "!=", Box::new(false)),
        ("false == false", Box::new(false), "==", Box::new(false)),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Program { statements }) = program {
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
                    test_literal_expression(left, &*tt.1);

                    assert!(
                        operator == tt.2,
                        "exp.operator is not '{}. got={}",
                        tt.2,
                        operator
                    );

                    test_literal_expression(right, &*tt.3);
                } else {
                    assert!(false, "exp is not InfixExpression. got={:?}", expression);
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        } else {
            assert!(false, "parse error");
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
        let program = p.parse_program();
        check_parser_errors(&mut p);

        let actual = program.unwrap().string();
        assert!(actual == tt.1, "expected={:?}, got={:?}", tt.1, actual);
    }
}

fn test_identifier(exp: &Expression, expected_value: String) {
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
        assert!(false, "exp not Identifier. got={:?}", exp);
    }
}

fn test_literal_expression(exp: &Expression, expected: &dyn std::any::Any) {
    if let Some(v) = expected.downcast_ref::<i64>() {
        test_integer_literal(exp, *v);
    } else if let Some(v) = expected.downcast_ref::<&str>() {
        test_identifier(exp, String::from(*v));
    } else if let Some(v) = expected.downcast_ref::<bool>() {
        test_boolean_literal(exp, *v);
    } else {
        assert!(false, "type of exp not handled. got={:?}", exp);
    }
}

fn test_infix_expression(
    exp: &Expression,
    expected_left: &dyn std::any::Any,
    expected_operator: String,
    expected_right: &dyn std::any::Any,
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
        assert!(false, "exp is not InfixExpression. got={:?}", exp);
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
        assert!(false, "exp not BooleanLiteral. got={:?}", exp);
    }
}

#[test]
fn test_if_expression() {
    let input = "if (x < y) { x }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                    &*Box::new("x"),
                    String::from("<"),
                    &*Box::new("y"),
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
                    test_identifier(expression, String::from("x"));

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
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_if_else_expression() {
    let input = "if (x < y) { x } else { y }";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                    &*Box::new("x"),
                    String::from("<"),
                    &*Box::new("y"),
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
                    test_identifier(expression, String::from("x"));

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
                            test_identifier(expression, String::from("y"));
                        } else {
                            assert!(
                                false,
                                "statements[0] is not ExpressionStatement. got={:?}",
                                &a.statements[0]
                            );
                        }
                    } else {
                        assert!(false, "exp alternative.statements was None");
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
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_function_literal_parsing() {
    let input = "fn(x, y) { x + y; }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { mut statements }) = program {
        assert!(
            statements.len() == 1,
            "program.body does not contain {} statements. got={}",
            1,
            statements.len()
        );

        if let Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        }) = statements.remove(0)
        {
            if let Expression::FunctionLiteral(FunctionLiteral {
                token: _,
                mut parameters,
                body,
            }) = expression
            {
                assert!(
                    parameters.len() == 2,
                    "function literal parameters wrong. want 2, got={}",
                    parameters.len()
                );

                test_literal_expression(
                    &Expression::Identifier(parameters.remove(0)),
                    &*Box::new("x"),
                );
                test_literal_expression(
                    &Expression::Identifier(parameters.remove(0)),
                    &*Box::new("y"),
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
                        &*Box::new("x"),
                        String::from("+"),
                        &*Box::new("y"),
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
    } else {
        assert!(false, "parse error");
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
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Program { statements }) = program {
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
                            ident,
                        );
                    }
                } else {
                    assert!(false, "parse error");
                }
            } else {
                assert!(false, "parse error");
            }
        } else {
            assert!(false, "parse error");
        }
    }
}

#[test]
fn test_call_expression_parsing() {
    let input = "add(1, 2 * 3, 4 + 5);";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                test_identifier(function, String::from("add"));

                assert!(
                    arguments.len() == 3,
                    "wrong length of arguments. got={}",
                    arguments.len()
                );

                test_literal_expression(&arguments[0], &*Box::new(1 as i64));
                test_infix_expression(
                    &arguments[1],
                    &*Box::new(2 as i64),
                    String::from("*"),
                    &*Box::new(3 as i64),
                );
                test_infix_expression(
                    &arguments[2],
                    &*Box::new(4 as i64),
                    String::from("+"),
                    &*Box::new(5 as i64),
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
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_string_literal_expression() {
    let input = r#""hello world";"#;
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                assert!(false, "exp not StringLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_parsing_array_literals() {
    let input = "[1, 2 * 2, 3 + 3]";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                    &*Box::new(2 as i64),
                    String::from("*"),
                    &*Box::new(2 as i64),
                );
                test_infix_expression(
                    &elements[2],
                    &*Box::new(3 as i64),
                    String::from("+"),
                    &*Box::new(3 as i64),
                );
            } else {
                assert!(false, "exp not ArrayLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_parsing_index_expressions() {
    let input = "myArray[1 + 1]";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                test_identifier(left, String::from("myArray"));
                test_infix_expression(
                    index,
                    &*Box::new(1 as i64),
                    String::from("+"),
                    &*Box::new(1 as i64),
                );
            } else {
                assert!(false, "exp not IndexExpression. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_parsing_hash_literals_string_keys() {
    let input = r#"{"one": 1, "two": 2, "three": 3}"#;
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
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
                        assert!(false, "key is not StringLiteral. got={:?}", key);
                    }
                }
            } else {
                assert!(false, "exp is not HashLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_parsing_empty_hash_literal() {
    let input = "{}";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);
    if let Some(Program { statements }) = program {
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
                assert!(false, "exp is not HashLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}

#[test]
fn test_parsing_hash_literal_with_expressions() {
    let input = r#"{"one": 0 + 1, "two": 10 - 8, "three": 15 / 5}"#;
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);
    if let Some(Program { statements }) = program {
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
                        &*Box::new(0 as i64),
                        String::from("+"),
                        &*Box::new(1 as i64),
                    )
                });
                tests.insert(String::from("two"), |e| {
                    test_infix_expression(
                        e,
                        &*Box::new(10 as i64),
                        String::from("-"),
                        &*Box::new(8 as i64),
                    )
                });
                tests.insert(String::from("three"), |e| {
                    test_infix_expression(
                        e,
                        &*Box::new(15 as i64),
                        String::from("/"),
                        &*Box::new(5 as i64),
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
                        assert!(false, "key is not StringLiteral. got={:?}", key);
                    }
                }
            } else {
                assert!(false, "exp is not HashLiteral. got={:?}", expression);
            }
        } else {
            assert!(false, "parse error");
        }
    } else {
        assert!(false, "parse error");
    }
}
