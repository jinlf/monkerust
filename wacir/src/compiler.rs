// src/compiler.rs

use super::ast::*;
use super::code::*;
use super::object::*;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Instructions::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile(&mut self, node: Node) -> Result<String, String> {
        match node {
            Node::Program(Program { statements }) => {
                for s in statements.iter() {
                    match self.compile(Node::Statement(s.clone())) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
                }
            }
            Node::Statement(Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            })) => {
                match self.compile(Node::Expression(expression)) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }
                self.emit(Opcode::OpPop, Vec::new());
            }
            Node::Expression(Expression::InfixExpression(InfixExpression {
                token: _,
                left,
                operator,
                right,
            })) => {
                match self.compile(Node::Expression(*left)) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }
                match self.compile(Node::Expression(*right)) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }
                match &operator[..] {
                    "+" => {
                        self.emit(Opcode::OpAdd, Vec::new());
                    }
                    "-" => {
                        self.emit(Opcode::OpSub, Vec::new());
                    }
                    "*" => {
                        self.emit(Opcode::OpMul, Vec::new());
                    }
                    "/" => {
                        self.emit(Opcode::OpDiv, Vec::new());
                    }
                    _ => return Err(format!("unknown operator {}", operator)),
                }
            }
            Node::Expression(Expression::IntegerLiteral(IntegerLiteral { token: _, value })) => {
                let integer = Integer { value: value };
                let v = vec![self.add_constant(Object::Integer(integer))];
                self.emit(Opcode::OpConstant, v);
            }
            Node::Expression(Expression::BooleanLiteral(BooleanLiteral { token: _, value })) => {
                if value {
                    self.emit(Opcode::OpTrue, Vec::new());
                } else {
                    self.emit(Opcode::OpFalse, Vec::new());
                }
            }
            _ => {}
        }
        return Ok(String::new());
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instuctions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }

    fn add_constant(&mut self, obj: Object) -> i64 {
        self.constants.push(obj);
        self.constants.len() as i64 - 1
    }

    fn emit(&mut self, op: Opcode, operands: Vec<i64>) -> usize {
        let ins = make(op, &operands);
        let pos = self.add_instruction(ins.0);
        pos
    }

    fn add_instruction(&mut self, ins: Vec<u8>) -> usize {
        let pos_new_instruction = self.instructions.0.len();
        self.instructions.0.extend_from_slice(&ins);
        pos_new_instruction
    }
}

pub struct Bytecode {
    pub instuctions: Instructions,
    pub constants: Vec<Object>,
}
