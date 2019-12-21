use std::cmp::Ordering;

use crate::runtime::objects::{KodyNumber, KodyObject, KodyValue};

fn modify_numbers(
    args: Vec<KodyObject>,
    operation: fn(f64, f64) -> f64,
    operation_name: &str,
) -> Result<KodyObject, String> {
    if args.len() != 2 {
        return Err(format!(
            "Attempting to {} objects other then two objects!",
            operation_name
        ));
    }
    match (*args[0].value.clone(), *args[1].value.clone()) {
        (
            KodyValue::Number(KodyNumber { value: val1 }),
            KodyValue::Number(KodyNumber { value: val2 }),
        ) => Ok(KodyObject {
            value: Box::new(KodyValue::Number(KodyNumber {
                value: operation(val1, val2),
            })),
        }),
        _ => Err(format!(
            "Cannot {} two objects other than numbers!",
            operation_name
        )),
    }
}

pub fn __multiply(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    modify_numbers(args, |a, b| a * b, "multiply")
}

pub fn __divide(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    modify_numbers(args, |a, b| a / b, "divide")
}

pub fn __add(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    modify_numbers(args, |a, b| a + b, "add")
}

pub fn __subtract(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    modify_numbers(args, |a, b| a - b, "subtract")
}

pub fn __negate(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    if args.len() != 1 {
        return Err(String::from("Cannot negate other than one argument"));
    }
    match *args[0].value.clone() {
        KodyValue::Number(KodyNumber { value: val }) => Ok(KodyObject {
            value: Box::new(KodyValue::Number(KodyNumber { value: -val })),
        }),
        _ => Err(String::from("Cannot negate an object other than a number!")),
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

pub fn __less_or_equal(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => true,
            Ordering::Less => true,
            Ordering::Greater => false,
        })),
    })
}

pub fn __greater_or_equal(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => true,
            Ordering::Less => false,
            Ordering::Greater => true,
        })),
    })
}

pub fn __equal(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => true,
            Ordering::Less => false,
            Ordering::Greater => false,
        })),
    })
}

pub fn __not_equal(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    Ok(KodyObject {
        value: Box::new(KodyValue::Bool(match compare_numbers(args)? {
            Ordering::Equal => false,
            Ordering::Less => true,
            Ordering::Greater => true,
        })),
    })
}
