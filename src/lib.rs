pub mod greedy;

use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::fmt::{Display, Formatter};

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
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.library_ids.len())?;
        for lib_id in &self.library_ids {
            let books = self.scanned_books.get(lib_id).unwrap();
            write!(f, "{} {}\n", lib_id, books.len())?;
            let book_list = books.iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            write!(f, "{}\n", book_list)?;
        }
        Ok(())
    }
}

pub trait Solver {
    fn solve(&self, input: &Input) -> Output;
}

pub fn score(input: &Input, output: &Output) -> Result<usize, String> {
    let mut min_days = 0;
    let mut signup_queue = Vec::new();
    for library_id in &output.library_ids {
        let library = input
            .libraries
            .get(*library_id)
            .ok_or_else(|| format!("Invalid library ID: {}", library_id))?;
        min_days += library.signup;
        if let Some(books) = output.scanned_books.get(library_id) {
            for book_id in books {
                if !library.books.contains(book_id) {
                    return Err(format!(
                        "Book {} is not in library {}",
                        book_id, library_id
                    ));
                }
            }
            signup_queue.push((min_days, library.scanrate, 0, books));
        }
    }

    let mut total_score = 0;
    let mut scanned: HashSet<usize> = HashSet::new();

    for day in 0..input.days {
        for (min_days, scanrate, next_book, books) in signup_queue.iter_mut() {
            if day < *min_days {
                break;
            }
            for index in *next_book..(*next_book + *scanrate) {
                if let Some(&book_id) = books.get(index) {
                    let score = input.scores.get(book_id).ok_or_else(|| {
                        format!("Invalid book ID: {}", book_id)
                    })?;
                    if scanned.insert(book_id) {
                        total_score += score;
                    }
                }
            }
            *next_book += *scanrate;
        }
    }
    Ok(total_score)
}
