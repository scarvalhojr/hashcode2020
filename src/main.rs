use clap::{crate_description, value_t, values_t, App, Arg, ArgGroup};
use hashcode2020::planner::{PlanBuilder, ScanningPlan, SignupExponent};
use hashcode2020::ScanningTask;
use std::fs::{read_to_string, write};
use std::process::exit;

fn main() {
    let (input_filename, output_filename, signup_exp) = get_args();
    println!(crate_description!());

    let task = read_input(&input_filename);
    let builder = PlanBuilder::new(&task, signup_exp);
    let plan = builder.build();
    match plan.score() {
        Ok(value) => println!("Score: {}", value),
        Err(err) => println!("Invalid output: {}", err),
    }
    if let Some(filename) = output_filename {
        write_output(&filename, &plan);
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

fn get_args() -> (String, Option<String>, SignupExponent) {
    let args = App::new(crate_description!())
        .arg(
            Arg::with_name("input")
                .value_name("input file")
                .help("Path to input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .value_name("output file")
                .help("Path to output file")
                .short("o")
                .long("output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("signup_exp")
                .value_name("signup exponent")
                .help("Sign-up exponent")
                .short("e")
                .long("signup-exp")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("signup_exp_range")
                .value_name("start,end,step")
                .help("Best result from a range of sign-up exponents")
                .short("r")
                .long("signup-exp-range")
                .takes_value(true)
                .multiple(true)
                .min_values(3)
                .max_values(3)
                .require_delimiter(true),
        )
        .arg(
            Arg::with_name("variable_signup_exp")
                .value_name("count,min,max")
                .help(
                    "Best result from multiple runs using variable random \
                     sign-up exponents at every step",
                )
                .short("v")
                .long("variable-signup-exp")
                .takes_value(true)
                .multiple(true)
                .min_values(3)
                .max_values(3)
                .require_delimiter(true),
        )
        .group(
            ArgGroup::with_name("mode")
                .args(&[
                    "signup_exp",
                    "signup_exp_range",
                    "variable_signup_exp",
                ])
                .required(false),
        )
        .get_matches();

    let input_file = args.value_of("input").unwrap().to_string();
    let output_file = args.value_of("output").map(str::to_string);
    let signup_exp = if args.is_present("signup_exp_range") {
        let values = values_t!(args.values_of("signup_exp_range"), f32)
            .unwrap_or_else(|e| e.exit());
        SignupExponent::Range(values[0], values[1], values[2])
    } else if args.is_present("variable_signup_exp") {
        let values = values_t!(args.values_of("variable_signup_exp"), f32)
            .unwrap_or_else(|e| e.exit());
        SignupExponent::Variable(values[0] as usize, values[1], values[2])
    } else {
        let exp = if args.is_present("signup_exp") {
            value_t!(args.value_of("signup_exp"), f32)
                .unwrap_or_else(|e| e.exit())
        } else {
            1.0
        };
        SignupExponent::Fixed(exp)
    };

    (input_file, output_file, signup_exp)
}
