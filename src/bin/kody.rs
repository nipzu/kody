use std::env::args_os;
use std::path::PathBuf;

use kody::{run, Arguments, SourceType};

fn main() {
    let arguments = parse_args().unwrap_or_else(|e| handle_error(e));
    if let Err(error) = run(&arguments) {
        handle_error(error);
    }
}

fn parse_args() -> Result<Arguments, String> {
    let mut args = args_os().skip(1);

    let source = match args.next() {
        Some(val) => SourceType::File(PathBuf::from(val)),
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
        source,
        is_verbose,
        ignore_extensions,
    })
}

fn handle_error(value: String) -> ! {
    println!("ERROR: {}", value);
    std::process::exit(1);
}
