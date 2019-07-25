use scripting_language::{handle_error, run};

fn main() {
    if let Err(error) = run() {
        handle_error(error);
    }
}
