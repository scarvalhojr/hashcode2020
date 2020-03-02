use clap::{crate_description, App, Arg};
use hashcode2020::planner::ScanningPlan;
use hashcode2020::ScanningTask;
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
    let task = read_input(args.value_of("INPUT").unwrap());
    let mut plan = ScanningPlan::new(&task);
    plan.solve();
    match plan.score() {
        Ok(value) => println!("Score: {}", value),
        Err(err) => println!("Invalid output: {}", err),
    }
    if let Some(output_filename) = args.value_of("OUTPUT") {
        write_output(output_filename, &plan);
    }
}

fn read_input(filename: &str) -> ScanningTask {
    let input = read_to_string(filename).unwrap_or_else(|err| {
        println!("Failed to read file '{}': {}", filename, err.to_string());
        exit(2);
    });
    input.parse().unwrap_or_else(|err: String| {
        println!("Failed to parse input: {}", err.to_string());
        exit(3);
    })
}

fn write_output(filename: &str, plan: &ScanningPlan) {
    write(filename, plan.to_string()).expect("Unable to write file");
}
