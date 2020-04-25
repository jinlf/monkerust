// src/ast/ast.rs

use crate::token::*;
use std::collections::*;
use std::hash::Hash as StdHash;
use std::hash::Hasher;

pub trait NodeTrait {
    fn string(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Node {
    Program(Program),
    Statement(Statement),
    Expression(Expression),
}
impl NodeTrait for Node {
    fn string(&self) -> String {
        match self {
            Node::Program(p) => p.string(),
            Node::Statement(s) => s.string(),
            Node::Expression(e) => e.string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
    ExpressionStatement(ExpressionStatement),
    BlockStatement(BlockStatement),
}
impl NodeTrait for Statement {
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

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}
impl NodeTrait for Program {
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
    fn string(&self) -> String {
        format!(
            "{} {} = {};",
            self.token.literal,
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
    fn string(&self) -> String {
        format!("{} {};", self.token.literal, self.return_value.string())
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Expression,
}
impl NodeTrait for ExpressionStatement {
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
    fn string(&self) -> String {
        format!(
            "{} ({}) {}",
            self.token.literal,
            self.parameters
                .iter()
                .map(|x| { x.string() })
                .collect::<Vec<String>>()
                .join(", "),
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
    fn string(&self) -> String {
        format!(
            "{}({})",
            self.function.string(),
            self.arguments
                .iter()
                .map(|x| x.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}
impl NodeTrait for StringLiteral {
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
        self.string() == other.string()
    }
}
impl StdHash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.string().hash(state);
        state.finish();
    }
}
