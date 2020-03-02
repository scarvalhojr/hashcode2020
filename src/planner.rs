use super::{BookRef, Library, ScanningTask};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;
use std::mem::swap;

pub struct ScanningPlan<'a> {
    task: &'a ScanningTask,
    library_queue: Vec<&'a Library>,
    library_scans: HashMap<&'a Library, HashSet<BookRef>>,
}

impl<'a> ScanningPlan<'a> {
    pub fn new(task: &'a ScanningTask) -> Self {
        Self {
            task,
            library_queue: Vec::new(),
            library_scans: HashMap::new(),
        }
    }

    fn add_library(&mut self, library: &'a Library, books: HashSet<BookRef>) {
        self.library_queue.push(library);
        self.library_scans.insert(library, books);
    }

    pub fn solve(&mut self) {
        let mut pending_libraries = self
            .task
            .libraries
            .iter()
            .map(|library| PendingLibrary::new(library))
            .collect::<Vec<_>>();

        let mut days_left = self.task.days;
        while days_left > 0 {
            // Update max scores of pending libraries
            for library in pending_libraries.iter_mut() {
                library.update_score(days_left);
            }

            // Remove libraries with max score zero
            pending_libraries.retain(|library| library.max_score > 0_f32);

            if let Some(next_lib) = pending_libraries.iter_mut().max() {
                // Sign up next library and select books for scanning
                let scanned_books = next_lib.scan_books(days_left);
                days_left -= next_lib.library.signup_days;
                // println!("Library {} will scan {:?}", next_lib.library.id, scanned_books);

                let signedup_library = next_lib.library.clone();
                // Remove scanned books from remaining libraries
                for library in pending_libraries.iter_mut() {
                    library.remove_books(&scanned_books);
                }

                self.add_library(signedup_library, scanned_books);
            } else {
                break;
            }
        }
    }

    pub fn score(&self) -> Result<u64, String> {
        let mut days_left = self.task.days;
        let mut scanned_books: HashSet<BookRef> = HashSet::new();
        for &library in self.library_queue.iter() {
            if library.signup_days > days_left {
                return Err(format!(
                    "Library {} could not be signed up",
                    library.id
                ));
            }
            days_left -= library.signup_days;
            let max_scans = (days_left * library.scan_rate) as usize;
            if let Some(books) = self.library_scans.get(library) {
                if books.len() > max_scans {
                    return Err(format!(
                        "Library {} cannot scan {} books",
                        library.id,
                        books.len()
                    ));
                }
                scanned_books.extend(books.iter().cloned());
            }
        }

        let score = scanned_books.iter().map(|book| book.score()).sum();
        Ok(score)
    }
}

struct PendingLibrary<'a> {
    library: &'a Library,
    books: Vec<BookRef>,
    max_score: f32,
}

impl<'a> PendingLibrary<'a> {
    fn new(library: &'a Library) -> Self {
        let mut books = library.books.iter().cloned().collect::<Vec<_>>();
        books.sort_unstable();
        books.reverse();
        Self {
            library,
            books,
            max_score: 0_f32,
        }
    }

    fn max_scans(&self, days: u64) -> usize {
        if days > self.library.signup_days {
            ((days - self.library.signup_days) * self.library.scan_rate)
                as usize
        } else {
            0
        }
    }

    fn update_score(&mut self, days_left: u64) {
        if self.library.signup_days >= days_left {
            self.max_score = 0_f32;
        } else {
            self.max_score = self
                .books
                .iter()
                .take(self.max_scans(days_left))
                .map(|book| book.score())
                .sum::<u64>() as f32;
            if self.max_score > 0_f32 {
                self.max_score /= self.library.signup_days as f32;
            }
        }
    }

    fn scan_books(&mut self, days_left: u64) -> HashSet<BookRef> {
        self.books.truncate(self.max_scans(days_left));
        let mut selected = Vec::new();
        swap(&mut self.books, &mut selected);
        HashSet::from_iter(selected)
    }

    fn remove_books(&mut self, scanned_books: &HashSet<BookRef>) {
        self.books.retain(|book| !scanned_books.contains(book))
    }
}

impl PartialEq for PendingLibrary<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.library == other.library
    }
}

impl Eq for PendingLibrary<'_> {}

impl Ord for PendingLibrary<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.max_score
            .partial_cmp(&other.max_score)
            .unwrap_or(Ordering::Less)
    }
}

impl PartialOrd for PendingLibrary<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.max_score.partial_cmp(&other.max_score)
    }
}

impl Display for ScanningPlan<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.library_queue.len())?;
        for library in self.library_queue.iter() {
            if let Some(books) = self.library_scans.get(library) {
                writeln!(f, "{} {}", library.id, books.len())?;
                let book_list = books
                    .iter()
                    .map(|book| book.id().to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(f, "{}", book_list)?;
            } else {
                writeln!(f, "{} 0\n", library.id)?;
            }
        }
        Ok(())
    }
}
