use clap::{crate_description, App, Arg};
use hashcode2020::greedy::Greedy;
use hashcode2020::{score, Input, Output, Solver};
use std::fs::{read_to_string, write};
use std::process::exit;

fn main() {
    let args = App::new(crate_description!())
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Sets the output file to use")
                .index(2),
        )
        .get_matches();

    println!(crate_description!());
    let input = read_input(args.value_of("INPUT").unwrap());
    let solver = Greedy::default();
    let output = solver.solve(&input);
    match score(&input, &output) {
        Ok(value) => println!("Score: {}", value),
        Err(err) => println!("Invalid output: {}", err),
    }
    if let Some(output_filename) = args.value_of("OUTPUT") {
        write_output(output_filename, &output);
    }
}

fn read_input(filename: &str) -> Input {
    let input = read_to_string(filename).unwrap_or_else(|err| {
        println!("Failed to read file '{}': {}", filename, err.to_string());
        exit(2);
    });
    input.parse().unwrap_or_else(|err: String| {
        println!("Failed to parse input: {}", err.to_string());
        exit(3);
    })
}

fn write_output(filename: &str, output: &Output) {
    write(filename, output.to_string()).expect("Unable to write file");
}
