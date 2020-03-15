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
        value: Expr,
    },
    ReturnStmt {
        token: Token,
        return_value: Expr,
    },
    ExprStmt {
        token: Token,
        expr: Expr,
    },
    BlockStmt(BlockStmt),
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
            Stmt::BlockStmt(block_stmt) => block_stmt.token_literal(),
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
                out.push_str(&value.string());
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
                out.push_str(&return_value.string());
                out.push_str(";");
                out
            }
            Stmt::ExprStmt { token, expr } => expr.string(),
            Stmt::BlockStmt(block_stmt) => block_stmt.string(),
        }
    }
}

pub trait ExprTrait {
    fn expr_node(&self);
}

#[derive(Debug, Clone)]
pub enum Expr {
    MockExpr {},
    Ident(Ident),
    IntegerLiteral(IntegerLiteral),
    PrefixExpr(PrefixExpr),
    InfixExpr(InfixExpr),
    Boolean(Boolean),
    IfExpr {
        token: Token,
        condition: Box<Expr>,
        consequence: BlockStmt,
        alternative: Option<BlockStmt>,
    },
    FuncLite {
        token: Token,
        parameters: Vec<Ident>,
        body: BlockStmt,
    },
}
impl ExprTrait for Expr {
    fn expr_node(&self) {}
}
impl Expr {
    pub fn token_literal(&self) -> String {
        match self {
            Expr::MockExpr {} => String::new(),
            Expr::Ident(ident) => ident.token_literal(),
            Expr::IntegerLiteral(integer_literal) => integer_literal.token_literal(),
            Expr::PrefixExpr(prefix_expr) => prefix_expr.token_literal(),
            Expr::InfixExpr(infix_expr) => infix_expr.token_literal(),
            Expr::Boolean(boolean) => boolean.token_literal(),
            Expr::IfExpr {
                token,
                condition,
                consequence,
                alternative,
            } => token.literal.clone(),
            Expr::FuncLite {
                token,
                parameters,
                body,
            } => token.literal.clone(),
        }
    }
    pub fn string(&self) -> String {
        match self {
            Expr::MockExpr {} => String::new(),
            Expr::Ident(ident) => ident.string(),
            Expr::IntegerLiteral(integer_literal) => integer_literal.string(),
            Expr::PrefixExpr(prefix_expr) => prefix_expr.string(),
            Expr::InfixExpr(infix_expr) => infix_expr.string(),
            Expr::Boolean(boolean) => boolean.string(),
            Expr::IfExpr {
                token,
                condition,
                consequence,
                alternative,
            } => {
                let mut out = String::new();
                out.push_str("if");
                out.push_str(&condition.string());
                out.push_str(" ");
                out.push_str(&consequence.string());
                if let Some(a) = alternative {
                    out.push_str(&a.string());
                }

                out
            }
            Expr::FuncLite {
                token,
                parameters,
                body,
            } => {
                let mut out = String::new();
                let mut params: Vec<String> = Vec::new();

                for p in parameters.iter() {
                    params.push(p.string());
                }

                out.push_str(&token.literal);
                out.push_str("(");
                out.push_str(&params.join(", "));
                out.push_str(") ");
                out.push_str(&body.string());

                out
            }
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

#[derive(Debug, Clone)]
pub struct InfixExpr {
    pub token: Token,
    pub left: Box<Expr>,
    pub operator: String,
    pub right: Box<Expr>,
}
impl InfixExpr {
    pub fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    pub fn string(&self) -> String {
        let mut out = String::new();

        out.push_str("(");
        out.push_str(&self.left.string());
        out.push_str(" ");
        out.push_str(&self.operator);
        out.push_str(" ");
        out.push_str(&self.right.string());
        out.push_str(")");

        out
    }
}

#[derive(Debug, Clone)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
}
impl Boolean {
    pub fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    pub fn string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug, Clone)]
pub struct BlockStmt {
    pub token: Token,
    pub stmts: Vec<Stmt>,
}
impl BlockStmt {
    pub fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    pub fn string(&self) -> String {
        let mut out = String::new();

        for s in self.stmts.iter() {
            out.push_str(&s.string());
        }
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
                value: Expr::Ident(Ident {
                    token: Token {
                        tk_type: TokenType::IDENT,
                        literal: String::from("anotherVar"),
                    },
                    value: String::from("anotherVar"),
                }),
            }],
        };

        assert!(
            program.string() == "let myVar = anotherVar;",
            "program.string() wrong. got={}",
            program.string()
        );
    }
}
