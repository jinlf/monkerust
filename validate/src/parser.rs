// src/parser.rs

use super::ast::*;
use super::lexer::*;
use super::token::*;
use std::collections::*;

pub struct Parser<'a> {
    pub l: Lexer<'a>,
    pub cur_token: Token,
    pub peek_token: Token,
    pub errors: Vec<String>,
}
impl<'a> Parser<'a> {
    pub fn new(l: Lexer<'a>) -> Parser<'a> {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::ILLEGAL, 0),
            peek_token: new_token(TokenType::ILLEGAL, 0),
            errors: Vec::new(),
        };
        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    pub fn parse_program(&mut self) -> Option<Program> {
        let mut statements: Vec<Statement> = Vec::new();

        while self.cur_token.tk_type != TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Some(Program {
            statements: statements,
        })
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.tk_type {
            TokenType::LET => {
                if let Some(stmt) = self.parse_let_statement() {
                    return Some(Statement::LetStatement(stmt));
                }
                None
            }
            TokenType::RETURN => {
                if let Some(stmt) = self.parse_return_statement() {
                    return Some(Statement::ReturnStatement(stmt));
                }
                None
            }
            _ => {
                if let Some(stmt) = self.parse_expression_statement() {
                    return Some(Statement::ExpressionStatement(stmt));
                }
                None
            }
        }
    }

    fn parse_let_statement(&mut self) -> Option<LetStatement> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::IDENT) {
            return None;
        }

        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::ASSIGN) {
            return None;
        }

        self.next_token();
        let value = self.parse_expression(Precedence::LOWEST);
        if value.is_none() {
            return None;
        }

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(LetStatement {
            token: token,
            name: name,
            value: value.unwrap(),
        })
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        self.cur_token.tk_type == t
    }
    fn peek_token_is(&self, t: TokenType) -> bool {
        self.peek_token.tk_type == t
    }
    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(t.clone()) {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.tk_type
        );
        self.errors.push(msg);
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        let token = self.cur_token.clone();
        self.next_token();

        let return_value = self.parse_expression(Precedence::LOWEST);
        if return_value.is_none() {
            return None;
        }

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(ReturnStatement {
            token: token,
            return_value: return_value.unwrap(),
        })
    }

    fn parse_expression_statement(&mut self) -> Option<ExpressionStatement> {
        let token = self.cur_token.clone();
        let expression = self.parse_expression(Precedence::LOWEST);
        if expression.is_none() {
            return None;
        }
        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }
        Some(ExpressionStatement {
            token: token,
            expression: expression.unwrap(),
        })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
            TokenType::IDENT => left_exp = self.parse_identifier(),
            TokenType::INT => left_exp = self.parse_integer_literal(),
            TokenType::BANG | TokenType::MINUS => left_exp = self.parse_prefix_expression(),
            TokenType::TRUE | TokenType::FALSE => left_exp = self.parse_boolean_literal(),
            TokenType::LPAREN => left_exp = self.parse_grouped_expression(),
            TokenType::IF => left_exp = self.parse_if_expression(),
            TokenType::FUNCTION => left_exp = self.parse_function_literal(),
            TokenType::STRING => left_exp = self.parse_string_literal(),
            TokenType::LBRACKET => left_exp = self.parse_array_literal(),
            TokenType::LBRACE => left_exp = self.parse_hash_literal(),
            _ => {
                self.no_prefix_parse_fn_error(tk_type);
                return None;
            }
        }
        if left_exp.is_none() {
            return None;
        }

        while !self.peek_token_is(TokenType::SEMICOLON) && precedence < self.peek_precedence() {
            let tk_type = self.peek_token.tk_type.clone();
            match tk_type {
                TokenType::PLUS
                | TokenType::MINUS
                | TokenType::SLASH
                | TokenType::ASTERISK
                | TokenType::EQ
                | TokenType::NOTEQ
                | TokenType::LT
                | TokenType::GT => {
                    self.next_token();
                    left_exp = self.parse_infix_expression(left_exp.unwrap());
                }
                TokenType::LPAREN => {
                    self.next_token();
                    left_exp = self.parse_call_expression(left_exp.unwrap())
                }
                TokenType::LBRACKET => {
                    self.next_token();
                    left_exp = self.parse_index_expression(left_exp.unwrap())
                }
                _ => return left_exp,
            }
        }
        left_exp
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        Some(Expression::Identifier(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&mut self) -> Option<Expression> {
        if let Ok(value) = self.cur_token.literal.parse::<i64>() {
            Some(Expression::IntegerLiteral(IntegerLiteral {
                token: self.cur_token.clone(),
                value: value,
            }))
        } else {
            self.errors.push(format!(
                "could not parse {} as integer",
                self.cur_token.literal
            ));
            None
        }
    }

    fn no_prefix_parse_fn_error(&mut self, t: TokenType) {
        self.errors
            .push(format!("no prefix parse function for {:?} found", t));
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX);
        if right.is_none() {
            return None;
        }

        Some(Expression::PrefixExpression(PrefixExpression {
            token: token,
            operator: operator,
            right: Box::new(right.unwrap()),
        }))
    }

    fn peek_precedence(&self) -> Precedence {
        get_precedence(&self.peek_token.tk_type)
    }
    fn cur_precedence(&self) -> Precedence {
        get_precedence(&self.cur_token.tk_type)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence);
        if right.is_none() {
            return None;
        }
        Some(Expression::InfixExpression(InfixExpression {
            token: token,
            left: Box::new(left),
            operator: operator,
            right: Box::new(right.unwrap()),
        }))
    }

    fn parse_boolean_literal(&self) -> Option<Expression> {
        Some(Expression::BooleanLiteral(BooleanLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenType::TRUE),
        }))
    }
    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let exp = self.parse_expression(Precedence::LOWEST);
        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        exp
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST);
        if condition.is_none() {
            return None;
        }
        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let consequence = self.parse_block_statement();
        if consequence.is_none() {
            return None;
        }

        let mut alternative: Option<BlockStatement> = None;

        if self.peek_token_is(TokenType::ELSE) {
            self.next_token();

            if !self.expect_peek(TokenType::LBRACE) {
                return None;
            }
            alternative = self.parse_block_statement();
            if alternative.is_none() {
                return None;
            }
        }

        Some(Expression::IfExpression(IfExpression {
            token: token,
            condition: Box::new(condition.unwrap()),
            consequence: consequence.unwrap(),
            alternative: alternative,
        }))
    }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        let token = self.cur_token.clone();
        let mut statements: Vec<Statement> = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenType::RBRACE) {
            if self.cur_token_is(TokenType::EOF) {
                return None;
            }
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                return None;
            }
            self.next_token();
        }
        Some(BlockStatement {
            token: token,
            statements: statements,
        })
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }

        let parameters = self.parse_function_parameters();
        if parameters.is_none() {
            return None;
        }

        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let body = self.parse_block_statement();
        if body.is_none() {
            return None;
        }

        Some(Expression::FunctionLiteral(FunctionLiteral {
            token: token,
            parameters: parameters.unwrap(),
            body: body.unwrap(),
        }))
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Identifier>> {
        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return Some(Vec::new());
        }

        self.next_token();

        let mut identfiers: Vec<Identifier> = Vec::new();
        identfiers.push(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        });

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();

            identfiers.push(Identifier {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            })
        }

        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        Some(identfiers)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let arguements = self.parse_expression_list(TokenType::RPAREN);
        if arguements.is_none() {
            return None;
        }

        Some(Expression::CallExpression(CallExpression {
            token: token,
            function: Box::new(function),
            arguments: arguements.unwrap(),
        }))
    }

    // fn parse_call_arguments(&mut self) -> Option<Vec<Expression>> {
    //     let mut args: Vec<Expression> = Vec::new();
    //     if self.peek_token_is(TokenType::RPAREN) {
    //         self.next_token();
    //         return Some(args);
    //     }

    //     self.next_token();
    //     let arg = self.parse_expression(Precedence::LOWEST);
    //     if arg.is_none() {
    //         return None;
    //     }
    //     args.push(arg.unwrap());

    //     while self.peek_token_is(TokenType::COMMA) {
    //         self.next_token();
    //         self.next_token();
    //         let arg = self.parse_expression(Precedence::LOWEST);
    //         if arg.is_none() {
    //             return None;
    //         }
    //         args.push(arg.unwrap());
    //     }

    //     if !self.expect_peek(TokenType::RPAREN) {
    //         return None;
    //     }

    //     Some(args)
    // }

    fn parse_string_literal(&self) -> Option<Expression> {
        Some(Expression::StringLiteral(StringLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let elements = self.parse_expression_list(TokenType::RBRACKET);
        if elements.is_none() {
            return None;
        }

        Some(Expression::ArrayLiteral(ArrayLiteral {
            token: token,
            elements: elements.unwrap(),
        }))
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Option<Vec<Expression>> {
        let mut list: Vec<Expression> = Vec::new();

        if self.peek_token_is(end.clone()) {
            self.next_token();
            return Some(list);
        }

        self.next_token();
        let mut expr = self.parse_expression(Precedence::LOWEST);
        if expr.is_none() {
            return None;
        }
        list.push(expr.unwrap());

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            expr = self.parse_expression(Precedence::LOWEST);
            if expr.is_none() {
                return None;
            }
            list.push(expr.unwrap());
        }

        if !self.expect_peek(end) {
            return None;
        }
        Some(list)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();

        self.next_token();
        let index = self.parse_expression(Precedence::LOWEST);
        if index.is_none() {
            return None;
        }

        if !self.expect_peek(TokenType::RBRACKET) {
            return None;
        }

        Some(Expression::IndexExpression(IndexExpression {
            token: token,
            left: Box::new(left),
            index: Box::new(index.unwrap()),
        }))
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let mut pairs: HashMap<Expression, Expression> = HashMap::new();
        while !self.peek_token_is(TokenType::RBRACE) {
            self.next_token();
            let key = self.parse_expression(Precedence::LOWEST);
            if key.is_none() {
                return None;
            }

            if !self.expect_peek(TokenType::COLON) {
                return None;
            }

            self.next_token();
            let value = self.parse_expression(Precedence::LOWEST);
            if value.is_none() {
                return None;
            }

            pairs.insert(key.unwrap(), value.unwrap());

            if !self.peek_token_is(TokenType::RBRACE) && !self.expect_peek(TokenType::COMMA) {
                return None;
            }
        }

        if !self.expect_peek(TokenType::RBRACE) {
            return None;
        }

        Some(Expression::HashLiteral(HashLiteral {
            token: token,
            pairs: pairs,
        }))
    }
}

#[derive(PartialOrd, PartialEq)]
pub enum Precedence {
    LOWEST,
    EQUALS,      // ==
    LESSGREATER, // > or <
    SUM,         // +
    PRODUCT,     // *
    PREFIX,      // -x or !x
    CALL,        // myFunction(X)
    INDEX,       // array[index]
}

fn get_precedence(t: &TokenType) -> Precedence {
    match t {
        TokenType::EQ | TokenType::NOTEQ => Precedence::EQUALS,
        TokenType::LT | TokenType::GT => Precedence::LESSGREATER,
        TokenType::PLUS | TokenType::MINUS => Precedence::SUM,
        TokenType::SLASH | TokenType::ASTERISK => Precedence::PRODUCT,
        TokenType::LPAREN => Precedence::CALL,
        TokenType::LBRACKET => Precedence::INDEX,
        _ => Precedence::LOWEST,
    }
}
