// src/ast/ast_test.rs

use crate::ast::*;
use crate::token::*;

#[test]
fn test_string() {
    let program = Program {
        statements: vec![Statement::LetStatement(LetStatement {
            token: Token {
                r#type: TokenType::LET,
                literal: String::from("let"),
            },
            name: Identifier {
                token: Token {
                    r#type: TokenType::IDENT,
                    literal: String::from("myVar"),
                },
                value: String::from("myVar"),
            },
            value: Expression::Identifier(Identifier {
                token: Token {
                    r#type: TokenType::IDENT,
                    literal: String::from("anotherVar"),
                },
                value: String::from("anotherVar"),
            }),
        })],
    };

    assert!(
        program.string() == "let myVar = anotherVar;",
        "program.string() wrong. got={}",
        program.string()
    );
}
