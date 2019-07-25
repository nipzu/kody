mod tokenizer;

use std::env::args_os;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use tokenizer::tokenize;

pub fn run() -> Result<(), String> {
    let arguments = parse_args()?;

    let mut filedata = String::new();

    {
        let mut file = File::open(&arguments.source_file).map_err(|_err| {
            String::from(format!(
                "Unable to read the contents of file {} !",
                arguments.source_file.display()
            ))
        })?;

        file.read_to_string(&mut filedata).map_err(|_err| {
            String::from(format!(
                "The data in file {} was not valid UTF-8 text!",
                arguments.source_file.display()
            ))
        })?;
    }

    if arguments.is_verbose {
        println!("{}", filedata);
    }

    let tokens = tokenize(&filedata)?;

    if arguments.is_verbose {
        println!("{:?}", tokens);
    }
    Ok(())
}

pub fn handle_error(value: String) {
    println!("ERROR: {:?}", value);
}

struct Arguments {
    source_file: PathBuf,
    is_verbose: bool,
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

    let mut is_verbose = false;
    if options
        .iter()
        .any(|string| string == "--verbose" || string == "-v")
    {
        is_verbose = true;
    }

    Ok(Arguments {
        source_file,
        is_verbose,
    })
}
