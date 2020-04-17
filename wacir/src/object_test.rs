// src/object_test.rs

extern crate test;

use test::{black_box, Bencher};
use super::object::*;

#[test]
fn test_string_hash_key() {
    let hello1 = StringObj {
        value: String::from("Hello World"),
    };
    let hello2 = StringObj {
        value: String::from("Hello World"),
    };
    let diff1 = StringObj {
        value: String::from("My name is johnny"),
    };
    let diff2 = StringObj {
        value: String::from("My name is johnny"),
    };

    assert!(
        hello1.hash_key() == hello2.hash_key(),
        "strings with same content have different hash keys"
    );
    assert!(
        diff1.hash_key() == diff2.hash_key(),
        "strings with same content have different hash keys"
    );
    assert!(
        hello1.hash_key() != diff1.hash_key(),
        "strings with different content have same hash keys"
    );
}
