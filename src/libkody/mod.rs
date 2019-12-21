mod math;

use std::collections::HashMap;

use crate::runtime::objects::{KodyObject, KodyValue};

// This be hella hacked together
macro_rules! bind_function {
    ($x:ident, $y:expr, $z:expr) => {
        $x.insert($y, KodyObject::from(KodyValue::NativeFunction($z)))
    };
}

// GLOBALS contains all globally available functions
lazy_static! {
    pub static ref GLOBALS: HashMap<&'static str, KodyObject> = {
        let mut globals = HashMap::new();
        bind_function!(globals, "print", __print);
        bind_function!(globals, "__equal", math::__equal);
        bind_function!(globals, "__not_equal", math::__not_equal);
        bind_function!(globals, "__less_than", math::__less_than);
        bind_function!(globals, "__less_than_or_equal", math::__less_or_equal);
        bind_function!(globals, "__greater_than", math::__greater_than);
        bind_function!(globals, "__greater_than_or_equal", math::__greater_or_equal);
        bind_function!(globals, "__add", math::__add);
        bind_function!(globals, "__subtract", math::__subtract);
        bind_function!(globals, "__multiply", math::__multiply);
        bind_function!(globals, "__divide", math::__divide);
        bind_function!(globals, "__negate", math::__negate);
        // TODO boolean operations
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
