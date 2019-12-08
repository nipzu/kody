use std::collections::HashMap;

use crate::runtime::objects::{KodyObject, KodyValue};

lazy_static! {
    pub static ref GLOBALS: HashMap<&'static str, KodyObject> = {
        let mut globals = HashMap::new();
        globals.insert(
            "print",
            KodyObject::from(KodyValue::NativeFunction(__print)),
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
            KodyValue::Number(val) => print!("{}", val),
            KodyValue::Bool(val) => print!("{}", val),
            _ => print!("{:?}", arg.value),
        }
    }
    print!("\n");
    Ok(KodyObject::new())
}
