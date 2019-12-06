mod runtime;
mod syntax_tree;
mod tokenizer;

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

use syntax_tree::{parse_tokens, KodySyntaxTree};
use tokenizer::tokenize;
use runtime::execute;

pub fn run(arguments: &Arguments) -> Result<(), String> {
    let start_time = Instant::now();

    if let SourceType::File(path) = &arguments.source {
        if path.extension() != Some(OsStr::new("kd")) && !arguments.ignore_extensions {
            return Err(String::from(
            "Incorrect source file extension. Use .kd extension or the --ignore-extensions flag.",
        ));
        }
    }

    let source_data = match &arguments.source {
        SourceType::File(path) => get_file_contents(&path)?,
        SourceType::Text(data) => data.clone(),
    };

    let tree = parse_file(source_data, &arguments)?;

    execute(&tree)?;

    let end_time = Instant::now();

    println!("Time elapsed: {} µs", (end_time - start_time).as_micros());

    Ok(())
}

pub struct Arguments {
    pub source: SourceType,
    pub is_verbose: bool,
    pub ignore_extensions: bool,
}

pub enum SourceType {
    File(PathBuf),
    Text(String),
}

fn get_file_contents(filename: &PathBuf) -> Result<String, String> {
    let mut file = File::open(filename).map_err(|_err| {
        format!(
            "Unable to read the contents of file {} !",
            filename.display()
        )
    })?;

    let mut filedata = String::new();
    file.read_to_string(&mut filedata).map_err(|_err| {
        format!(
            "The data in file {} was not valid UTF-8 text!",
            filename.display()
        )
    })?;

    Ok(filedata)
}

fn parse_file(filedata: String, arguments: &Arguments) -> Result<KodySyntaxTree, String> {
    if arguments.is_verbose {
        println!();
        println!("[INFO]: File contents:");
        println!("{}", filedata);
    }

    let tokens = tokenize(&filedata)?;

    if arguments.is_verbose {
        println!();
        println!("[INFO]: Tokens:");
        println!("{:#?}", tokens);
    }

    let tree = parse_tokens(&tokens[..])?;

    if arguments.is_verbose {
        println!();
        println!("[INFO]: Syntax tree:");
        println!("{:#?}", tree);
    }

    Ok(tree)
}
