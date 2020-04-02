// src/code_test.rs

use super::code::*;

#[test]
fn test_make() {
    let tests = [(OP_CONSTANT, vec![65534], vec![OP_CONSTANT, 255, 254])];

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
        make(OP_CONSTANT, &vec![1]),
        make(OP_CONSTANT, &vec![2]),
        make(OP_CONSTANT, &vec![65535]),
    ];

    let expected = "0000 OP_CONSTANT 1
0003 OP_CONSTANT 2
0006 OP_CONSTANT 65535";

    let mut concatted = Instructions {
        content: Vec::new(),
    };
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
