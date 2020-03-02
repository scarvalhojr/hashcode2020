pub mod planner;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug)]
struct Book {
    id: u32,
    score: u64,
}

impl Hash for Book {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Book {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Book {}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BookRef(Rc<Book>);

impl BookRef {
    fn new(id: u32, score: u64) -> Self {
        Self(Rc::new(Book { id, score }))
    }
    pub fn id(&self) -> u32 {
        self.0.id
    }
    pub fn score(&self) -> u64 {
        self.0.score
    }
}

impl Borrow<u32> for BookRef {
    fn borrow(&self) -> &u32 {
        &self.0.id
    }
}

impl Ord for BookRef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.score.cmp(&other.0.score)
    }
}

impl PartialOrd for BookRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq)]
pub struct Library {
    pub id: u32,
    pub signup_days: u64,
    pub scan_rate: u64,
    pub books: HashSet<BookRef>,
}

impl Library {
    fn new(
        id: u32,
        signup_days: u64,
        scan_rate: u64,
        books: HashSet<BookRef>,
    ) -> Self {
        Self {
            id,
            signup_days,
            scan_rate,
            books,
        }
    }
}

impl Hash for Library {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Library {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct ScanningTask {
    pub days: u64,
    pub books: HashSet<BookRef>,
    pub libraries: HashSet<Library>,
}

impl ScanningTask {
    fn new(days: u64, num_libraries: usize, book_scores: Vec<u32>) -> Self {
        let books = book_scores
            .iter()
            .enumerate()
            .map(|(id, score)| BookRef::new(id as u32, *score as u64))
            .collect();
        let libraries = HashSet::with_capacity(num_libraries);
        Self {
            days,
            books,
            libraries,
        }
    }

    fn add_library(
        &mut self,
        signup_days: u64,
        scan_rate: u64,
        book_ids: Vec<u32>,
    ) -> Result<(), String> {
        let id = self.libraries.len() as u32;
        let books = book_ids
            .iter()
            .map(|book_id| {
                self.books.get(book_id).cloned().ok_or_else(|| {
                    format!("Invalid book id {} in library {}", book_id, id)
                })
            })
            .collect::<Result<_, _>>()?;
        self.libraries
            .insert(Library::new(id, signup_days, scan_rate, books));
        Ok(())
    }
}

impl FromStr for ScanningTask {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut next_values = || -> Result<Vec<u32>, Self::Err> {
            lines
                .next()
                .ok_or("Incomplete input")?
                .split_whitespace()
                .map(|token| {
                    token.parse().map_err(|err| format!("{}: {}", err, token))
                })
                .collect::<Result<_, _>>()
        };
        let take3 = |values: Vec<_>| -> Result<(_, _, _), Self::Err> {
            if values.len() != 3 {
                return Err("Invalid format".to_string());
            }
            Ok((values[0], values[1], values[2]))
        };

        let (_, num_libraries, task_days) = take3(next_values()?)?;
        let book_scores = next_values()?;
        let mut task = ScanningTask::new(task_days as u64, num_libraries as usize, book_scores);
        for _ in 0..num_libraries {
            let (_, signup_days, scan_rate) = take3(next_values()?)?;
            let book_ids = next_values()?;
            task.add_library(signup_days as u64, scan_rate as u64, book_ids)?;
        }

        Ok(task)
    }
}
