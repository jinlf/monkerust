// src/code_test.rs

use super::code::*;

#[test]
fn test_make() {
    let tests = [
        (
            Opcode::OpConstant,
            vec![65534],
            vec![Opcode::OpConstant as u8, 255, 254],
        ),
        (Opcode::OpAdd, Vec::new(), vec![Opcode::OpAdd as u8]),
        (
            Opcode::OpGetLocal,
            vec![255],
            vec![Opcode::OpGetLocal as u8, 255],
        ),
        (
            Opcode::OpClosure,
            vec![65534, 255],
            vec![Opcode::OpClosure as u8, 255, 254, 255],
        ),
    ];

    for tt in tests.iter() {
        let instruction = make(tt.0, &tt.1);
        assert!(
            instruction.0.len() == tt.2.len(),
            "instruction has wrong length. want={}, got={}",
            tt.2.len(),
            instruction.0.len()
        );
        for (i, b) in tt.2.iter().enumerate() {
            assert!(
                instruction.0[i] == tt.2[i],
                "wrong byte at pos {}. want={}, got={}",
                i,
                b,
                instruction.0[i]
            );
        }
    }
}

#[test]
fn test_instructions_string() {
    let instructions = vec![
        make(Opcode::OpAdd, &Vec::new()),
        make(Opcode::OpGetLocal, &vec![1]),
        make(Opcode::OpConstant, &vec![2]),
        make(Opcode::OpConstant, &vec![65535]),
        make(Opcode::OpClosure, &vec![65535, 255]),
    ];

    let expected = "0000 OpAdd
0001 OpGetLocal 1
0003 OpConstant 2
0006 OpConstant 65535
0009 OpClosure 65535 255
";

    let mut concatted = Instructions::new();
    for ins in instructions {
        concatted.0.extend_from_slice(&ins.0);
    }

    assert!(
        concatted.string() == expected,
        "instructions wrongly formatted.\nwant={:?}\ngot={:?}",
        expected,
        concatted
    );
}

#[test]
fn test_read_operands() {
    let tests = [
        (Opcode::OpConstant, vec![65535], 2),
        (Opcode::OpGetLocal, vec![255], 1),
        (Opcode::OpClosure, vec![65535, 255], 3),
    ];

    for tt in tests.iter() {
        let instruction = make(tt.0, &tt.1);
        match lookup(tt.0 as u8) {
            Ok(def) => {
                let (operands_read, n) = read_operands(&def, &instruction.0[1..]);
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
