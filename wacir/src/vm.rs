// src/vm.rs

use super::code::*;
use super::compiler::*;
use super::object::*;
use std::convert::TryInto;

const STACK_SIZE: usize = 2048;
const GLOBALS_SIZE: usize = 65536;
pub const TRUE: Object = Object::Boolean(Boolean { value: true });
pub const FALSE: Object = Object::Boolean(Boolean { value: false });
pub const NULL: Object = Object::Null(Null {});

pub struct Vm {
    pub constants: Vec<Object>,
    pub instructions: Instructions,
    pub stack: Vec<Option<Object>>,
    pub sp: usize, // Always points to the next value. Top of stack is stack[sp-1]
    globals: Vec<Option<Object>>,
}
impl Vm {
    pub fn new(bytecode: Bytecode) -> Vm {
        Vm {
            instructions: bytecode.instuctions,
            constants: bytecode.constants,

            stack: vec![None; STACK_SIZE],
            sp: 0, // Always points to the next value. Top of stack is stack[sp-1]
            globals: vec![None; GLOBALS_SIZE],
        }
    }

    pub fn stack_top(&self) -> Option<Object> {
        if self.sp == 0 {
            None
        } else {
            self.stack[(self.sp - 1) as usize].clone()
        }
    }

    pub fn run(&mut self) -> Result<String, String> {
        let mut ip = 0;
        while ip < self.instructions.0.len() {
            let op = Opcode::from(self.instructions.0[ip]);
            match op {
                Opcode::OpConstant => {
                    let src = self.instructions.0[(ip + 1)..(ip + 3)]
                        .try_into()
                        .expect("wrong size");
                    let const_index = read_u16(src);
                    ip += 2;
                    match self.push(self.constants[const_index as usize].clone()) {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpAdd | Opcode::OpSub | Opcode::OpMul | Opcode::OpDiv => {
                    match self.execute_binary_operation(op) {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpPop => {
                    self.pop();
                }
                Opcode::OpTrue => {
                    match self.push(TRUE) {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpFalse => {
                    match self.push(FALSE) {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpEqual | Opcode::OpNotEqual | Opcode::OpGreaterThan => {
                    match self.execute_comparison(op) {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpBang => {
                    match self.execute_bang_operator() {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpMinus => {
                    match self.execute_minus_operator() {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpJump => {
                    let src = self.instructions.0[(ip + 1)..(ip + 3)]
                        .try_into()
                        .expect("wrong size");
                    let pos = read_u16(src) as usize;
                    ip = pos - 1;
                }
                Opcode::OpJumpNotTruthy => {
                    let src = self.instructions.0[(ip + 1)..(ip + 3)]
                        .try_into()
                        .expect("wrong size");
                    let pos = read_u16(src) as usize;
                    ip += 2;
                    let condition = self.pop();
                    if !is_truthy(condition) {
                        ip = pos - 1;
                    }
                }
                Opcode::OpNull => {
                    match self.push(NULL) {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
                Opcode::OpSetGlobal => {
                    let src = self.instructions.0[(ip + 1)..(ip + 3)]
                        .try_into()
                        .expect("wrong size");
                    let global_index = read_u16(src) as usize;
                    ip += 2;
                    self.globals[global_index] = self.pop();
                }
                Opcode::OpGetGlobal => {
                    let src = self.instructions.0[(ip + 1)..(ip + 3)]
                        .try_into()
                        .expect("wrong size");
                    let global_index = read_u16(src) as usize;
                    ip += 2;
                    match self.push(self.globals[global_index].as_ref().unwrap().clone()) {
                        Err(err) => return Err(err),
                        _ => {}
                    };
                }
            }
            ip += 1;
        }
        Ok(String::new())
    }

    pub fn push(&mut self, o: Object) -> Result<String, String> {
        if self.sp >= STACK_SIZE {
            return Err(String::from("stack overflow"));
        }
        self.stack[self.sp] = Some(o);
        self.sp += 1;
        Ok(String::new())
    }

    pub fn pop(&mut self) -> Option<Object> {
        let o = self.stack[self.sp - 1].clone();
        self.sp -= 1;
        o
    }

    pub fn last_popped_stack_elem(&self) -> Option<Object> {
        self.stack[self.sp].clone()
    }

    fn execute_binary_operation(&mut self, op: Opcode) -> Result<String, String> {
        let right = self.pop();
        let left = self.pop();
        if let Some(Object::Integer(Integer { value })) = left {
            let left_value = value;
            if let Some(Object::Integer(Integer { value })) = right {
                let right_value = value;
                return self.execute_binary_integer_operation(op, left_value, right_value);
            } else {
                return Err(String::from("error"));
            }
        } else {
            return Err(String::from("error"));
        }
    }

    fn execute_binary_integer_operation(
        &mut self,
        op: Opcode,
        left_value: i64,
        right_value: i64,
    ) -> Result<String, String> {
        let result = match op {
            Opcode::OpAdd => left_value + right_value,
            Opcode::OpSub => left_value - right_value,
            Opcode::OpMul => left_value * right_value,
            Opcode::OpDiv => left_value / right_value,
            _ => return Err(format!("unknown integer operator: {:?}", op)),
        };
        match self.push(Object::Integer(Integer { value: result })) {
            Err(err) => return Err(err),
            _ => {}
        };
        Ok(String::new())
    }

    fn execute_comparison(&mut self, op: Opcode) -> Result<String, String> {
        let right = self.pop();
        let left = self.pop();

        if let Some(Object::Integer(Integer { value })) = right.clone() {
            let right_value = value;
            if let Some(Object::Integer(Integer { value })) = left.clone() {
                let left_value = value;
                return self.execute_integer_comparison(op, left_value, right_value);
            }
        }

        match op {
            Opcode::OpEqual => return self.push(native_bool_to_boolean_object(left == right)),
            Opcode::OpNotEqual => return self.push(native_bool_to_boolean_object(left != right)),
            _ => {
                return Err(format!(
                    "unknown operator: {:?} ({:?} {:?})",
                    op, left, right
                ));
            }
        }
    }

    fn execute_integer_comparison(
        &mut self,
        op: Opcode,
        left: i64,
        right: i64,
    ) -> Result<String, String> {
        match op {
            Opcode::OpEqual => self.push(native_bool_to_boolean_object(right == left)),
            Opcode::OpNotEqual => self.push(native_bool_to_boolean_object(right != left)),
            Opcode::OpGreaterThan => self.push(native_bool_to_boolean_object(left > right)),
            _ => return Err(format!("unknown operator: {:?}", op)),
        }
    }

    fn execute_bang_operator(&mut self) -> Result<String, String> {
        let operand = self.pop();
        match operand {
            Some(TRUE) => self.push(FALSE),
            Some(FALSE) => self.push(TRUE),
            Some(NULL) => self.push(TRUE),
            _ => self.push(FALSE),
        }
    }
    fn execute_minus_operator(&mut self) -> Result<String, String> {
        let operand = self.pop();
        match operand {
            Some(Object::Integer(Integer { value })) => {
                return self.push(Object::Integer(Integer { value: -value }));
            }
            _ => {
                return Err(format!(
                    "unsupported type for negation: {}",
                    get_type(&operand)
                ))
            }
        }
    }
}

fn native_bool_to_boolean_object(input: bool) -> Object {
    if input {
        Object::Boolean(Boolean { value: true })
    } else {
        Object::Boolean(Boolean { value: false })
    }
}

pub fn get_type(obj: &Option<Object>) -> String {
    if obj.is_some() {
        obj.as_ref().unwrap().get_type()
    } else {
        String::from("None")
    }
}

fn is_truthy(obj: Option<Object>) -> bool {
    if let Some(Object::Boolean(Boolean { value })) = obj {
        return value;
    } else if let Some(NULL) = obj {
        return false;
    }
    true
}
