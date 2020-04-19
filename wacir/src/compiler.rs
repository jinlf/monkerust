// src/compiler.rs

use super::ast::*;
use super::builtins::*;
use super::code::*;
use super::object::*;
use super::symbol_table::*;
use std::cell::*;
use std::rc::*;

pub struct Compiler {
    pub constants: Rc<RefCell<Vec<Object>>>,
    pub symbol_table: Rc<RefCell<SymbolTable>>,
    pub scopes: Vec<CompilationScope>,
    pub scope_index: usize,
}

impl Compiler {
    pub fn new() -> Compiler {
        let mut symbol_table = SymbolTable::new();
        for (i, v) in get_builtin_names().iter().enumerate() {
            symbol_table.define_builtin(i, &v);
        }
        let constants = Rc::new(RefCell::new(Vec::new()));
        let symbol_table = Rc::new(RefCell::new(symbol_table));
        let main_scope = CompilationScope {
            instructions: Instructions::new(),
            last_instruction: None,
            previous_instruction: None,
        };
        Compiler {
            constants: Rc::clone(&constants),
            symbol_table: Rc::clone(&symbol_table),
            scopes: vec![main_scope],
            scope_index: 0,
        }
    }

    pub fn compile(&mut self, node: Node) -> Result<(), String> {
        match node {
            Node::Program(Program { statements }) => {
                for s in statements.iter() {
                    self.compile(Node::Statement(s.clone()))?;
                }
            }
            Node::Statement(Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            })) => {
                self.compile(Node::Expression(expression))?;
                self.emit(Opcode::OpPop, Vec::new());
            }
            Node::Expression(Expression::InfixExpression(InfixExpression {
                token: _,
                left,
                operator,
                right,
            })) => {
                if operator == "<" {
                    self.compile(Node::Expression(*right))?;
                    self.compile(Node::Expression(*left))?;
                    self.emit(Opcode::OpGreaterThan, Vec::new());
                } else {
                    self.compile(Node::Expression(*left))?;
                    self.compile(Node::Expression(*right))?;
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
                self.compile(Node::Expression(*right))?;
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
                self.compile(Node::Expression(*condition))?;
                let jump_not_truthy_pos = self.emit(Opcode::OpJumpNotTruthy, vec![9999]);
                self.compile(Node::Statement(Statement::BlockStatement(consequence)))?;

                if self.last_instruction_is(Opcode::OpPop) {
                    self.remove_last_pop();
                }

                let jump_pos = self.emit(Opcode::OpJump, vec![9999]);

                let after_consequence_pos = self.current_instructions().0.len();
                self.change_operand(jump_not_truthy_pos, after_consequence_pos as i64);

                if let Some(a) = alternative {
                    self.compile(Node::Statement(Statement::BlockStatement(a)))?;
                    if self.last_instruction_is(Opcode::OpPop) {
                        self.remove_last_pop();
                    }
                } else {
                    self.emit(Opcode::OpNull, Vec::new());
                }
                let after_alternative_pos = self.current_instructions().0.len();
                self.change_operand(jump_pos, after_alternative_pos as i64);
            }
            Node::Statement(Statement::BlockStatement(BlockStatement {
                token: _,
                statements,
            })) => {
                for s in statements.iter() {
                    self.compile(Node::Statement(s.clone()))?;
                }
            }
            Node::Statement(Statement::LetStatement(LetStatement {
                token: _,
                name,
                value,
            })) => {
                let symbol = self.symbol_table.borrow_mut().define(&name.value);
                self.compile(Node::Expression(value))?;
                if symbol.scope == SymbolScope::GlobalScope {
                    self.emit(Opcode::OpSetGlobal, vec![symbol.index]);
                } else {
                    self.emit(Opcode::OpSetLocal, vec![symbol.index]);
                }
            }
            Node::Expression(Expression::Identifier(Identifier { token: _, value })) => {
                let s = self.symbol_table.borrow_mut().resolve(&value);
                if let Some(symbol) = s {
                    self.load_symbol(&symbol);
                } else {
                    return Err(format!("undefined variable {}", value));
                };
            }
            Node::Expression(Expression::StringLiteral(StringLiteral { token: _, value })) => {
                let s = Object::StringObj(StringObj {
                    value: value.to_owned(),
                });
                let index = self.add_constant(s);
                self.emit(Opcode::OpConstant, vec![index]);
            }
            Node::Expression(Expression::ArrayLiteral(ArrayLiteral { token: _, elements })) => {
                let len = elements.len() as i64;
                for el in elements {
                    self.compile(Node::Expression(el))?;
                }
                self.emit(Opcode::OpArray, vec![len]);
            }
            Node::Expression(Expression::HashLiteral(HashLiteral { token: _, pairs })) => {
                let mut keys: Vec<Expression> =
                    pairs.keys().into_iter().map(|x| x.clone()).collect();

                keys.sort_by_key(|x| x.string());

                for k in keys.iter() {
                    let v = &pairs[k];
                    self.compile(Node::Expression(k.clone()))?;
                    self.compile(Node::Expression(v.clone()))?;
                }

                self.emit(Opcode::OpHash, vec![(pairs.len() * 2) as i64]);
            }
            Node::Expression(Expression::IndexExpression(IndexExpression {
                token: _,
                left,
                index,
            })) => {
                self.compile(Node::Expression(*left))?;
                self.compile(Node::Expression(*index))?;

                self.emit(Opcode::OpIndex, Vec::new());
            }
            Node::Expression(Expression::FunctionLiteral(FunctionLiteral {
                token: _,
                parameters,
                body,
            })) => {
                self.enter_scope();

                for p in parameters.iter() {
                    self.symbol_table.borrow_mut().define(&p.value);
                }

                self.compile(Node::Statement(Statement::BlockStatement(body)))?;

                if self.last_instruction_is(Opcode::OpPop) {
                    self.replace_last_pop_with_return();
                }
                if !self.last_instruction_is(Opcode::OpReturnValue) {
                    self.emit(Opcode::OpReturn, Vec::new());
                }

                let free_symbols = self.symbol_table.borrow().free_symbols.clone();
                let num_locals = self.symbol_table.borrow().num_definitions;
                let instructions = self.leave_scope();

                for s in free_symbols.iter() {
                    self.load_symbol(s);
                }

                let compiled_fn = CompiledFunction {
                    instructions: instructions,
                    num_locals: num_locals,
                    num_parameters: parameters.len(),
                };
                let fn_index = self.add_constant(Object::CompiledFunction(compiled_fn));
                self.emit(Opcode::OpClosure, vec![fn_index, free_symbols.len() as i64]);
            }
            Node::Statement(Statement::ReturnStatement(ReturnStatement {
                token: _,
                return_value,
            })) => {
                self.compile(Node::Expression(return_value))?;
                self.emit(Opcode::OpReturnValue, Vec::new());
            }
            Node::Expression(Expression::CallExpression(CallExpression {
                token: _,
                function,
                arguments,
            })) => {
                self.compile(Node::Expression(*function))?;
                let len = arguments.len();
                for a in arguments {
                    self.compile(Node::Expression(a))?;
                }
                self.emit(Opcode::OpCall, vec![len as i64]);
            }
        }
        Ok(())
    }

    pub fn bytecode(&mut self) -> Bytecode {
        Bytecode {
            instuctions: self.current_instructions().clone(),
            constants: Rc::clone(&self.constants),
        }
    }

    fn add_constant(&mut self, obj: Object) -> i64 {
        self.constants.borrow_mut().push(obj);
        self.constants.borrow().len() as i64 - 1
    }

    pub fn emit(&mut self, op: Opcode, operands: Vec<i64>) -> usize {
        let ins = make(op, &operands);
        let pos = self.add_instruction(ins.0);

        self.set_last_instruction(op, pos);
        pos
    }

    fn add_instruction(&mut self, ins: Vec<u8>) -> usize {
        let pos_new_instruction = self.current_instructions().0.len();
        let mut updated_instruction = Instructions::new();
        updated_instruction
            .0
            .extend_from_slice(&self.current_instructions().0[..]);
        updated_instruction.0.extend_from_slice(&ins[..]);

        self.scopes[self.scope_index].instructions = updated_instruction;
        pos_new_instruction
    }

    fn set_last_instruction(&mut self, op: Opcode, pos: usize) {
        let previous = self.scopes[self.scope_index].last_instruction.clone();
        let last = EmittedInstruction {
            opcode: op,
            position: pos,
        };

        self.scopes[self.scope_index].previous_instruction = previous;
        self.scopes[self.scope_index].last_instruction = Some(last);
    }

    // fn last_instruction_is_pop(&self) -> bool {
    //     if let Some(EmittedInstruction {
    //         opcode,
    //         position: _,
    //     }) = self.scopes[self.scope_index].last_instruction
    //     {
    //         if opcode == Opcode::OpPop {
    //             return true;
    //         }
    //     }
    //     false
    // }

    fn remove_last_pop(&mut self) {
        let last = self.scopes[self.scope_index].last_instruction.clone();
        let previous = self.scopes[self.scope_index].previous_instruction.clone();

        let old_ins = self.current_instructions();
        let mut new_ins = Instructions::new();
        new_ins
            .0
            .extend_from_slice(&old_ins.0[..(last.unwrap().position)]); // TODO: can unwrap?
        self.scopes[self.scope_index].instructions = new_ins;
        self.scopes[self.scope_index].last_instruction = previous;
    }

    fn replace_instruction(&mut self, pos: usize, new_instruction: Instructions) {
        let ins = self.current_instructions();
        let length = new_instruction.0.len();
        ins.0[pos..(pos + length)].copy_from_slice(&new_instruction.0)
    }

    fn change_operand(&mut self, op_pos: usize, operand: i64) {
        let op = Opcode::from(self.current_instructions().0[op_pos]);
        let new_instruction = make(op, &vec![operand]);

        self.replace_instruction(op_pos, new_instruction);
    }

    pub fn new_with_state(
        s: Rc<RefCell<SymbolTable>>,
        constants: Rc<RefCell<Vec<Object>>>,
    ) -> Compiler {
        let main_scope = CompilationScope {
            instructions: Instructions::new(),
            last_instruction: None,
            previous_instruction: None,
        };
        Compiler {
            constants: Rc::clone(&constants),
            symbol_table: Rc::clone(&s),
            scopes: vec![main_scope],
            scope_index: 0,
        }
    }

    fn current_instructions(&mut self) -> &mut Instructions {
        &mut self.scopes[self.scope_index].instructions
    }

    pub fn enter_scope(&mut self) {
        let scope = CompilationScope {
            instructions: Instructions::new(),
            last_instruction: None,
            previous_instruction: None,
        };
        self.scopes.push(scope);
        self.scope_index += 1;

        self.symbol_table = Rc::new(RefCell::new(SymbolTable::new_enclosed_symbol_table(
            Rc::clone(&self.symbol_table),
        )));
    }

    pub fn leave_scope(&mut self) -> Instructions {
        let scope = self.scopes.pop();
        self.scope_index -= 1;

        let outer = self.symbol_table.borrow().outer.clone();
        self.symbol_table = Rc::clone(&outer.unwrap()); // TODO: can unwrap?

        scope.unwrap().instructions //TODO: can unwrap?
    }

    fn last_instruction_is(&mut self, op: Opcode) -> bool {
        if self.current_instructions().0.len() == 0 {
            return false;
        }
        self.scopes[self.scope_index]
            .last_instruction
            .clone()
            .unwrap()
            .opcode
            == op //TODO: can unwrap?
    }

    fn replace_last_pop_with_return(&mut self) {
        let mut last = self.scopes[self.scope_index]
            .last_instruction
            .clone()
            .unwrap();
        self.replace_instruction(last.position, make(Opcode::OpReturnValue, &Vec::new()));

        last.opcode = Opcode::OpReturnValue;
        self.scopes[self.scope_index].last_instruction = Some(last);
    }

    fn load_symbol(&mut self, s: &Symbol) {
        match s.scope {
            SymbolScope::GlobalScope => {
                self.emit(Opcode::OpGetGlobal, vec![s.index as i64]);
            }
            SymbolScope::LocalScope => {
                self.emit(Opcode::OpGetLocal, vec![s.index as i64]);
            }
            SymbolScope::BuiltinScope => {
                self.emit(Opcode::OpGetBuiltin, vec![s.index as i64]);
            }
            SymbolScope::FreeScope => {
                self.emit(Opcode::OpGetFree, vec![s.index as i64]);
            }
        }
    }
}

pub struct Bytecode {
    pub instuctions: Instructions,
    pub constants: Rc<RefCell<Vec<Object>>>,
}

#[derive(Clone, Debug)]
pub struct EmittedInstruction {
    pub opcode: Opcode,
    pub position: usize,
}

#[derive(Debug)]
pub struct CompilationScope {
    pub instructions: Instructions,
    pub last_instruction: Option<EmittedInstruction>,
    pub previous_instruction: Option<EmittedInstruction>,
}
