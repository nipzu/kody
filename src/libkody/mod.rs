mod math;

use std::collections::HashMap;

use crate::runtime::objects::{KodyObject, KodyValue};

lazy_static! {
    pub static ref GLOBALS: HashMap<&'static str, KodyObject> = {
        let mut globals = HashMap::new();
        globals.insert(
            "print",
            KodyObject::from(KodyValue::NativeFunction(__print)),
        );
        globals.insert(
            "__less_than",
            KodyObject::from(KodyValue::NativeFunction(math::__less_than)),
        );
        globals
    };
}

fn __print(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    let mut gap = false;
    for arg in args {
        if gap {
            print!(" ");
        } else {
            gap = true;
        }
        match *arg.value {
            KodyValue::StringLiteral(val) => print!("{}", val),
            KodyValue::Number(obj) => print!("{}", obj.value),
            KodyValue::Bool(val) => print!("{}", val),
            _ => print!("{:?}", arg.value),
        }
    }
    println!();
    Ok(KodyObject::new())
}
