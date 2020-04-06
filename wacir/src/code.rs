// src/code.rs

#[derive(Clone, Debug)]
pub struct Instructions {
    pub content: Vec<u8>,
}
impl From<Vec<u8>> for Instructions {
    fn from(v: Vec<u8>) -> Self {
        Instructions { content: v }
    }
}
impl Into<Vec<u8>> for Instructions {
    fn into(self) -> Vec<u8> {
        self.content
    }
}
impl Instructions {
    pub fn put_be_u16(&mut self, offset: usize, value: u16) {
        let bytes = value.to_be_bytes();
        self.content[offset..offset + 2].copy_from_slice(&bytes)
    }
    pub fn new() -> Self {
        Instructions {
            content: Vec::new(),
        }
    }
    pub fn string(&self) -> String {
        let mut out = String::new();
        let mut i = 0;
        while i < self.content.len() {
            match lookup(self.content[i]) {
                Ok(def) => {
                    let (operands, read) =
                        read_operands(&def, Instructions::from(self.content[i + 1..].to_vec()));
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

    fn fmt_instruction(&self, def: &Definition, operands: Vec<isize>) -> String {
        let operand_cound = def.operand_widths.len();
        if operands.len() != operand_cound {
            return format!(
                "ERROR: operand len {} does not match defined {}\n",
                operands.len(),
                operand_cound
            );
        }
        println!("operand_cound:{}", operand_cound);
        match operand_cound {
            0 => return def.name.clone(),
            1 => return format!("{} {}", def.name, operands[0]),            
            _ => {
                return format!("ERROR: unhandled operand_count for {}\n", def.name);
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum Opcode {
    OpConstant,
    OpAdd,
    OpUnknown,
}
impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::OpConstant,
            1 => Opcode::OpAdd,
            _ => Opcode::OpUnknown,
        }
    }
}
impl Into<u8> for Opcode {
    fn into(self) -> u8 {
        match self {
            Opcode::OpConstant => 0,
            Opcode::OpAdd => 1,
            _ => std::u8::MAX,
        }
    }
}

pub struct Definition {
    pub name: String,
    pub operand_widths: Vec<usize>,
}

fn get_definition(opcode: Opcode) -> Option<Definition> {
    match opcode {
        Opcode::OpConstant => Some(Definition {
            name: String::from("OpConstant"),
            operand_widths: vec![2],
        }),
        Opcode::OpAdd => Some(Definition{
            name:String::from("OpAdd"),
            operand_widths: Vec::new(),
        }),
        _ => None,
    }
}

pub fn lookup(op: u8) -> Result<Definition, String> {
    if let Some(def) = get_definition(Opcode::from(op)) {
        Ok(def)
    } else {
        Err(format!("opcode {} undefined", op))
    }
}

pub fn make(op: Opcode, operands: &Vec<isize>) -> Vec<u8> {
    if let Some(def) = get_definition(op) {
        let instruction_len = def.operand_widths.iter().fold(1, |acc, w| acc + w);
        let mut instruction = Instructions::from(vec![0; instruction_len]);
        instruction.content[0] = op.into();
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
        instruction.into()
    } else {
        Vec::new()
    }
}

pub fn read_operands(def: &Definition, ins: Instructions) -> (Vec<isize>, usize) {
    let mut operands = vec![0; def.operand_widths.len()];
    let mut offset = 0;

    for (i, width) in def.operand_widths.iter().enumerate() {
        match width {
            2 => {
                operands[i] = read_u16(&ins.content[offset..offset+2]) as isize
            }
            _ => {
                // error
            }
        }
        offset += width;
    }
    (operands, offset)
}
pub fn read_u16(ins: &[u8]) -> u16 {
    let mut bytes: [u8; 2] = Default::default();
    bytes.copy_from_slice(ins);
    u16::from_be_bytes(bytes)
}