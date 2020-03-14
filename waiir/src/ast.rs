use super::lexer::*;
use super::token::*;

pub trait NodeTrait {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}
pub enum Node {
    Program { stmts: Vec<Stmt> },
    Stmt(Stmt),
    Expr(Expr),
}
impl NodeTrait for Node {
    fn token_literal(&self) -> String {
        match self {
            Node::Program { stmts } => {
                if stmts.len() > 0 {
                    stmts[0].token_literal()
                } else {
                    String::new()
                }
            }
            Node::Stmt(stmt) => stmt.token_literal(),
            Node::Expr(expr) => expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Node::Program { stmts } => {
                let mut out = String::new();
                for s in stmts.iter() {
                    out.push_str(&s.string());
                }
                out
            }
            Node::Stmt(stmt) => stmt.string(),
            Node::Expr(expr) => expr.string(),
        }
    }
}

pub trait StmtTrait {
    fn stmt_node(&self);
}

#[derive(Debug, Clone)]
pub enum Stmt {
    LetStmt {
        token: Token,
        name: Ident,
        value: Option<Expr>,
    },
    ReturnStmt {
        token: Token,
        return_value: Option<Expr>,
    },
    ExprStmt {
        token: Token,
        expr: Option<Expr>,
    },
}
impl StmtTrait for Stmt {
    fn stmt_node(&self) {}
}
impl Stmt {
    pub fn token_literal(&self) -> String {
        match self {
            Stmt::LetStmt { token, name, value } => token.literal.clone(),
            Stmt::ReturnStmt {
                token,
                return_value,
            } => token.literal.clone(),
            Stmt::ExprStmt { token, expr } => token.literal.clone(),
        }
    }
    pub fn string(&self) -> String {
        match self {
            Stmt::LetStmt { token, name, value } => {
                let mut out = String::new();

                out.push_str(&token.literal);
                out.push_str(" ");
                out.push_str(&name.string());
                out.push_str(" = ");
                if value.is_some() {
                    out.push_str(&value.as_ref().unwrap().string());
                }
                out.push_str(";");
                out
            }
            Stmt::ReturnStmt {
                token,
                return_value,
            } => {
                let mut out = String::new();

                out.push_str(&token.literal);
                out.push_str(" ");
                if return_value.is_some() {
                    out.push_str(&return_value.as_ref().unwrap().string());
                }
                out.push_str(";");
                out
            }
            Stmt::ExprStmt { token, expr } => match expr {
                Some(e) => e.string(),
                _ => String::new(),
            },
        }
    }
}

pub trait ExprTrait {
    fn expr_node(&self);
}

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(Ident),
    IntegerLiteral(IntegerLiteral),
    PrefixExpr(PrefixExpr),
}
impl ExprTrait for Expr {
    fn expr_node(&self) {}
}
impl Expr {
    pub fn token_literal(&self) -> String {
        match self {
            Expr::Ident(ident) => ident.token_literal(),
            Expr::IntegerLiteral(integer_literal) => integer_literal.token_literal(),
            Expr::PrefixExpr(prefix_expr) => prefix_expr.token_literal(),
        }
    }
    pub fn string(&self) -> String {
        match self {
            Expr::Ident(ident) => ident.string(),
            Expr::IntegerLiteral(integer_literal) => integer_literal.string(),
            Expr::PrefixExpr(prefix_expr) => prefix_expr.string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub token: Token,
    pub value: String,
}
impl Ident {
    pub fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    pub fn string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}
impl IntegerLiteral {
    pub fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    pub fn string(&self) -> String {
        format!("{}", self.value)
    }
}

#[derive(Debug, Clone)]
pub struct PrefixExpr {
    pub token: Token,
    pub operator: String,
    pub right: Box<Expr>,
}
impl PrefixExpr {
    pub fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    pub fn string(&self) -> String {
        let mut out = String::new();
        out.push_str("(");
        out.push_str(&self.operator);
        out.push_str(&self.right.string());
        out.push_str(")");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let program = Node::Program {
            stmts: vec![Stmt::LetStmt {
                token: Token {
                    tk_type: TokenType::LET,
                    literal: String::from("let"),
                },
                name: Ident {
                    token: Token {
                        tk_type: TokenType::IDENT,
                        literal: String::from("myVar"),
                    },
                    value: String::from("myVar"),
                },
                value: Some(Expr::Ident(Ident {
                    token: Token {
                        tk_type: TokenType::IDENT,
                        literal: String::from("anotherVar"),
                    },
                    value: String::from("anotherVar"),
                })),
            }],
        };

        assert!(
            program.string() == "let myVar = anotherVar;",
            "program.string() wrong. got={}",
            program.string()
        );
    }
}
