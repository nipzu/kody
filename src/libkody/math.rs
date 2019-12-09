use crate::runtime::objects::{KodyObject, KodyValue, KodyNumber};

pub fn __less_than(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    if args.len() != 2 {
        return Err(String::from(
            "Attempting to compare other then two objects!",
        ));
    }
    match (*args[0].value.clone(), *args[1].value.clone()) {
        (KodyValue::Number(KodyNumber { value: val1 }), KodyValue::Number(KodyNumber { value: val2 })) => {
            if val1 < val2 {
                return Ok(KodyObject{value: Box::new(KodyValue::Bool(true))});
            } else {
                return Ok(KodyObject{value: Box::new(KodyValue::Bool(false))});
            }
        }
        _ => {
            return Err(String::from(
                "Cannot compare two objects other than numbers!",
            ))
        }
    }
}
