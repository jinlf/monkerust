// src/code.rs

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
            _ => {
                return format!("ERROR: unhandled operand_count for {}\n", def.name);
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
            _ => panic!("invalid Opcode"),
        }
    }
}

pub struct Definition<'a> {
    pub name: &'a str,
    pub operand_widths: Vec<usize>,
}

fn get_definition<'a>(opcode: Opcode) -> Option<Definition<'a>> {
    match opcode {
        Opcode::OpConstant => Some(Definition {
            name: "OpConstant",
            operand_widths: vec![2],
        }),
        Opcode::OpAdd => Some(Definition {
            name: "OpAdd",
            operand_widths: Vec::new(),
        }),
        Opcode::OpPop => Some(Definition {
            name: "OpPop",
            operand_widths: Vec::new(),
        }),
        Opcode::OpSub => Some(Definition {
            name: "OpSub",
            operand_widths: Vec::new(),
        }),
        Opcode::OpMul => Some(Definition {
            name: "OpMul",
            operand_widths: Vec::new(),
        }),
        Opcode::OpDiv => Some(Definition {
            name: "OpDiv",
            operand_widths: Vec::new(),
        }),
        Opcode::OpTrue => Some(Definition {
            name: "OpTrue",
            operand_widths: Vec::new(),
        }),
        Opcode::OpFalse => Some(Definition {
            name: "OpFalse",
            operand_widths: Vec::new(),
        }),
        Opcode::OpEqual => Some(Definition {
            name: "OpEqual",
            operand_widths: Vec::new(),
        }),
        Opcode::OpNotEqual => Some(Definition {
            name: "OpNotEqual",
            operand_widths: Vec::new(),
        }),
        Opcode::OpGreaterThan => Some(Definition {
            name: "OpGreaterThan",
            operand_widths: Vec::new(),
        }),
        Opcode::OpMinus => Some(Definition {
            name: "OpMinus",
            operand_widths: Vec::new(),
        }),
        Opcode::OpBang => Some(Definition {
            name: "OpBang",
            operand_widths: Vec::new(),
        }),
        Opcode::OpJumpNotTruthy => Some(Definition {
            name: "OpJumpNotTruthy",
            operand_widths: vec![2],
        }),
        Opcode::OpJump => Some(Definition {
            name: "OpJump",
            operand_widths: vec![2],
        }),
        Opcode::OpNull => Some(Definition {
            name: "OpNull",
            operand_widths: Vec::new(),
        }),
        Opcode::OpGetGlobal => Some(Definition {
            name: "OpGetGlobal",
            operand_widths: vec![2],
        }),
        Opcode::OpSetGlobal => Some(Definition {
            name: "OpSetGlobal",
            operand_widths: vec![2],
        }),
        Opcode::OpArray => Some(Definition {
            name: "OpArray",
            operand_widths: vec![2],
        }),
        Opcode::OpHash => Some(Definition {
            name: "OpHash",
            operand_widths: vec![2],
        }),
        Opcode::OpIndex => Some(Definition {
            name: "OpIndex",
            operand_widths: Vec::new(),
        }),
        Opcode::OpCall => Some(Definition {
            name: "OpCall",
            operand_widths: Vec::new(),
        }),
        Opcode::OpReturnValue => Some(Definition {
            name: "OpReturnValue",
            operand_widths: Vec::new(),
        }),
        Opcode::OpReturn => Some(Definition {
            name: "OpReturn",
            operand_widths: Vec::new(),
        }),
    }
}

pub fn lookup<'a>(op: u8) -> Result<Definition<'a>, String> {
    if let Some(def) = get_definition(Opcode::from(op)) {
        Ok(def)
    } else {
        Err(format!("opcode {} undefined", op))
    }
}

pub fn make(op: Opcode, operands: &Vec<i64>) -> Instructions {
    if let Some(def) = get_definition(op) {
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
