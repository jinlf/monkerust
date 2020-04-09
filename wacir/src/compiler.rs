// src/compiler.rs

use super::ast::*;
use super::code::*;
use super::object::*;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,

    pub last_instruction: Option<EmittedInstruction>,
    pub previous_instruction: Option<EmittedInstruction>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Instructions::new(),
            constants: Vec::new(),
            last_instruction: None,
            previous_instruction: None,
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
                if operator == "<" {
                    match self.compile(Node::Expression(*right)) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
                    match self.compile(Node::Expression(*left)) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
                    self.emit(Opcode::OpGreaterThan, Vec::new());
                } else {
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
                        ">" => {
                            self.emit(Opcode::OpGreaterThan, Vec::new());
                        }
                        "==" => {
                            self.emit(Opcode::OpEqual, Vec::new());
                        }
                        "!=" => {
                            self.emit(Opcode::OpNotEqual, Vec::new());
                        }
                        _ => return Err(format!("unknown operator {}", operator)),
                    }
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
            Node::Expression(Expression::PrefixExpression(PrefixExpression {
                token: _,
                operator,
                right,
            })) => {
                match self.compile(Node::Expression(*right)) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                };
                match &operator[..] {
                    "!" => self.emit(Opcode::OpBang, Vec::new()),
                    "-" => self.emit(Opcode::OpMinus, Vec::new()),
                    _ => return Err(format!("unknown operator {}", operator)),
                };
            }
            Node::Expression(Expression::IfExpression(IfExpression {
                token: _,
                condition,
                consequence,
                alternative,
            })) => {
                match self.compile(Node::Expression(*condition)) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                };
                let jump_not_truthy_pos = self.emit(Opcode::OpJumpNotTruthy, vec![9999]);
                match self.compile(Node::Statement(Statement::BlockStatement(consequence))) {
                    Ok(_) => {}
                    Err(err) => return Err(err),
                }

                if self.last_instruction_is_pop() {
                    self.remove_last_pop();
                }

                let jump_pos = self.emit(Opcode::OpJump, vec![9999]);

                let after_consequence_pos = self.instructions.0.len();
                self.change_operand(jump_not_truthy_pos, after_consequence_pos as i64);

                if let Some(a) = alternative {
                    match self.compile(Node::Statement(Statement::BlockStatement(a))) {
                        Ok(_) => {
                            if self.last_instruction_is_pop() {
                                self.remove_last_pop();
                            }
                        }
                        Err(err) => return Err(err),
                    }
                } else {
                    self.emit(Opcode::OpNull, Vec::new());
                }
                let after_alternative_pos = self.instructions.0.len();
                self.change_operand(jump_pos, after_alternative_pos as i64);
            }
            Node::Statement(Statement::BlockStatement(BlockStatement {
                token: _,
                statements,
            })) => {
                for s in statements.iter() {
                    match self.compile(Node::Statement(s.clone())) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
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

        self.set_last_instruction(op, pos);
        pos
    }

    fn add_instruction(&mut self, ins: Vec<u8>) -> usize {
        let pos_new_instruction = self.instructions.0.len();
        self.instructions.0.extend_from_slice(&ins);
        pos_new_instruction
    }

    fn set_last_instruction(&mut self, op: Opcode, pos: usize) {
        let previous = self.last_instruction.clone();
        let last = EmittedInstruction {
            opcode: op,
            position: pos,
        };

        self.previous_instruction = previous;
        self.last_instruction = Some(last);
    }

    fn last_instruction_is_pop(&self) -> bool {
        if let Some(EmittedInstruction {
            opcode,
            position: _,
        }) = self.last_instruction
        {
            if opcode == Opcode::OpPop {
                return true;
            }
        }
        false
    }

    fn remove_last_pop(&mut self) {
        self.instructions.0.pop();
        self.last_instruction = self.previous_instruction.clone();
    }

    fn replace_instruction(&mut self, pos: usize, new_instruction: Instructions) {
        self.instructions.0[pos..(pos + new_instruction.0.len())]
            .copy_from_slice(&new_instruction.0)
    }

    fn change_operand(&mut self, op_pos: usize, operand: i64) {
        let op = Opcode::from(self.instructions.0[op_pos]);
        let new_instruction = make(op, &vec![operand]);

        self.replace_instruction(op_pos, new_instruction);
    }
}

pub struct Bytecode {
    pub instuctions: Instructions,
    pub constants: Vec<Object>,
}

#[derive(Clone)]
pub struct EmittedInstruction {
    pub opcode: Opcode,
    pub position: usize,
}
