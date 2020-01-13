mod logic;
mod math;

use std::collections::HashMap;

use crate::runtime::objects::{KodyObject, KodyValue};

// GLOBALS contains all globally available functions
lazy_static! {
    pub static ref GLOBALS: HashMap<&'static str, KodyObject> = {
        [
            // the as fn(..) -> Result<..> is there to stop an error
            (
                "print",
                __print as fn(Vec<KodyObject>) -> Result<KodyObject, String>,
            ),
            ("__equal", math::__equal),
            ("__not_equal", math::__not_equal),
            ("__less_than", math::__less_than),
            ("__less_than_or_equal", math::__less_or_equal),
            ("__greater_than", math::__greater_than),
            ("__greater_than_or_equal", math::__greater_or_equal),
            ("__add", math::__add),
            ("__subtract", math::__subtract),
            ("__multiply", math::__multiply),
            ("__divide", math::__divide),
            ("__negate", math::__negate),
            ("__not", logic::__not),
            ("__and", logic::__and),
            ("__or", logic::__or),
        ]
        .iter()
        .map(|(name, func)| (*name, KodyObject::from(KodyValue::NativeFunction(*func))))
        .collect()
    };
}

fn __print(args: Vec<KodyObject>) -> Result<KodyObject, String> {
    for arg in args {
        match *arg.value {
            KodyValue::StringLiteral(val) => print!("{}", val),
            KodyValue::Number(val) => print!("{}", val),
            KodyValue::Bool(val) => print!("{}", val),
            _ => print!("{:?}", arg.value),
        }
    }
    println!();
    Ok(KodyObject::new())
}
