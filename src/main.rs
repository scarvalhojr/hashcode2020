use clap::{crate_description, value_t, values_t, App, Arg, ArgGroup};
use hashcode2020::planner::{PlanBuilder, ScanningPlan, SignupExponent};
use hashcode2020::ScanningTask;
use num_format::{Locale, ToFormattedString};
use std::fs::{read_to_string, write};
use std::process::exit;

fn main() {
    let (input_filename, output_filename, idle_exp, signup_exp) = get_args();
    println!(crate_description!());

    let task = read_input(&input_filename);
    let total_book_score = task.total_book_score();
    let book_copies = task.total_book_copies();
    println!(
        "Days: {}\n\
         Books: {}\n\
         Book scores: {} - {} ({:0.2} average)\n\
         Max theoretical score: {}\n\
         Book copies: {} ({:0.2} average per book)\n\
         Libraries: {}\n\
         Idle exponent: {:0.4}",
        task.days.to_formatted_string(&Locale::en),
        task.books.len().to_formatted_string(&Locale::en),
        task.min_book_score().to_formatted_string(&Locale::en),
        task.max_book_score().to_formatted_string(&Locale::en),
        (total_book_score as f32 / task.books.len() as f32),
        total_book_score.to_formatted_string(&Locale::en),
        book_copies.to_formatted_string(&Locale::en),
        (book_copies as f32 / task.books.len() as f32),
        task.libraries.len().to_formatted_string(&Locale::en),
        idle_exp,
    );
    let builder = PlanBuilder::new(&task, idle_exp, signup_exp);
    let plan = builder.build();
    let (score, idle_library_count, idle_slot_count) =
        plan.score().unwrap_or_else(|err| {
            println!("Invalid output: {}", err);
            exit(4);
        });
    let scanned = plan.count_scanned_books();
    let signedup = plan.count_signedup_libraries();
    println!(
        "Books scanned: {} ({:0.1}% of total)\n\
         Libraries signed-up: {} ({:.1}% of total)\n\
         Libraries partially idle: {} ({:.1}% of signed-up)\n\
         Total idle slots: {}\n\
         Score: {} ({:.1}% of max theoretical)",
        scanned.to_formatted_string(&Locale::en),
        (100_f32 * scanned as f32 / task.books.len() as f32),
        signedup.to_formatted_string(&Locale::en),
        (100_f32 * signedup as f32 / task.libraries.len() as f32),
        idle_library_count.to_formatted_string(&Locale::en),
        (100_f32 * idle_library_count as f32 / signedup as f32),
        idle_slot_count.to_formatted_string(&Locale::en),
        score.to_formatted_string(&Locale::en),
        (100_f32 * score as f32 / total_book_score as f32),
    );
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

fn get_args() -> (String, Option<String>, f32, SignupExponent) {
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
            Arg::with_name("idle_exp")
                .value_name("idle exponent")
                .help("Idle exponent")
                .short("i")
                .long("idle-exp")
                .takes_value(true)
                .allow_hyphen_values(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("signup_exp")
                .value_name("signup exponent")
                .help("Sign-up exponent")
                .short("e")
                .long("signup-exp")
                .takes_value(true)
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::with_name("signup_exp_range")
                .value_name("start,end,step")
                .help("Best result from a range of sign-up exponents")
                .short("r")
                .long("signup-exp-range")
                .takes_value(true)
                .number_of_values(3)
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
                .number_of_values(3)
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
    let idle_exp =
        value_t!(args.value_of("idle_exp"), f32).unwrap_or_else(|e| e.exit());
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

    (input_file, output_file, idle_exp, signup_exp)
}
