use super::{Input, Output, Solver};
use std::cmp::{min, Reverse};
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
                // TODO: make sure book IDs are unique
                let mut book_scores = library
                    .books
                    .iter()
                    .map(|book_id| (*book_id, input.scores[*book_id]))
                    .collect::<Vec<_>>();
                book_scores.sort_unstable_by_key(|(_, score)| Reverse(*score));
                (library_id, book_scores)
            })
            .collect::<HashMap<_, _>>();

        let mut days_left = input.days;
        let mut total_scanned = 0;
        let mut scanned_books = HashSet::new();
        while days_left > 0 {
            println!(
                "{}/{} days left => {}/{} pending libraries, {}/{} books scanned",
                days_left,
                input.days,
                pending_libraries.len(),
                input.libraries.len(),
                total_scanned,
                input.scores.len(),
            );

            // Evaluate libraries available for sign up
            let mut library_scores: Vec<(usize, usize)> = Vec::new();
            for library_id in pending_libraries.iter() {
                let library = &input.libraries[*library_id];
                if library.signup >= days_left {
                    continue;
                }

                // Remove books scanned by last library signed up
                let books = library_books.get_mut(library_id).unwrap();
                books.retain(|(book_id, _)| !scanned_books.contains(book_id));

                // Compute the maximum sum of book scores this library can add
                let maxscans = (days_left - library.signup) * library.scanrate;
                let lib_score = books
                    .iter()
                    .take(maxscans)
                    .map(|(_, book_score)| *book_score)
                    .sum();

                // Only keep libraries that can still add to the total score
                if lib_score > 0 {
                    library_scores.push((*library_id, lib_score));
                }
            }

            // Pick next library to sign up
            let next_library_id;
            if let Some((library_id, _)) =
                library_scores.iter().max_by_key(|(_, score)| *score)
            {
                next_library_id = library_id;
            } else {
                break;
            }

            // Sign up library
            output.add_library(*next_library_id);
            let library = &input.libraries[*next_library_id];
            days_left -= library.signup;

            // Scan books
            let books = library_books.get_mut(next_library_id).unwrap();
            let max_scans = min(books.len(), days_left * library.scanrate);
            scanned_books = books
                .drain(0..max_scans)
                .map(|(book_id, _)| book_id)
                .collect::<HashSet<usize>>();
            assert_eq!(max_scans, scanned_books.len());
            total_scanned += scanned_books.len();
            for book_id in scanned_books.iter() {
                output.add_scan(*next_library_id, *book_id);
            }

            // Update pending libraries list
            pending_libraries = library_scores
                .iter()
                .map(|(library_id, _)| *library_id)
                .filter(|library_id| library_id != next_library_id)
                .collect();
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
