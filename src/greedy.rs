use super::{Input, Output, Solver};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Greedy {}

impl Solver for Greedy {
    fn solve(&self, input: &Input) -> Output {
        let mut output = Output::default();

        let mut pending_libraries =
            (0..input.libraries.len()).collect::<HashSet<_>>();
        let mut library_books = input
            .libraries
            .iter()
            .enumerate()
            .map(|(library_id, library)| {
                let mut book_scores = library
                    .books
                    .iter()
                    .map(|book_id| (*book_id, input.scores[*book_id]))
                    .collect::<Vec<_>>();
                book_scores.sort_unstable_by_key(|(_, score)| *score);
                (library_id, book_scores)
            })
            .collect::<HashMap<_, _>>();

        let mut signup_days = 0;
        let mut next_active_library_id = None;
        let mut active_libraries: HashSet<usize> = HashSet::new();
        let mut scanned_books: HashSet<usize> = HashSet::new();
        for day in 0..input.days {
            println!(
                "Day {}/{} => {} active/{} pending/ {} total libraries, {}/{} books scanned",
                day,
                input.days,
                active_libraries.len(),
                pending_libraries.len(),
                input.libraries.len(),
                scanned_books.len(),
                input.scores.len(),
            );

            if signup_days > 0 {
                signup_days -= 1;
            } else {
                // Activate next library
                if let Some(library_id) = next_active_library_id {
                    pending_libraries.remove(&library_id);
                    active_libraries.insert(library_id);
                    output.add_library(library_id);
                }

                // Evaluate libraries available for sign up
                let days_left = input.days - day;
                let mut library_scores: Vec<(usize, usize)> = Vec::new();
                for library_id in pending_libraries.iter() {
                    let library = &input.libraries[*library_id];
                    if library.signup >= days_left {
                        continue;
                    }
                    let maxscans =
                        (days_left - library.signup) * library.scanrate;

                    // Compute the maximum sum of book scores this library can
                    // add, while removing books that have already been scanned
                    let books = library_books.get_mut(library_id).unwrap();
                    books.retain(|(book_id, _)| {
                        !scanned_books.contains(book_id)
                    });

                    let lib_score = books
                        .iter()
                        .rev()
                        .take(maxscans)
                        .map(|(_, book_score)| *book_score)
                        .sum();

                    if lib_score > 0 {
                        library_scores.push((*library_id, lib_score));
                    }
                }

                // Pick next library to sign up
                next_active_library_id = library_scores
                    .iter()
                    .max_by_key(|(_, lib_score)| *lib_score)
                    .map(|(library_id, _)| *library_id);

                // Update pending libraries list
                pending_libraries = library_scores
                    .iter()
                    .map(|(library_id, _)| *library_id)
                    .collect();

                // Track days left until activation
                if let Some(library_id) = next_active_library_id {
                    // TODO: can signup days be zero?
                    signup_days = input.libraries[library_id].signup - 1;
                }
            }

            active_libraries.retain(|library_id| {
                let mut lib_scans = 0;
                let mut retain = true;
                let library = &input.libraries[*library_id];
                let books = library_books.get_mut(library_id).unwrap();
                while lib_scans < library.scanrate {
                    if let Some((book_id, _)) = books.pop() {
                        if scanned_books.insert(book_id) {
                            output.add_scan(*library_id, book_id);
                            lib_scans += 1;
                        }
                    } else {
                        retain = false;
                        break;
                    }
                }
                retain
            });
        }

        // Remove libraries that have not done any scanning
        println!(
            "{} libraries signed up before purging",
            output.library_ids.len()
        );
        output.purge_idle();
        println!(
            "{} libraries signed up after purging",
            output.library_ids.len()
        );

        output
    }
}
