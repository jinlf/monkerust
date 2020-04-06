// src/code_test.rs

use super::code::*;

#[test]
fn test_make() {
    let tests = [(
        Opcode::OpConstant,
        vec![65534],
        vec![Opcode::OpConstant.into(), 255, 254],
    ),
    (
        Opcode::OpAdd,
        Vec::new(),
        vec![Opcode::OpAdd.into()],
    )];

    for tt in tests.iter() {
        let instruction = make(tt.0, &tt.1);
        assert!(
            instruction.len() == tt.2.len(),
            "instruction has wrong length. want={}, got={}",
            tt.2.len(),
            instruction.len()
        );
        for (i, b) in tt.2.iter().enumerate() {
            assert!(
                instruction[i] == tt.2[i],
                "wrong byte at pos {}. want={}, got={}",
                i,
                b,
                instruction[i]
            );
        }
    }
}

#[test]
fn test_instructions_string() {
    let instructions = vec![
        make(Opcode::OpAdd, &Vec::new()),
        make(Opcode::OpConstant, &vec![2]),
        make(Opcode::OpConstant, &vec![65535]),
    ];

    let expected = "0000 OpAdd
0001 OpConstant 2
0004 OpConstant 65535
";

    let mut concatted = Instructions::new();
    for ins in instructions {
        concatted.content.extend_from_slice(&ins);
    }

    assert!(
        concatted.string() == expected,
        "instructions wrongly formatted.\nwant={:?}\ngot={:?}",
        expected,
        concatted.string()
    );
}

#[test]
fn test_read_operands() {
    let tests = [(Opcode::OpConstant, vec![65535], 2)];

    for tt in tests.iter() {
        let instruction = make(tt.0, &tt.1);
        match lookup(tt.0.into()) {
            Ok(def) => {
                let (operands_read, n) =
                    read_operands(&def, Instructions::from(instruction[1..].to_vec()));
                assert!(n == tt.2, "n wrong. want={}, got={}", tt.2, n);

                for (i, want) in tt.1.iter().enumerate() {
                    assert!(
                        operands_read[i] == *want,
                        "operand wong. want={}, got{}",
                        want,
                        operands_read[i]
                    );
                }
            }
            Err(e) => {
                assert!(false, "definition not found: {:?}", e);
            }
        }
    }
}
