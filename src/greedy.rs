use super::{Input, Output, Solver};
use std::collections::HashSet;

#[derive(Default)]
pub struct Greedy {}

impl Solver for Greedy {
    fn solve(&self, input: &Input) -> Output {
        let mut output = Output::default();

        let mut available_books = input
            .libraries
            .iter()
            .flat_map(|library| library.books.clone())
            .collect::<HashSet<_>>()
            .iter()
            .map(|&book_id| input.scores.get(book_id).map(|&score| (book_id, score)))
            .collect::<Option<Vec<(usize, usize)>>>()
            .expect("Invalid book ID in library");
        println!(
            "{} books available, {} total in input",
            available_books.len(),
            input.scores.len(),
        );

        available_books.sort_unstable_by(|(_, score_a), (_, score_b)| {
            score_a.cmp(score_b).reverse()
        });
        let mut sorted_books = available_books
            .iter()
            .map(|(book_id, _)| *book_id)
            .collect::<Vec<_>>();
        // println!("Book IDs sorted by score: {:?}", sorted_books);

        let mut libraries = input
            .libraries
            .iter()
            .enumerate()
            .map(|(lib_id, library)| {
                (
                    lib_id,
                    if library.signup < input.days {
                        (input.days - library.signup) * library.scanrate
                    } else {
                        0
                    },
                )
            })
            .collect::<Vec<_>>();
        libraries.sort_unstable_by_key(|(_, max_scans)| *max_scans);
        libraries.reverse();

        let mut min_days = 0;
        let mut sorted_libraries = Vec::new();
        for (library_id, _) in &libraries {
            let library = &input.libraries[*library_id];
            min_days += library.signup;
            sorted_libraries.push((min_days, library_id));
            output.add_library(*library_id);
            // println!("Library {} can scan after {} days", library_id, min_days);
        }

        let mut active_libraries = 0;
        let mut scan_capacity = 0;
        let mut new_scans = false;
        for day in 0..input.days {
            println!("Day {}/{}", day, input.days);
            if sorted_books.is_empty() {
                break;
            }

            // Sign up new library if possible
            let mut new_library = false;
            if active_libraries < sorted_libraries.len() {
                let (min_days, library_id) = sorted_libraries[active_libraries];
                if day >= min_days {
                    active_libraries += 1;
                    scan_capacity += input.libraries[*library_id].scanrate;
                    new_library = true;
                    // println!("Library {} is now active", library_id);
                }
            }

            if !new_library && !new_scans {
                continue;
            }

            // Reset daily capacity of active libraries
            let mut daily_capacity = sorted_libraries
                .iter()
                .map(|&(_, lib_id)| input.libraries[*lib_id].scanrate)
                .collect::<Vec<_>>();

            println!(
                " => {}/{} active libraries, {}/{} books available",
                active_libraries,
                input.libraries.len(),
                sorted_books.len(),
                input.scores.len(),
            );

            let mut next_book = 0;
            let mut day_capacity = scan_capacity;

            new_scans = false;
            while day_capacity > 0 && next_book < sorted_books.len() {
                let book_id = sorted_books[next_book];
                let mut scanned = false;
                for lib_index in 0..active_libraries {
                    let (_, &library_id) = sorted_libraries[lib_index];
                    if daily_capacity[lib_index] == 0 {
                        continue;
                    }
                    if input.libraries[library_id].books.contains(&book_id) {
                        // println!(
                        //     "Library {} scans book {}",
                        //     library_id, book_id
                        // );
                        output.add_scan(library_id, book_id);
                        daily_capacity[lib_index] -= 1;
                        day_capacity -= 1;
                        scanned = true;
                        new_scans = true;
                        break;
                    }
                }
                if scanned {
                    sorted_books.remove(next_book);
                } else {
                    next_book += 1;
                }
            }
        }

        // Remove libraries that have not done any scanning
        println!("{} libraries signed up before purging",
            output.library_ids.len());
        output.purge_idle();
        println!("{} libraries signed up after purging",
            output.library_ids.len());

        output
    }
}
