use crate::runtime::objects::{KodyObject, KodyValue};

pub fn __not(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    if args.len() != 1 {
        return Err(String::from("Cannot apply not to more than one object!"));
    }

    match *args[0].value.clone() {
        KodyValue::Bool(val) => Ok(KodyObject {
            value: Box::new(KodyValue::Bool(!val)),
        }),

        _ => Err(String::from(
            "Cannot apply not to an object other than a boolean!",
        )),
    }
}

pub fn __and(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    if args.len() != 2 {
        return Err(String::from("Cannot apply and to other than two objects!"));
    }

    match (*args[0].value.clone(), *args[1].value.clone()) {
        (KodyValue::Bool(val1), KodyValue::Bool(val2)) => Ok(KodyObject {
            value: Box::new(KodyValue::Bool(val1 && val2)),
        }),

        _ => Err(String::from(
            "Cannot use and on objects other than booleans!",
        )),
    }
}

pub fn __or(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    if args.len() != 2 {
        return Err(String::from("Cannot apply or to other than two objects!"));
    }

    match (*args[0].value.clone(), *args[1].value.clone()) {
        (KodyValue::Bool(val1), KodyValue::Bool(val2)) => Ok(KodyObject {
            value: Box::new(KodyValue::Bool(val1 || val2)),
        }),

        _ => Err(String::from(
            "Cannot use or on objects other than booleans!",
        )),
    }
}
