// src/parser.rs

use crate::ast::*;
use crate::lexer::*;
use crate::token::*;
use std::collections::HashMap;

type PrefixParseFn = fn(&mut Parser) -> Result<Expression, String>;
type InfixParseFn = fn(&mut Parser, Expression) -> Result<Expression, String>;

pub struct Parser {
    pub l: Lexer,
    pub cur_token: Token,
    pub peek_token: Token,
    pub prefix_parse_fns: HashMap<TokenType, PrefixParseFn>,
    pub infix_parse_fns: HashMap<TokenType, InfixParseFn>,
}
impl Parser {
    pub fn new(l: Lexer) -> Parser {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::ILLEGAL, 0),
            peek_token: new_token(TokenType::ILLEGAL, 0),
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };
        p.register_prefix(TokenType::IDENT, |parser| parser.parse_identifier());
        p.register_prefix(TokenType::INT, |parser| parser.parse_integer_literal());
        p.register_prefix(TokenType::BANG, |parser| parser.parse_prefix_expression());
        p.register_prefix(TokenType::MINUS, |parser| parser.parse_prefix_expression());
        p.register_prefix(TokenType::TRUE, |parser| parser.parse_boolean_literal());
        p.register_prefix(TokenType::FALSE, |parser| parser.parse_boolean_literal());
        p.register_prefix(TokenType::LPAREN, |parser| {
            parser.parse_grouped_expression()
        });
        p.register_prefix(TokenType::IF, |parser| parser.parse_if_expression());
        p.register_prefix(TokenType::FUNCTION, |parser| {
            parser.parse_function_literal()
        });
        p.register_prefix(TokenType::STRING, |parser| parser.parse_string_literal());
        p.register_prefix(TokenType::LBRACKET, |parser| parser.parse_array_literal());
        p.register_prefix(TokenType::LBRACE, |parser| parser.parse_hash_literal());

        p.register_infix(TokenType::PLUS, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::MINUS, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::SLASH, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::ASTERISK, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::EQ, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::NOTEQ, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::LT, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::GT, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::LPAREN, |parser, exp| {
            parser.parse_call_expression(exp)
        });
        p.register_infix(TokenType::LBRACKET, |parser, exp| {
            parser.parse_index_expression(exp)
        });

        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.cur_token = std::mem::replace(&mut self.peek_token, self.l.next_token());
    }

    pub fn parse_program(&mut self) -> Result<Program, Vec<String>> {
        let mut statements: Vec<Statement> = Vec::new();
        let mut errors = Vec::new();
        while self.cur_token.tk_type != TokenType::EOF {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => errors.push(err),
            }
            self.next_token();
        }
        if errors.len() != 0 {
            return Err(errors);
        }

        Ok(Program {
            statements: statements,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.cur_token.tk_type {
            TokenType::LET => self.parse_let_statement(),
            TokenType::RETURN => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();

        self.expect_peek(TokenType::IDENT)?;

        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        self.expect_peek(TokenType::ASSIGN)?;

        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST)?;

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Ok(Statement::LetStatement(LetStatement {
            token: token,
            name: name,
            value: value,
        }))
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        self.cur_token.tk_type == t
    }
    fn peek_token_is(&self, t: TokenType) -> bool {
        self.peek_token.tk_type == t
    }
    fn expect_peek(&mut self, t: TokenType) -> Result<(), String> {
        if self.peek_token_is(t.clone()) {
            self.next_token();
            Ok(())
        } else {
            Err(self.peek_error(t))
        }
    }

    fn peek_error(&mut self, t: TokenType) -> String {
        format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.tk_type
        )
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();
        self.next_token();

        let return_value = self.parse_expression(Precedence::LOWEST)?;

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Ok(Statement::ReturnStatement(ReturnStatement {
            token: token,
            return_value: return_value,
        }))
    }

    fn register_prefix(&mut self, token_type: TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, func);
    }
    fn register_infix(&mut self, token_type: TokenType, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, func);
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();
        let expression = self.parse_expression(Precedence::LOWEST)?;
        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }
        Ok(Statement::ExpressionStatement(ExpressionStatement {
            token: token,
            expression: expression,
        }))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, String> {
        if let Some(prefix) = self.prefix_parse_fns.get(&self.cur_token.tk_type) {
            let mut left_exp = prefix(self)?;
            while !self.peek_token_is(TokenType::SEMICOLON) && precedence < self.peek_precedence() {
                let infix_fn: InfixParseFn;
                if let Some(infix) = self.infix_parse_fns.get(&self.peek_token.tk_type) {
                    infix_fn = *infix;
                } else {
                    return Ok(left_exp);
                }
                self.next_token();
                left_exp = infix_fn(self, left_exp)?;
            }
            Ok(left_exp)
        } else {
            Err(self.no_prefix_parse_fn_error(&self.cur_token.tk_type))
        }
    }

    fn parse_identifier(&mut self) -> Result<Expression, String> {
        Ok(Expression::Identifier(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        if let Ok(value) = self.cur_token.literal.parse::<i64>() {
            Ok(Expression::IntegerLiteral(IntegerLiteral {
                token: token,
                value: value,
            }))
        } else {
            Err(format!(
                "could not parse {} as integer",
                self.cur_token.literal
            ))
        }
    }

    fn no_prefix_parse_fn_error(&self, t: &TokenType) -> String {
        format!("no prefix parse function for {:?} found", t)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX)?;

        Ok(Expression::PrefixExpression(PrefixExpression {
            token: token,
            operator: operator,
            right: Box::new(right),
        }))
    }

    fn peek_precedence(&self) -> Precedence {
        get_precedence(&self.peek_token.tk_type)
    }
    fn cur_precedence(&self) -> Precedence {
        get_precedence(&self.cur_token.tk_type)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Ok(Expression::InfixExpression(InfixExpression {
            token: token,
            left: Box::new(left),
            operator: operator,
            right: Box::new(right),
        }))
    }

    fn parse_boolean_literal(&self) -> Result<Expression, String> {
        Ok(Expression::BooleanLiteral(BooleanLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenType::TRUE),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression, String> {
        self.next_token();
        let exp = self.parse_expression(Precedence::LOWEST)?;
        self.expect_peek(TokenType::RPAREN)?;
        Ok(exp)
    }

    fn parse_if_expression(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        self.expect_peek(TokenType::LPAREN)?;
        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST)?;
        self.expect_peek(TokenType::RPAREN)?;
        self.expect_peek(TokenType::LBRACE)?;

        let consequence = self.parse_block_statement()?;

        let mut alternative: Option<BlockStatement> = None;

        if self.peek_token_is(TokenType::ELSE) {
            self.next_token();

            self.expect_peek(TokenType::LBRACE)?;
            alternative = Some(self.parse_block_statement()?);
        }

        Ok(Expression::IfExpression(IfExpression {
            token: token,
            condition: Box::new(condition),
            consequence: consequence,
            alternative: alternative,
        }))
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement, String> {
        let token = self.cur_token.clone();
        let mut statements: Vec<Statement> = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenType::RBRACE) {
            if self.cur_token_is(TokenType::EOF) {
                return Err(String::from("EOF"));
            }
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.next_token();
        }
        Ok(BlockStatement {
            token: token,
            statements: statements,
        })
    }

    fn parse_function_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        self.expect_peek(TokenType::LPAREN)?;

        let parameters = self.parse_function_parameters()?;

        self.expect_peek(TokenType::LBRACE)?;

        let body = self.parse_block_statement()?;

        Ok(Expression::FunctionLiteral(FunctionLiteral {
            token: token,
            parameters: parameters,
            body: body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Identifier>, String> {
        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return Ok(Vec::new());
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

        self.expect_peek(TokenType::RPAREN)?;
        Ok(identfiers)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let arguements = self.parse_expression_list(TokenType::RPAREN)?;

        Ok(Expression::CallExpression(CallExpression {
            token: token,
            function: Box::new(function),
            arguments: arguements,
        }))
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>, String> {
        let mut args: Vec<Expression> = Vec::new();
        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return Ok(args);
        }

        self.next_token();
        let arg = self.parse_expression(Precedence::LOWEST)?;
        args.push(arg);

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            let arg = self.parse_expression(Precedence::LOWEST)?;
            args.push(arg);
        }

        self.expect_peek(TokenType::RPAREN)?;

        Ok(args)
    }

    fn parse_string_literal(&self) -> Result<Expression, String> {
        Ok(Expression::StringLiteral(StringLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let elements = self.parse_expression_list(TokenType::RBRACKET)?;

        Ok(Expression::ArrayLiteral(ArrayLiteral {
            token: token,
            elements: elements.to_vec(),
        }))
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Result<Vec<Expression>, String> {
        let mut list: Vec<Expression> = Vec::new();

        if self.peek_token_is(end.clone()) {
            self.next_token();
            return Ok(list);
        }

        self.next_token();
        let mut expr = self.parse_expression(Precedence::LOWEST)?;
        list.push(expr);

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            expr = self.parse_expression(Precedence::LOWEST)?;
            list.push(expr);
        }

        self.expect_peek(end)?;
        Ok(list)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();

        self.next_token();
        let index = self.parse_expression(Precedence::LOWEST)?;

        self.expect_peek(TokenType::RBRACKET)?;

        Ok(Expression::IndexExpression(IndexExpression {
            token: token,
            left: Box::new(left),
            index: Box::new(index),
        }))
    }

    fn parse_hash_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let mut pairs: HashMap<Expression, Expression> = HashMap::new();
        while !self.peek_token_is(TokenType::RBRACE) {
            self.next_token();
            let key = self.parse_expression(Precedence::LOWEST)?;
            self.expect_peek(TokenType::COLON)?;

            self.next_token();
            let value = self.parse_expression(Precedence::LOWEST)?;

            pairs.insert(key, value);

            if !self.peek_token_is(TokenType::RBRACE) {
                self.expect_peek(TokenType::COMMA)?;
            }
        }

        self.expect_peek(TokenType::RBRACE)?;

        Ok(Expression::HashLiteral(HashLiteral {
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
