// src/buildins.rs

use crate::evaluator::*;
use crate::object::*;

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                return match &args[0] {
                    Object::Array(Array { elements }) => Ok(Object::Integer(Integer {
                        value: elements.len() as i64,
                    })),
                    Object::StringObj(StringObj { value }) => Ok(Object::Integer(Integer {
                        value: value.len() as i64,
                    })),
                    _ => Err(format!(
                        "argument to `len` not supported, got {}",
                        args[0].get_type()
                    )),
                };
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "first" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    if elements.len() > 0 {
                        return Ok(elements[0].clone());
                    }
                    return Ok(Object::Null(NULL));
                } else {
                    return Err(format!(
                        "arguemnt to `first` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "last" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        return Ok(elements[length - 1].clone());
                    }
                    return Ok(Object::Null(NULL));
                } else {
                    return Err(format!(
                        "arguemnt to `last` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "rest" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        let mut new_vec: Vec<Object> = vec![Object::Null(NULL); length - 1];
                        new_vec.clone_from_slice(&elements[1..length]);
                        return Ok(Object::Array(Array { elements: new_vec }));
                    }
                    return Ok(Object::Null(NULL));
                } else {
                    return Err(format!(
                        "arguemnt to `rest` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "push" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 2 {
                    return Err(format!(
                        "wrong number of arguments. got={}, want=2",
                        args.len()
                    ));
                }
                if let Object::Array(Array { elements }) = &args[0] {
                    let mut new_elements = elements.to_vec();
                    new_elements.push(args[1].clone());
                    return Ok(Object::Array(Array {
                        elements: new_elements,
                    }));
                } else {
                    return Err(format!(
                        "arguemnt to `push` must be ARRAY, got={:?}",
                        args[0].get_type()
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "puts" => {
            let func: BuiltinFunction = |args| {
                for arg in args.iter() {
                    println!("{}", arg.inspect());
                }
                return Ok(Object::Null(NULL));
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}

pub fn get_builtin_by_name(name: &str) -> Option<Builtin> {
    if let Some(Object::Builtin(bi)) = get_builtin(name) {
        Some(bi)
    } else {
        None
    }
}

pub fn get_builtin_names() -> Vec<String> {
    return vec![
        String::from("len"),
        String::from("puts"),
        String::from("first"),
        String::from("last"),
        String::from("rest"),
        String::from("push"),
    ];
}
