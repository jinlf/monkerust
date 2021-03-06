// src/vm.rs

use super::frame::*;
use crate::code::*;
use crate::compiler::*;
use crate::evaluator::*;
use crate::object::*;

use std::cell::*;
use std::collections::*;
use std::convert::TryInto;
use std::rc::*;

const STACK_SIZE: usize = 2048;
pub const GLOBALS_SIZE: usize = 65536;
const MAX_FRAMES: usize = 1024;
pub const TRUE: Object = Object::Boolean(Boolean { value: true });
pub const FALSE: Object = Object::Boolean(Boolean { value: false });
pub const NULL: Object = Object::Null(Null {});

pub struct Vm {
    pub constants: Rc<RefCell<Vec<Object>>>,
    pub stack: Vec<Option<Object>>,
    pub sp: usize, // Always points to the next value. Top of stack is stack[sp-1]
    globals: Rc<RefCell<Vec<Option<Object>>>>,
    frames: Vec<Frame>,
    frame_index: usize,
    pub last_popped_stack_elem: Option<Object>,
}
impl Vm {
    pub fn new(bytecode: Bytecode) -> Vm {
        let main_fn = CompiledFunction {
            instructions: bytecode.instuctions,
            num_locals: 0,
            num_parameters: 0,
        };
        let main_closure = Closure {
            func: main_fn,
            free: Vec::new(),
        };
        let mut frames: Vec<Frame> = vec![
            Frame::new(
                Closure {
                    func: CompiledFunction {
                        instructions: Instructions::new(),
                        num_locals: 0,
                        num_parameters: 0,
                    },
                    free: Vec::new(),
                },
                0
            );
            MAX_FRAMES
        ];
        let main_frame = Frame::new(main_closure, 0);
        frames[0] = main_frame;
        Vm {
            constants: Rc::clone(&bytecode.constants),
            stack: vec![None; STACK_SIZE],
            sp: 0, // Always points to the next value. Top of stack is stack[sp-1]
            globals: Rc::new(RefCell::new(vec![None; GLOBALS_SIZE])),
            frames: frames,
            frame_index: 1,
            last_popped_stack_elem: None,
        }
    }

    // pub fn stack_top(&self) -> Option<Object> {
    //     if self.sp == 0 {
    //         None
    //     } else {
    //         self.stack[(self.sp - 1) as usize].clone()
    //     }
    // }

    pub fn run(&mut self) -> Result<(), String> {
        let mut ip: usize;
        let mut ins: &Instructions;
        let mut op: Opcode;
        while self.current_frame().ip < self.current_frame().instructions().0.len() as i64 - 1 {
            self.current_frame().ip += 1;

            ip = self.current_frame().ip as usize;
            ins = self.current_frame().instructions();
            op = Opcode::from(ins.0[ip]);
            match op {
                Opcode::OpConstant => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let const_index = read_u16(src);
                    self.current_frame().ip += 2;
                    let obj = self.constants.borrow()[const_index as usize].clone();
                    self.push(obj)?;
                }
                Opcode::OpAdd | Opcode::OpSub | Opcode::OpMul | Opcode::OpDiv => {
                    self.execute_binary_operation(op)?;
                }
                Opcode::OpPop => {
                    self.pop();
                }
                Opcode::OpTrue => {
                    self.push(TRUE)?;
                }
                Opcode::OpFalse => {
                    self.push(FALSE)?;
                }
                Opcode::OpEqual | Opcode::OpNotEqual | Opcode::OpGreaterThan => {
                    self.execute_comparison(op)?;
                }
                Opcode::OpBang => {
                    self.execute_bang_operator()?;
                }
                Opcode::OpMinus => {
                    self.execute_minus_operator()?;
                }
                Opcode::OpJump => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let pos = read_u16(src) as i64;
                    self.current_frame().ip = pos - 1;
                }
                Opcode::OpJumpNotTruthy => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let pos = read_u16(src) as i64;
                    self.current_frame().ip += 2;
                    let condition = self.pop();
                    if !is_truthy(&condition) {
                        self.current_frame().ip = pos - 1;
                    }
                }
                Opcode::OpNull => {
                    self.push(NULL)?;
                }
                Opcode::OpSetGlobal => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let global_index = read_u16(src) as usize;
                    self.current_frame().ip += 2;
                    self.globals.borrow_mut()[global_index] = self.pop();
                }
                Opcode::OpGetGlobal => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let global_index = read_u16(src) as usize;
                    self.current_frame().ip += 2;
                    let obj = self.globals.borrow_mut()[global_index]
                        .as_ref()
                        .unwrap()
                        .clone(); // TODO: can unwrap?
                    self.push(obj)?;
                }
                Opcode::OpArray => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let num_elements = read_u16(src) as usize;
                    self.current_frame().ip += 2;

                    let array = self.build_array(self.sp - num_elements, self.sp);
                    self.sp -= num_elements;

                    self.push(array)?;
                }
                Opcode::OpHash => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let num_elements = read_u16(src) as usize;
                    self.current_frame().ip += 2;
                    let hash = self.build_hash(self.sp - num_elements, self.sp)?;
                    self.sp -= num_elements;
                    self.push(hash)?;
                }
                Opcode::OpIndex => {
                    let index = self.pop();
                    let left = self.pop();
                    self.execute_index_expression(&left, &index)?;
                }
                Opcode::OpCall => {
                    let num_args = ins.0[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    self.execute_call(num_args)?;
                }
                Opcode::OpReturnValue => {
                    let return_value = self.pop();
                    let base_pointer = self.pop_frame().base_pointer;
                    self.sp = base_pointer - 1;
                    self.push(return_value.unwrap())?;
                }
                Opcode::OpReturn => {
                    let base_pointer = self.pop_frame().base_pointer;
                    self.sp = base_pointer - 1;

                    self.push(NULL)?;
                }
                Opcode::OpSetLocal => {
                    let local_index = ins.0[ip + 1] as usize;
                    self.current_frame().ip += 1;

                    let base_pointer = self.current_frame().base_pointer;
                    self.stack[base_pointer + local_index] = self.pop();
                }
                Opcode::OpGetLocal => {
                    let local_index = ins.0[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let base_pointer = self.current_frame().base_pointer;
                    let obj = self.stack[base_pointer + local_index].as_ref().unwrap(); // TODO: can unwrap?
                    let obj_clone = obj.clone();
                    self.push(obj_clone)?;
                }
                Opcode::OpGetBuiltin => {
                    let builtin_index = ins.0[ip + 1] as usize;
                    self.current_frame().ip += 1;

                    let definition =
                        get_builtin_by_name(&get_builtin_names()[builtin_index]).unwrap(); //TODO: can unwrap?

                    self.push(Object::Builtin(definition))?;
                }
                Opcode::OpClosure => {
                    let src = ins.0[(ip + 1)..(ip + 3)].try_into().expect("wrong size");
                    let const_index = read_u16(src) as usize;
                    let num_free = ins.0[ip + 3] as usize;
                    self.current_frame().ip += 3;

                    self.push_closure(const_index, num_free)?;
                }
                Opcode::OpGetFree => {
                    let free_index = ins.0[ip + 1] as usize;
                    self.current_frame().ip += 1;

                    let current_closure = &self.current_frame().cl;
                    let free = current_closure.free[free_index].as_ref().unwrap().clone();
                    self.push(free)?;
                }
            }
        }
        Ok(())
    }

    pub fn push(&mut self, o: Object) -> Result<(), String> {
        if self.sp >= STACK_SIZE {
            return Err(String::from("stack overflow"));
        }
        self.stack[self.sp] = Some(o);
        self.sp += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Object> {
        let o = std::mem::replace(&mut self.stack[self.sp - 1], None);
        self.last_popped_stack_elem = o.clone();
        self.sp -= 1;
        o
    }

    // pub fn last_popped_stack_elem(&self) -> Option<Object> {
    //     self.stack[self.sp].clone()
    // }

    fn execute_binary_operation(&mut self, op: Opcode) -> Result<(), String> {
        let right = self.pop();
        let left = self.pop();
        if let Some(Object::Integer(Integer { value })) = &left {
            let left_value = value;
            if let Some(Object::Integer(Integer { value })) = &right {
                let right_value = value;
                return self.execute_binary_integer_operation(op, *left_value, *right_value);
            }
        } else if let Some(Object::StringObj(StringObj { value })) = &left {
            let left_value = value;
            if let Some(Object::StringObj(StringObj { value })) = &right {
                let right_value = value;
                return self.execute_binary_string_operation(op, left_value, right_value);
            }
        }
        Err(format!(
            "unsupported types for binary operation: {} {}",
            get_type(&left),
            get_type(&right),
        ))
    }

    fn execute_binary_integer_operation(
        &mut self,
        op: Opcode,
        left_value: i64,
        right_value: i64,
    ) -> Result<(), String> {
        let result = match op {
            Opcode::OpAdd => left_value + right_value,
            Opcode::OpSub => left_value - right_value,
            Opcode::OpMul => left_value * right_value,
            Opcode::OpDiv => left_value / right_value,
            _ => return Err(format!("unknown integer operator: {:?}", op)),
        };
        self.push(Object::Integer(Integer { value: result }))?;
        Ok(())
    }

    fn execute_comparison(&mut self, op: Opcode) -> Result<(), String> {
        let right = self.pop();
        let left = self.pop();

        if let Some(Object::Integer(Integer { value })) = right {
            let right_value = value;
            if let Some(Object::Integer(Integer { value })) = left {
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
    ) -> Result<(), String> {
        match op {
            Opcode::OpEqual => self.push(native_bool_to_boolean_object(right == left)),
            Opcode::OpNotEqual => self.push(native_bool_to_boolean_object(right != left)),
            Opcode::OpGreaterThan => self.push(native_bool_to_boolean_object(left > right)),
            _ => return Err(format!("unknown operator: {:?}", op)),
        }
    }

    fn execute_bang_operator(&mut self) -> Result<(), String> {
        let operand = self.pop();
        match operand {
            Some(TRUE) => self.push(FALSE),
            Some(FALSE) => self.push(TRUE),
            Some(NULL) => self.push(TRUE),
            _ => self.push(FALSE),
        }
    }
    fn execute_minus_operator(&mut self) -> Result<(), String> {
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

    pub fn new_with_globals_store(bytecode: Bytecode, s: Rc<RefCell<Vec<Option<Object>>>>) -> Vm {
        let main_fn = CompiledFunction {
            instructions: bytecode.instuctions.clone(),
            num_locals: 0,
            num_parameters: 0,
        };
        let main_closure = Closure {
            func: main_fn,
            free: Vec::new(),
        };
        let main_frame = Frame::new(main_closure, 0);
        let mut frames: Vec<Frame> = vec![
            Frame::new(
                Closure {
                    func: CompiledFunction {
                        instructions: Instructions::new(),
                        num_locals: 0,
                        num_parameters: 0,
                    },
                    free: Vec::new()
                },
                0
            );
            MAX_FRAMES
        ];
        frames[0] = main_frame;

        Vm {
            constants: bytecode.constants,

            stack: vec![None; STACK_SIZE],
            sp: 0, // Always points to the next value. Top of stack is stack[sp-1]
            globals: Rc::clone(&s),
            frames: frames,
            frame_index: 1,
            last_popped_stack_elem: None,
        }
    }

    fn execute_binary_string_operation(
        &mut self,
        op: Opcode,
        left: &str,
        right: &str,
    ) -> Result<(), String> {
        if op != Opcode::OpAdd {
            return Err(format!("unknown string operator: {:?}", op));
        }
        self.push(Object::StringObj(StringObj {
            value: format!("{}{}", left, right),
        }))
    }

    fn build_array(&mut self, start_index: usize, end_index: usize) -> Object {
        let mut elements: Vec<Object> = vec![NULL; end_index - start_index];
        let mut i = start_index;
        while i < end_index {
            elements[i - start_index] = std::mem::replace(&mut self.stack[i], None).unwrap();
            i += 1;
        }
        Object::Array(Array { elements: elements })
    }

    fn build_hash(&self, start_index: usize, end_index: usize) -> Result<Object, String> {
        let mut hashed_pairs: HashMap<HashKey, Object> = HashMap::new();
        let mut i = start_index;
        while i < end_index {
            let key = &self.stack[i];
            let value = &self.stack[i + 1];

            if let Some(key_obj) = key {
                if let Some(hashable) = key_obj.as_hashable() {
                    let hash_key = hashable.hash_key();
                    if let Some(value_obj) = value {
                        hashed_pairs.insert(hash_key, value_obj.clone());
                    } else {
                        return Err(format!("uninitialized value"));
                    }
                } else {
                    return Err(format!("unusable as hash key: {}", get_type(&key)));
                }
            } else {
                return Err(format!("unusable as hash key: {}", get_type(&key)));
            }

            i += 2;
        }

        Ok(Object::Hash(Hash {
            pairs: hashed_pairs,
        }))
    }

    fn execute_index_expression(
        &mut self,
        left: &Option<Object>,
        index: &Option<Object>,
    ) -> Result<(), String> {
        if let Some(Object::Array(Array { elements })) = left {
            if let Some(Object::Integer(Integer { value })) = index {
                return self.execute_array_index(elements, *value);
            }
        } else if let Some(Object::Hash(Hash { pairs })) = left {
            return self.execute_hash_index(pairs, index);
        }
        Err(format!("index operator not supported: {}", get_type(&left)))
    }

    fn execute_array_index(&mut self, elements: &Vec<Object>, index: i64) -> Result<(), String> {
        let max = elements.len() as i64 - 1;
        if index < 0 || index > max {
            return self.push(NULL);
        }

        self.push(elements[index as usize].clone())
    }

    fn execute_hash_index(
        &mut self,
        pairs: &HashMap<HashKey, Object>,
        index: &Option<Object>,
    ) -> Result<(), String> {
        if let Some(key) = index {
            if let Some(hash_key) = key.as_hashable() {
                if let Some(value) = pairs.get(&hash_key.hash_key()) {
                    return self.push(value.clone());
                } else {
                    return self.push(NULL);
                }
            } else {
                return Err(format!("unusable as hash key: {}", get_type(&index)));
            }
        } else {
            return Err(format!("unusable as hash key: {}", get_type(&index)));
        }
    }

    fn current_frame(&mut self) -> &mut Frame {
        &mut self.frames[self.frame_index - 1]
    }

    fn push_frame(&mut self, f: Frame) {
        self.frames[self.frame_index] = f;
        self.frame_index += 1;
    }

    fn pop_frame(&mut self) -> Frame {
        self.frame_index -= 1;
        self.frames[self.frame_index].clone()
    }

    // fn call_function(&mut self, func: CompiledFunction, num_args: usize) -> Result<(), String> {
    //     if num_args != func.num_parameters {
    //         return Err(format!(
    //             "wrong number of arguments: want={}, got={}",
    //             func.num_parameters, num_args
    //         ));
    //     }

    //     let frame = Frame::new(func.clone(), self.sp - num_args);
    //     self.sp = frame.base_pointer + func.num_locals;
    //     self.push_frame(frame);
    //     return Ok(());
    // }

    fn execute_call(&mut self, num_args: usize) -> Result<(), String> {
        let callee = self.stack[self.sp - 1 - num_args].clone();
        if let Some(Object::Closure(cl)) = callee {
            return self.call_closure(&cl, num_args);
        } else if let Some(Object::Builtin(builtin)) = callee {
            return self.call_builtin(&builtin, num_args);
        } else {
            return Err(format!("calling non-function and no-built-in"));
        }
    }

    fn call_builtin(&mut self, builtin: &Builtin, num_args: usize) -> Result<(), String> {
        let mut v: Vec<Object> = Vec::new();
        for i in 0..num_args {
            v.push(std::mem::replace(&mut self.stack[self.sp - num_args + i], None).unwrap());
        }
        let func = builtin.func;
        let result = func(&v)?;
        self.sp = self.sp - num_args - 1;
        self.push(result)?;
        Ok(())
    }

    fn call_closure(&mut self, cl: &Closure, num_args: usize) -> Result<(), String> {
        if num_args != cl.func.num_parameters {
            return Err(format!(
                "wrong number of arguments: want={}, got={}",
                cl.func.num_parameters, num_args
            ));
        }

        let num_locals = cl.func.num_locals;
        let frame = Frame::new(cl.clone(), self.sp - num_args);
        let base_pointer = frame.base_pointer;
        self.push_frame(frame);
        self.sp = base_pointer + num_locals;
        Ok(())
    }

    fn push_closure(&mut self, const_index: usize, num_free: usize) -> Result<(), String> {
        let constant = self.constants.borrow()[const_index].clone();
        if let Object::CompiledFunction(function) = constant {
            let mut free: Vec<Option<Object>> = vec![None; num_free];
            let mut i = 0;
            while i < num_free {
                free[i] = self.stack[self.sp - num_free + i].clone();
                i += 1;
            }
            self.sp -= num_free;

            let closure = Closure {
                func: function,
                free: free,
            };
            self.push(Object::Closure(closure))
        } else {
            Err(format!("not a function: {:?}", constant))
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

pub fn get_type(obj: &Option<Object>) -> &str {
    if obj.is_some() {
        obj.as_ref().unwrap().get_type()
    } else {
        "None"
    }
}

fn is_truthy(obj: &Option<Object>) -> bool {
    if let Some(Object::Boolean(Boolean { value })) = obj {
        return *value;
    } else if let Some(NULL) = obj {
        return false;
    }
    true
}
