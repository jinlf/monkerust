use super::token::*;
use std::collections::*;
use std::fmt::*;
use std::hash::*;

pub trait NodeTrait {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Node {
    Program(Program),
    Stmt(Stmt),
    Expr(Expr),
}
impl NodeTrait for Node {
    fn token_literal(&self) -> String {
        match self {
            Node::Program(program) => program.token_literal(),
            Node::Stmt(stmt) => stmt.token_literal(),
            Node::Expr(expr) => expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Node::Program(program) => program.string(),
            Node::Stmt(stmt) => stmt.string(),
            Node::Expr(expr) => expr.string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
impl NodeTrait for Program {
    fn token_literal(&self) -> String {
        if self.stmts.len() > 0 {
            self.stmts[0].token_literal()
        } else {
            String::new()
        }
    }
    fn string(&self) -> String {
        let mut out = String::new();
        for s in self.stmts.iter() {
            out.push_str(&s.string());
        }
        out
    }
}

pub trait StmtTrait: NodeTrait {
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
        value: Expr,
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
impl NodeTrait for Stmt {
    fn token_literal(&self) -> String {
        match self {
            Stmt::LetStmt {
                token,
                name: _,
                value: _,
            } => token.literal.clone(),
            Stmt::ReturnStmt { token, value: _ } => token.literal.clone(),
            Stmt::ExprStmt { token, expr: _ } => token.literal.clone(),
            Stmt::BlockStmt(block_stmt) => block_stmt.token_literal(),
        }
    }
    fn string(&self) -> String {
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
            Stmt::ReturnStmt { token, value } => {
                let mut out = String::new();

                out.push_str(&token.literal);
                out.push_str(" ");
                out.push_str(&value.string());
                out.push_str(";");
                out
            }
            Stmt::ExprStmt { token: _, expr } => expr.string(),
            Stmt::BlockStmt(block_stmt) => block_stmt.string(),
        }
    }
}

pub trait ExprTrait: NodeTrait {
    fn expr_node(&self);
}

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(Ident),
    IntLiteral(IntLiteral),
    PrefixExpr(PrefixExpr),
    InfixExpr(InfixExpr),
    Bool {
        token: Token,
        value: bool,
    },
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
    CallExpr {
        token: Token,
        func: Box<Expr>,
        arguments: Vec<Option<Expr>>,
    },
    StrLite {
        token: Token,
        value: String,
    },
    ArrayLite {
        token: Token,
        elements: Vec<Option<Expr>>,
    },
    IndexExpr {
        token: Token,
        left: Box<Expr>,
        index: Box<Expr>,
    },
    HashLite(HashLite),
}
impl Eq for Expr {}
impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        let addr1 = self as *const Expr as usize;
        let addr2 = other as *const Expr as usize;
        return addr1 == addr2;
    }
}
impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let addr = self as *const Expr as usize;
        state.write_usize(addr);
        state.finish();
    }
}
impl ExprTrait for Expr {
    fn expr_node(&self) {}
}
impl NodeTrait for Expr {
    fn token_literal(&self) -> String {
        match self {
            Expr::Ident(ident) => ident.token_literal(),
            Expr::IntLiteral(integer_literal) => integer_literal.token_literal(),
            Expr::PrefixExpr(prefix_expr) => prefix_expr.token_literal(),
            Expr::InfixExpr(infix_expr) => infix_expr.token_literal(),
            Expr::Bool { token, value: _ } => token.literal.clone(),
            Expr::IfExpr {
                token,
                condition: _,
                consequence: _,
                alternative: _,
            } => token.literal.clone(),
            Expr::FuncLite {
                token,
                parameters: _,
                body: _,
            } => token.literal.clone(),
            Expr::CallExpr {
                token,
                func: _,
                arguments: _,
            } => token.literal.clone(),
            Expr::StrLite { token, value: _ } => token.literal.clone(),
            Expr::ArrayLite { token, elements: _ } => token.literal.clone(),
            Expr::IndexExpr {
                token,
                left: _,
                index: _,
            } => token.literal.clone(),
            Expr::HashLite(HashLite { token, pairs: _ }) => token.literal.clone(),
        }
    }
    fn string(&self) -> String {
        match self {
            Expr::Ident(ident) => ident.string(),
            Expr::IntLiteral(integer_literal) => integer_literal.string(),
            Expr::PrefixExpr(prefix_expr) => prefix_expr.string(),
            Expr::InfixExpr(infix_expr) => infix_expr.string(),
            Expr::Bool { token, value: _ } => token.literal.clone(),
            Expr::IfExpr {
                token: _,
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
            Expr::CallExpr {
                token: _,
                func,
                arguments,
            } => {
                let mut out = String::new();
                let mut args: Vec<String> = Vec::new();
                for a in arguments.iter() {
                    args.push(a.as_ref().unwrap().string()); //TODO argument must not None
                }
                out.push_str(&func.string());
                out.push_str("(");
                out.push_str(&args.join(", "));
                out.push_str(")");
                out
            }
            Expr::StrLite { token, value: _ } => token.literal.clone(),
            Expr::ArrayLite { token: _, elements } => {
                let mut out = String::new();
                let mut elems: Vec<String> = Vec::new();
                for el in elements.iter() {
                    elems.push(el.as_ref().unwrap().string()); //TODO array item must not None
                }
                out.push_str("[");
                out.push_str(&elems.join(", "));
                out.push_str("]");
                out
            }
            Expr::IndexExpr {
                token: _,
                left,
                index,
            } => {
                let mut out = String::new();
                out.push_str("(");
                out.push_str(&left.string());
                out.push_str("[");
                out.push_str(&index.string());
                out.push_str("])");
                out
            }
            Expr::HashLite(HashLite { token: _, pairs }) => {
                let mut out = String::new();
                let mut pairs1: Vec<String> = Vec::new();
                for (key, value) in pairs.iter() {
                    pairs1.push(format!("{}:{}", key.string(), value.string()));
                }
                out.push_str("{");
                out.push_str(&pairs1.join(", "));
                out.push_str("}");
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
impl NodeTrait for Ident {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct IntLiteral {
    pub token: Token,
    pub value: i64,
}
impl NodeTrait for IntLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("{}", self.value)
    }
}

#[derive(Debug, Clone)]
pub struct PrefixExpr {
    pub token: Token,
    pub operator: String,
    pub right: Box<Expr>,
}
impl NodeTrait for PrefixExpr {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
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
impl NodeTrait for InfixExpr {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
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
pub struct BlockStmt {
    pub token: Token,
    pub stmts: Vec<Stmt>,
}
impl NodeTrait for BlockStmt {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut out = String::new();

        for s in self.stmts.iter() {
            out.push_str(&s.string());
        }
        out
    }
}

#[derive(Clone)]
pub struct HashLite {
    pub token: Token,
    pub pairs: HashMap<Expr, Expr>,
}
impl Debug for HashLite {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "HashLite")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let program = Program {
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
