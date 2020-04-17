// src/code.rs

use std::collections::*;
use std::convert::TryInto;

#[derive(Clone, PartialEq, Eq)]
pub struct Instructions(pub Vec<u8>);
impl std::fmt::Debug for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string())
    }
}
impl Instructions {
    pub fn new() -> Self {
        Instructions(Vec::new())
    }
    pub fn put_be_u16(&mut self, offset: usize, value: u16) {
        let bytes = value.to_be_bytes();
        self.0[offset..offset + 2].copy_from_slice(&bytes)
    }
    pub fn string(&self) -> String {
        let mut out = String::new();
        let mut i = 0;
        while i < self.0.len() {
            match lookup(self.0[i]) {
                Ok(def) => {
                    let (operands, read) = read_operands(&def, &self.0[i + 1..]);
                    out.push_str(&format!(
                        "{:04} {}\n",
                        i,
                        self.fmt_instruction(&def, operands)
                    ));
                    i += 1 + read;
                }
                Err(err) => {
                    out.push_str(&format!("ERROR: {}", err));
                    continue;
                }
            }
        }
        out
    }

    fn fmt_instruction(&self, def: &Definition, operands: Vec<i64>) -> String {
        let operand_count = def.operand_widths.len();
        if operands.len() != operand_count {
            return format!(
                "ERROR: operand len {} does not match defined {}\n",
                operands.len(),
                operand_count
            );
        }
        match operand_count {
            0 => return String::from(def.name),
            1 => return format!("{} {}", def.name, operands[0]),
            2 => return format!("{} {} {}", def.name, operands[0], operands[1]),
            _ => {
                return format!("ERROR: unhandled operand_count for {}\n", def.name);
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Opcode {
    OpConstant = 0,
    OpAdd,
    OpPop,
    OpSub,
    OpMul,
    OpDiv,
    OpTrue,
    OpFalse,
    OpEqual,
    OpNotEqual,
    OpGreaterThan,
    OpMinus,
    OpBang,
    OpJumpNotTruthy,
    OpJump,
    OpNull,
    OpGetGlobal,
    OpSetGlobal,
    OpArray,
    OpHash,
    OpIndex,
    OpCall,
    OpReturnValue,
    OpReturn,
    OpGetLocal,
    OpSetLocal,
    OpGetBuiltin,
    OpClosure,
    OpGetFree,
}
impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::OpConstant,
            1 => Opcode::OpAdd,
            2 => Opcode::OpPop,
            3 => Opcode::OpSub,
            4 => Opcode::OpMul,
            5 => Opcode::OpDiv,
            6 => Opcode::OpTrue,
            7 => Opcode::OpFalse,
            8 => Opcode::OpEqual,
            9 => Opcode::OpNotEqual,
            10 => Opcode::OpGreaterThan,
            11 => Opcode::OpMinus,
            12 => Opcode::OpBang,
            13 => Opcode::OpJumpNotTruthy,
            14 => Opcode::OpJump,
            15 => Opcode::OpNull,
            16 => Opcode::OpGetGlobal,
            17 => Opcode::OpSetGlobal,
            18 => Opcode::OpArray,
            19 => Opcode::OpHash,
            20 => Opcode::OpIndex,
            21 => Opcode::OpCall,
            22 => Opcode::OpReturnValue,
            23 => Opcode::OpReturn,
            24 => Opcode::OpGetLocal,
            25 => Opcode::OpSetLocal,
            26 => Opcode::OpGetBuiltin,
            27 => Opcode::OpClosure,
            28 => Opcode::OpGetFree,
            _ => panic!("invalid Opcode"),
        }
    }
}

pub struct Definition<'a> {
    pub name: &'a str,
    pub operand_widths: Vec<usize>,
}

lazy_static! {
    pub static ref DEFINITIONS: HashMap<Opcode, Definition<'static>> = {
        let mut map = HashMap::new();
        map.insert(
            Opcode::OpConstant,
            Definition {
                name: "OpConstant",
                operand_widths: vec![2],
            },
        );
        map.insert(
            Opcode::OpAdd,
            Definition {
                name: "OpAdd",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpPop,
            Definition {
                name: "OpPop",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpSub,
            Definition {
                name: "OpSub",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpMul,
            Definition {
                name: "OpMul",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpDiv,
            Definition {
                name: "OpDiv",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpTrue,
            Definition {
                name: "OpTrue",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpFalse,
            Definition {
                name: "OpFalse",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpEqual,
            Definition {
                name: "OpEqual",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpNotEqual,
            Definition {
                name: "OpNotEqual",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpGreaterThan,
            Definition {
                name: "OpGreaterThan",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpMinus,
            Definition {
                name: "OpMinus",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpBang,
            Definition {
                name: "OpBang",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpJumpNotTruthy,
            Definition {
                name: "OpJumpNotTruthy",
                operand_widths: vec![2],
            },
        );
        map.insert(
            Opcode::OpJump,
            Definition {
                name: "OpJump",
                operand_widths: vec![2],
            },
        );
        map.insert(
            Opcode::OpNull,
            Definition {
                name: "OpNull",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpGetGlobal,
            Definition {
                name: "OpGetGlobal",
                operand_widths: vec![2],
            },
        );
        map.insert(
            Opcode::OpSetGlobal,
            Definition {
                name: "OpSetGlobal",
                operand_widths: vec![2],
            },
        );
        map.insert(
            Opcode::OpArray,
            Definition {
                name: "OpArray",
                operand_widths: vec![2],
            },
        );
        map.insert(
            Opcode::OpHash,
            Definition {
                name: "OpHash",
                operand_widths: vec![2],
            },
        );
        map.insert(
            Opcode::OpIndex,
            Definition {
                name: "OpIndex",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpCall,
            Definition {
                name: "OpCall",
                operand_widths: vec![1],
            },
        );
        map.insert(
            Opcode::OpReturnValue,
            Definition {
                name: "OpReturnValue",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpReturn,
            Definition {
                name: "OpReturn",
                operand_widths: Vec::new(),
            },
        );
        map.insert(
            Opcode::OpGetLocal,
            Definition {
                name: "OpGetLocal",
                operand_widths: vec![1],
            },
        );
        map.insert(
            Opcode::OpSetLocal,
            Definition {
                name: "OpSetLocal",
                operand_widths: vec![1],
            },
        );
        map.insert(
            Opcode::OpGetBuiltin,
            Definition {
                name: "OpGetBuiltin",
                operand_widths: vec![1],
            },
        );
        map.insert(
            Opcode::OpClosure,
            Definition {
                name: "OpClosure",
                operand_widths: vec![2, 1],
            },
        );
        map.insert(
            Opcode::OpGetFree,
            Definition {
                name: "OpGetFree",
                operand_widths: vec![1],
            },
        );
        map
    };
}

pub fn lookup<'a>(op: u8) -> Result<&'a Definition<'a>, String> {
    if let Some(def) = DEFINITIONS.get(&Opcode::from(op)) {
        Ok(def)
    } else {
        Err(format!("opcode {} undefined", op))
    }
}

pub fn make(op: Opcode, operands: &Vec<i64>) -> Instructions {
    if let Some(def) = DEFINITIONS.get(&op) {
        let instruction_len = def.operand_widths.iter().fold(1, |acc, w| acc + w);
        let mut instruction = Instructions(vec![0; instruction_len]);
        instruction.0[0] = op as u8;
        let mut offset = 1;
        for (i, o) in operands.iter().enumerate() {
            let width = def.operand_widths[i];
            match width {
                2 => {
                    let two = *o as u16;
                    instruction.put_be_u16(offset, two);
                }
                1 => {
                    instruction.0[offset] = (*o) as u8;
                }
                _ => {
                    // error
                }
            }
            offset += width;
        }
        instruction
    } else {
        Instructions::new()
    }
}

pub fn read_operands(def: &Definition, ins: &[u8]) -> (Vec<i64>, usize) {
    let mut operands = vec![0; def.operand_widths.len()];
    let mut offset = 0;

    for (i, width) in def.operand_widths.iter().enumerate() {
        match width {
            2 => {
                let src = ins[offset..offset + 2].try_into().expect("wrong size");
                operands[i] = read_u16(src) as i64
            }
            1 => {
                operands[i] = ins[offset] as i64;
            }
            _ => {
                // error
            }
        }
        offset += width;
    }
    (operands, offset)
}
pub fn read_u16(ins: [u8; 2]) -> u16 {
    let mut bytes: [u8; 2] = Default::default();
    bytes.copy_from_slice(&ins);
    u16::from_be_bytes(bytes)
}
