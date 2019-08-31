use std::path::PathBuf;
use std::env::args_os;

use kody::{Arguments, handle_error, run};

fn main() {
    let arguments = parse_args().unwrap_or_else(|e| handle_error(e));
    if let Err(error) = run(&arguments) {
        handle_error(error);
    }
}

fn parse_args() -> Result<Arguments, String> {
    let mut args = args_os().skip(1);

    let source_file = match args.next() {
        Some(val) => PathBuf::from(val),
        None => {
            return Err(String::from(
                "Please provide a source file as a program argument!",
            ));
        }
    };

    let options = args
        .map(|os_string| os_string.to_string_lossy().into_owned())
        .collect::<Vec<String>>();

    let is_verbose = options.iter().any(|opt| opt == "--verbose" || opt == "-v");
    let ignore_extensions = options
        .iter()
        .any(|opt| opt == "--ignore-extensions" || opt == "-e");

    Ok(Arguments {
        source_file,
        is_verbose,
        ignore_extensions,
    })
}
