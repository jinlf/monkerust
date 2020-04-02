// src/code.rs

pub type Instructions = Vec<u8>;
pub struct VecU8(Vec<u8>);
impl VecU8 {
    pub fn string(&self) -> String {
        String::new()
    }
}

pub type Opcode = u8;

pub const OP_CONSTANT: Opcode = 0;

pub struct Definition {
    pub name: String,
    pub operand_widths: Vec<usize>,
}

fn get_definition(opcode: Opcode) -> Option<Definition> {
    match opcode {
        OP_CONSTANT => Some(Definition {
            name: String::from("OpConstant"),
            operand_widths: vec![2],
        }),
        _ => None,
    }
}

pub fn lookup(op: u8) -> Result<Definition, String> {
    if let Some(def) = get_definition(op) {
        Ok(def)
    } else {
        Err(format!("opcode {} undefined", op))
    }
}

pub fn make(op: Opcode, operands: &Vec<i32>) -> Vec<u8> {
    if let Some(def) = get_definition(op) {
        let instruction_len = def.operand_widths.iter().fold(1, |acc, w| acc + w);
        let mut instruction = vec![0; instruction_len];
        instruction[0] = op;
        let mut offset = 1;
        for (i, o) in operands.iter().enumerate() {
            let width = def.operand_widths[i];
            match width {
                2 => {
                    let two = *o as u16;
                    instruction[offset..].copy_from_slice(&two.to_be_bytes());
                }
                _ => {
                    // error
                }
            }
            offset += width;
        }
        instruction
    } else {
        Vec::new()
    }
}
