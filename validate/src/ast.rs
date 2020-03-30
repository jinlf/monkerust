// src/ast.rs

use super::token::*;
use std::collections::*;
use std::hash::Hash as StdHash;
use std::hash::Hasher;

pub trait NodeTrait {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}

pub enum Node {
    Program(Program),
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
    ExpressionStatement(ExpressionStatement),
    BlockStatement(BlockStatement),
}
impl NodeTrait for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.token_literal(),
            Statement::ReturnStatement(return_stmt) => return_stmt.token_literal(),
            Statement::ExpressionStatement(expr_stmt) => expr_stmt.token_literal(),
            Statement::BlockStatement(block_stmt) => block_stmt.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.string(),
            Statement::ReturnStatement(return_stmt) => return_stmt.string(),
            Statement::ExpressionStatement(expr_stmt) => expr_stmt.string(),
            Statement::BlockStatement(block_stmt) => block_stmt.string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    PrefixExpression(PrefixExpression),
    InfixExpression(InfixExpression),
    BooleanLiteral(BooleanLiteral),
    IfExpression(IfExpression),
    FunctionLiteral(FunctionLiteral),
    CallExpression(CallExpression),
    StringLiteral(StringLiteral),
    ArrayLiteral(ArrayLiteral),
    IndexExpression(IndexExpression),
    HashLiteral(HashLiteral),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.token_literal(),
            Expression::IntegerLiteral(integer_literal) => integer_literal.token_literal(),
            Expression::PrefixExpression(prefix_expr) => prefix_expr.token_literal(),
            Expression::InfixExpression(infix_expr) => infix_expr.token_literal(),
            Expression::BooleanLiteral(bo) => bo.token_literal(),
            Expression::IfExpression(if_expr) => if_expr.token_literal(),
            Expression::FunctionLiteral(function_literal) => function_literal.token_literal(),
            Expression::CallExpression(call_expr) => call_expr.token_literal(),
            Expression::StringLiteral(string_literal) => string_literal.token_literal(),
            Expression::ArrayLiteral(array_literal) => array_literal.token_literal(),
            Expression::IndexExpression(index_expr) => index_expr.token_literal(),
            Expression::HashLiteral(hash_literal) => hash_literal.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.string(),
            Expression::IntegerLiteral(integer_literal) => integer_literal.string(),
            Expression::PrefixExpression(prefix_expr) => prefix_expr.string(),
            Expression::InfixExpression(infix_expr) => infix_expr.string(),
            Expression::BooleanLiteral(bo) => bo.string(),
            Expression::IfExpression(if_expr) => if_expr.string(),
            Expression::FunctionLiteral(function_literal) => function_literal.string(),
            Expression::CallExpression(call_expr) => call_expr.string(),
            Expression::StringLiteral(string_literal) => string_literal.string(),
            Expression::ArrayLiteral(array_literal) => array_literal.string(),
            Expression::IndexExpression(index_expr) => index_expr.string(),
            Expression::HashLiteral(hash_literal) => hash_literal.string(),
        }
    }
}

pub struct Program {
    pub statements: Vec<Statement>,
}
impl NodeTrait for Program {
    fn token_literal(&self) -> String {
        if self.statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            String::new()
        }
    }
    fn string(&self) -> String {
        let mut out = String::new();
        for s in self.statements.iter() {
            out.push_str(&s.string());
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}
impl NodeTrait for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!(
            "{} {} = {};",
            self.token_literal(),
            self.name.string(),
            self.value.string(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}
impl NodeTrait for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Expression,
}
impl NodeTrait for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("{} {};", self.token_literal(), self.return_value.string())
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Expression,
}
impl NodeTrait for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        self.expression.string()
    }
}

#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}
impl NodeTrait for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug, Clone)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Box<Expression>,
}
impl NodeTrait for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("({}{})", self.operator, self.right.string())
    }
}

#[derive(Debug, Clone)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub operator: String,
    pub right: Box<Expression>,
}
impl NodeTrait for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.string(),
            self.operator,
            self.right.string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct BooleanLiteral {
    pub token: Token,
    pub value: bool,
}
impl NodeTrait for BooleanLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}
impl NodeTrait for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "if{} {}",
            self.condition.string(),
            self.consequence.string()
        ));
        if let Some(a) = &self.alternative {
            out.push_str(&format!("else {}", a.string()));
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}
impl NodeTrait for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut out = String::new();

        for s in self.statements.iter() {
            out.push_str(&s.string());
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}
impl NodeTrait for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut params: Vec<String> = Vec::new();
        for p in self.parameters.iter() {
            params.push(p.string());
        }
        format!(
            "{} ({}) {}",
            self.token_literal(),
            params.join(", "),
            self.body.string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}
impl NodeTrait for CallExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut args: Vec<String> = Vec::new();
        for a in self.arguments.iter() {
            args.push(a.string());
        }

        format!("{}({})", self.function.string(), args.join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}
impl NodeTrait for StringLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("{}", self.value)
    }
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<Expression>,
}
impl NodeTrait for ArrayLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!(
            "[{}]",
            self.elements
                .iter()
                .map(|x| x.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub index: Box<Expression>,
}
impl NodeTrait for IndexExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("({}[{}])", self.left.string(), self.index.string())
    }
}

#[derive(Clone)]
pub struct HashLiteral {
    pub token: Token,
    pub pairs: HashMap<Expression, Expression>,
}
impl NodeTrait for HashLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!(
            "{{{}}}",
            self.pairs
                .iter()
                .map(|(k, v)| format!("{}:{}", k.string(), v.string()))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
impl std::fmt::Debug for HashLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string())
    }
}

impl Eq for Expression {}
impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        let addr = self as *const Expression as usize;
        let other_addr = other as *const Expression as usize;
        addr == other_addr
    }
}
impl StdHash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let addr = self as *const Expression as usize;
        state.write_usize(addr);
        state.finish();
    }
}
