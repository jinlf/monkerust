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
            })) => match self.compile(Node::Expression(expression)) {
                Ok(_) => {}
                Err(err) => return Err(err),
            },

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
            }
            Node::Expression(Expression::IntegerLiteral(IntegerLiteral { token: _, value })) => {
                let integer = Integer { value: value };
                let v = vec![self.add_constant(Object::Integer(integer))];
                self.emit(Opcode::OpConstant, v);
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

    fn add_constant(&mut self, obj: Object) -> isize {
        self.constants.push(obj);
        self.constants.len() as isize - 1
    }

    fn emit(&mut self, op: Opcode, operands: Vec<isize>) -> usize {
        let ins = make(op, &operands);
        let pos = self.add_instruction(ins.into());
        pos
    }

    fn add_instruction(&mut self, ins: Vec<u8>) -> usize {
        let pos_new_instruction = self.instructions.content.len();
        self.instructions.content.extend_from_slice(&ins);
        pos_new_instruction
    }
}

pub struct Bytecode {
    pub instuctions: Instructions,
    pub constants: Vec<Object>,
}
