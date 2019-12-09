use std::cmp::Ordering;

use crate::runtime::objects::{KodyNumber, KodyObject, KodyValue};

pub fn __multiply(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    if args.len() != 2 {
        return Err(String::from(
            "Attempting to multiply objects other then two objects!",
        ));
    }
    match (*args[0].value.clone(), *args[1].value.clone()) {
        (
            KodyValue::Number(KodyNumber { value: val1 }),
            KodyValue::Number(KodyNumber { value: val2 }),
        ) => Ok(KodyObject {
            value: Box::new(KodyValue::Number(KodyNumber { value: val1 * val2 })),
        }),
        _ => Err(String::from(
            "Cannot multiply two objects other than numbers!",
        )),
    }
}

pub fn __add(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    if args.len() != 2 {
        return Err(String::from(
            "Attempting to multiply objects other then two objects!",
        ));
    }
    match (*args[0].value.clone(), *args[1].value.clone()) {
        (
            KodyValue::Number(KodyNumber { value: val1 }),
            KodyValue::Number(KodyNumber { value: val2 }),
        ) => Ok(KodyObject {
            value: Box::new(KodyValue::Number(KodyNumber { value: val1 + val2 })),
        }),
        _ => Err(String::from(
            "Cannot multiply two objects other than numbers!",
        )),
    }
}

fn compare_numbers(args: Vec<KodyObject>) -> Result<Ordering, String> {
    if args.len() != 2 {
        return Err(String::from(
            "Attempting to compare objects other then two objects!",
        ));
    }
    match (*args[0].value.clone(), *args[1].value.clone()) {
        (
            KodyValue::Number(KodyNumber { value: val1 }),
            KodyValue::Number(KodyNumber { value: val2 }),
        ) => Ok(val1.partial_cmp(&val2).unwrap()),
        _ => Err(String::from(
            "Cannot compare two objects other than numbers!",
        )),
    }
}

pub fn __less_than(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => false,
            Ordering::Less => true,
            Ordering::Greater => false,
        })),
    })
}

pub fn __greater_than(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => false,
            Ordering::Less => false,
            Ordering::Greater => true,
        })),
    })
}

pub fn __less_than_or_equal(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => true,
            Ordering::Less => true,
            Ordering::Greater => false,
        })),
    })
}

pub fn __greater_than_or_equal(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => true,
            Ordering::Less => false,
            Ordering::Greater => true,
        })),
    })
}
