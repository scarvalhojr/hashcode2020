pub mod greedy;

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct Input {
    pub days: usize,
    pub scores: Vec<usize>,
    pub libraries: Vec<Library>,
}

pub struct Library {
    pub signup: usize,
    pub scanrate: usize,
    pub books: Vec<usize>,
}

impl Library {
    pub fn new(signup: usize, scanrate: usize, books: Vec<usize>) -> Library {
        Library {
            signup,
            scanrate,
            books,
        }
    }
}

impl Input {
    fn new(days: usize, scores: Vec<usize>, libraries: Vec<Library>) -> Input {
        Input {
            days,
            scores,
            libraries,
        }
    }
}

impl FromStr for Input {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut next_values = || -> Result<Vec<_>, Self::Err> {
            lines
                .next()
                .ok_or("Incomplete input")?
                .split_whitespace()
                .map(|token| {
                    token.parse().map_err(|err| format!("{}: {}", err, token))
                })
                .collect::<Result<_, _>>()
        };
        let take2 = |values: Vec<usize>| -> Result<(usize, usize), Self::Err> {
            if values.len() != 3 {
                return Err("Invalid format".to_string());
            }
            Ok((values[1], values[2]))
        };

        let (num_libs, days) = take2(next_values()?)?;
        let books = next_values()?;
        let mut libraries = Vec::with_capacity(num_libs as usize);
        for _ in 0..num_libs {
            let (signup, scanrate) = take2(next_values()?)?;
            let books = next_values()?;
            // TODO: validate book IDs
            libraries.push(Library::new(signup, scanrate, books));
        }

        Ok(Input::new(days, books, libraries))
    }
}

#[derive(Default)]
pub struct Output {
    pub library_ids: Vec<usize>,
    pub scanned_books: HashMap<usize, Vec<usize>>,
}

impl Output {
    pub fn add_library(&mut self, library_id: usize) {
        self.library_ids.push(library_id);
        self.scanned_books.insert(library_id, Vec::new());
    }
    pub fn add_scan(&mut self, library_id: usize, book_id: usize) {
        // TODO: check library ID is in library_ids
        if let Some(books) = self.scanned_books.get_mut(&library_id) {
            books.push(book_id);
        } // TODO: handle else
    }
    pub fn purge_idle(&mut self) {
        self.scanned_books.retain(|_, books| !books.is_empty());
        let retained_ids = self.scanned_books.keys().collect::<HashSet<_>>();
        self.library_ids
            .retain(|lib_id| retained_ids.contains(lib_id));
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.library_ids.len())?;
        for lib_id in &self.library_ids {
            let books = self.scanned_books.get(lib_id).unwrap();
            writeln!(f, "{} {}", lib_id, books.len())?;
            let book_list = books
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(f, "{}", book_list)?;
        }
        Ok(())
    }
}

pub trait Solver {
    fn solve(&self, input: &Input) -> Output;
}

pub fn score(input: &Input, output: &Output) -> Result<usize, String> {
    let mut day = 0;
    let mut total_score = 0;
    let mut scanned = HashSet::new();
    for library_id in &output.library_ids {
        let library = input
            .libraries
            .get(*library_id)
            .ok_or_else(|| format!("Invalid library ID {}", library_id))?;
        day += library.signup;
        let books = output.scanned_books.get(library_id).ok_or_else(|| {
            format!("Missing book list for library {}", library_id)
        })?;
        let max_scans = library.scanrate * (input.days - day);
        if books.len() > max_scans {
            return Err(format!(
                "Library {} cannot scan more than {} books",
                library_id, max_scans,
            ));
        }
        for book_id in books.iter() {
            if !library.books.contains(book_id) {
                return Err(format!(
                    "Book {} is not in library {}",
                    book_id, library_id,
                ));
            }
            if scanned.insert(book_id) {
                total_score += input
                    .scores
                    .get(*book_id)
                    .ok_or_else(|| format!("Invalid book ID {}", book_id))?;
            }
        }
    }

    Ok(total_score)
}
