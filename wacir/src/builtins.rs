// src/buildins.rs

use super::evaluator::*;
use super::object::*;

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                return match &args[0] {
                    Some(Object::Array(Array { elements })) => Some(Object::Integer(Integer {
                        value: elements.len() as i64,
                    })),
                    Some(Object::StringObj(StringObj { value })) => {
                        Some(Object::Integer(Integer {
                            value: value.len() as i64,
                        }))
                    }
                    _ => new_error(format!(
                        "argument to `len` not supported, got {}",
                        get_type(&args[0])
                    )),
                };
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "first" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    if elements.len() > 0 {
                        return Some(elements[0].clone());
                    }
                    return Some(Object::Null(NULL));
                } else {
                    return new_error(format!(
                        "arguemnt to `first` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "last" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        return Some(elements[length - 1].clone());
                    }
                    return Some(Object::Null(NULL));
                } else {
                    return new_error(format!(
                        "arguemnt to `last` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "rest" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    let length = elements.len();
                    if length > 0 {
                        let mut new_vec: Vec<Object> = vec![Object::Null(NULL); length - 1];
                        new_vec.clone_from_slice(&elements[1..length]);
                        return Some(Object::Array(Array { elements: new_vec }));
                    }
                    return Some(Object::Null(NULL));
                } else {
                    return new_error(format!(
                        "arguemnt to `rest` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "push" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 2 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=2",
                        args.len()
                    ));
                }
                if let Some(Object::Array(Array { elements })) = &args[0] {
                    let mut new_elements = elements.to_vec();
                    new_elements.push(args[1].as_ref().unwrap().clone());
                    return Some(Object::Array(Array {
                        elements: new_elements,
                    }));
                } else {
                    return new_error(format!(
                        "arguemnt to `push` must be ARRAY, got={:?}",
                        get_type(&args[0])
                    ));
                }
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        "puts" => {
            let func: BuiltinFunction = |args| {
                for arg in args.iter() {
                    println!("{}", arg.as_ref().unwrap().inspect());
                }
                return Some(Object::Null(NULL));
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}
