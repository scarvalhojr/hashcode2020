use super::{BookRef, Library, ScanningTask};
use num_format::{Locale, ToFormattedString};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use std::cmp::{min, Ordering};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::iter::{repeat, FromIterator};
use std::mem::swap;

pub enum SignupExponent {
    Fixed(f32),
    Range(f32, f32, f32),
    Variable(usize, f32, f32),
}

pub struct PlanBuilder<'a> {
    task: &'a ScanningTask,
    idle_exp: f32,
    signup_exp: SignupExponent,
    copy_count: HashMap<BookRef, usize>,
    idential_scores: bool,
}

impl<'a> PlanBuilder<'a> {
    pub fn new(
        task: &'a ScanningTask,
        idle_exp: f32,
        signup_exp: SignupExponent,
    ) -> Self {
        Self {
            task,
            idle_exp,
            signup_exp,
            copy_count: task.count_book_copies(),
            idential_scores: task.min_book_score() == task.max_book_score()
        }
    }

    pub fn build(&self) -> ScanningPlan {
        let max_copies = self.copy_count.values().max().unwrap_or(&0);
        println!("Max number of copies: {}", max_copies);
        println!("Identical scores: {}", self.idential_scores);
        match &self.signup_exp {
            SignupExponent::Fixed(exp) => {
                println!("Sign-up exponent: {:0.4}", *exp);
                self.build_plan(&mut repeat(*exp))
            }
            SignupExponent::Range(start, end, step) => {
                let mut best_plan = ScanningPlan::new(self.task);
                let mut best_score = 0;
                let mut exp = *start;
                while exp <= *end {
                    let plan = self.build_plan(&mut repeat(exp));
                    if let Ok((score, _, _)) = plan.score() {
                        println!(
                            "Sign-up exponent {:0.4}, score {}",
                            exp,
                            score.to_formatted_string(&Locale::en)
                        );
                        if score > best_score {
                            best_plan = plan;
                            best_score = score;
                        }
                    }
                    exp += *step;
                }
                best_plan
            }
            SignupExponent::Variable(count, min_exp, max_exp) => {
                println!(
                    "Variable sign-up exponent: {:0.4} - {:0.4}",
                    *min_exp, *max_exp
                );
                let mut best_plan = ScanningPlan::new(self.task);
                let mut best_score = 0;
                if min_exp > max_exp {
                    return best_plan;
                }

                let mut exponents = Uniform::new_inclusive(min_exp, max_exp)
                    .sample_iter(thread_rng());
                for i in 1..=*count {
                    let plan = self.build_plan(&mut exponents);
                    if let Ok((score, _, _)) = plan.score() {
                        println!(
                            "Iteration {}, score {}",
                            i,
                            score.to_formatted_string(&Locale::en)
                        );
                        if score > best_score {
                            best_plan = plan;
                            best_score = score;
                        }
                    }
                }
                best_plan
            }
        }
    }

    fn build_plan<I>(&self, signup_exp: &mut I) -> ScanningPlan
    where
        I: Iterator<Item = f32>,
    {
        let mut copy_count = self.copy_count.clone();
        let mut plan = ScanningPlan::new(self.task);
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
                library.update(
                    days_left,
                    self.idle_exp,
                    signup_exp.next().unwrap(),
                    &copy_count,
                    self.idential_scores
                );
            }

            // Remove libraries with score zero and update copy counts
            pending_libraries.retain(|library| {
                if library.score > 0_f32 {
                    true
                } else {
                    library.discard_books(&mut copy_count);
                    false
                }
            });

            if let Some(next_lib) = pending_libraries.iter_mut().max() {
                // Sign up next library and select books for scanning
                let scanned_books = next_lib.scan_books(days_left, &mut copy_count);
                // let already_scanned = min(next_lib.max_scans(days_left), next_lib.library.books.len()) - scanned_books.len();
                // if already_scanned > 0 {
                //     println!("Scanned {} books from library {} **** {} already scanned!", scanned_books.len(), next_lib.library.id, already_scanned);
                // } else {
                //     println!("Scanned {} books from library {}", scanned_books.len(), next_lib.library.id);
                // }
                days_left -= next_lib.library.signup_days;

                // TODO: Fix Clippy warning
                let signedup_library = next_lib.library.clone();
                // Remove scanned books from remaining libraries
                for library in pending_libraries.iter_mut() {
                    library.remove_books(&scanned_books);
                }

                plan.add_library(signedup_library, scanned_books);
            } else {
                break;
            }
        }
        plan
    }
}

pub struct ScanningPlan<'a> {
    task: &'a ScanningTask,
    queue: Vec<(&'a Library, HashSet<BookRef>)>,
}

impl<'a> ScanningPlan<'a> {
    pub fn new(task: &'a ScanningTask) -> Self {
        Self {
            task,
            queue: Vec::new(),
        }
    }

    fn add_library(&mut self, library: &'a Library, books: HashSet<BookRef>) {
        self.queue.push((library, books));
    }

    pub fn score(&self) -> Result<(u64, u64, u64), String> {
        let mut idle_library_count = 0;
        let mut idle_slot_count = 0;
        let mut days_left = self.task.days;
        let mut scanned_books: HashSet<BookRef> = HashSet::new();
        for (library, books) in self.queue.iter() {
            if library.signup_days > days_left {
                return Err(format!(
                    "Library {} could not be signed up",
                    library.id
                ));
            }
            days_left -= library.signup_days;
            let max_scans = (days_left * library.scan_rate) as usize;
            if books.len() > max_scans {
                return Err(format!(
                    "Library {} cannot scan {} books",
                    library.id,
                    books.len()
                ));
            }
            if max_scans > books.len() {
                let scan_days = (books.len() as f32 / library.scan_rate as f32)
                    .ceil() as u64;
                if days_left > scan_days {
                    idle_library_count += 1;
                    idle_slot_count += days_left - scan_days;
                }
            }
            scanned_books.extend(books.iter().cloned());
        }

        let score = scanned_books.iter().map(|book| book.score()).sum();
        Ok((score, idle_library_count, idle_slot_count))
    }

    pub fn count_signedup_libraries(&self) -> usize {
        self.queue.len()
    }

    pub fn count_scanned_books(&self) -> usize {
        self.queue.iter().map(|(_, books)| books.len()).sum()
    }
}

struct PendingLibrary<'a> {
    library: &'a Library,
    books: Vec<BookRef>,
    score: f32,
}

impl<'a> PendingLibrary<'a> {
    fn new(library: &'a Library) -> Self {
        let books = library.books.iter().cloned().collect::<Vec<_>>();
        Self {
            library,
            books,
            score: 0_f32,
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

    fn discard_books(&self, copy_count: &mut HashMap<BookRef, usize>) {
        for book in self.books.iter() {
            copy_count
                .entry(book.clone())
                .and_modify(|count| *count -= 1);
        }
    }

    fn update(
        &mut self,
        days_left: u64,
        idle_exp: f32,
        signup_exp: f32,
        copy_count: &HashMap<BookRef, usize>,
        idential_scores: bool,
    ) {
        if self.library.signup_days >= days_left {
            self.score = 0_f32;
            return;
        }
        let max_scans = self.max_scans(days_left);
        // println!("Updating pending library {}: {} max scans, {} books available, {} days left)", self.library.id, max_scans, self.books.len(), days_left);
        if idential_scores {
            self.books.sort_unstable_by(|book_a: &BookRef, book_b: &BookRef| {
                copy_count
                    .get(book_a)
                    .unwrap_or(&0)
                    .cmp(copy_count.get(book_b).unwrap_or(&0))
            });
            // self.score = min(self.books.len(), max_scans) as f32;
            self.score = self
                .books
                .iter()
                .take(max_scans)
                .map(|book| (*(copy_count.get(book).unwrap_or(&0)) as f32).powf(-1.5))
                .sum::<f32>();
        } else {
            self.books.sort_unstable_by(|book_a: &BookRef, book_b: &BookRef| {
                book_a
                    .cmp(book_b)
                    .reverse()
                    .then(
                        copy_count
                            .get(book_a)
                            .unwrap_or(&0)
                            .cmp(copy_count.get(book_b).unwrap_or(&0)))
            });
            self.score = self
                .books
                .iter()
                .take(max_scans)
                .map(|book| {
                    book.score() as f32 / (*(copy_count.get(book).unwrap_or(&0)) as f32).powf(0.5)
                })
                .sum::<f32>() as f32;
        }
        // println!(
        //     "  Sorted books: {}",
        //     self.books
        //         .iter()
        //         .map(|book| format!("{}({},{})", book.id(), book.score(), copy_count.get(book).unwrap_or(&0)))
        //         .collect::<Vec<_>>()
        //         .join(", ")
        // );
        if self.score == 0_f32 {
            return;
        }
        self.score /= (self.library.signup_days as f32).powf(signup_exp);
        if idle_exp == 0_f32 {
            return;
        }
        let idle_days = if max_scans > self.books.len() {
            let scan_days = (self.books.len() as f32
                / self.library.scan_rate as f32)
                .ceil() as u64;
            days_left - self.library.signup_days - scan_days
        } else {
            0
        };
        if idle_days > 0 {
            self.score /= (idle_days as f32).powf(idle_exp);
        }
    }

    fn scan_books(
        &mut self,
        days_left: u64,
        copy_count: &mut HashMap<BookRef, usize>
    ) -> HashSet<BookRef>
    {
        let scans = self.max_scans(days_left);
        if scans < self.books.len() {
            for book in self.books.drain(scans..) {
                copy_count
                    .entry(book)
                    .and_modify(|count| *count -= 1);
            }
        }
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
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(Ordering::Less)
    }
}

impl PartialOrd for PendingLibrary<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Display for ScanningPlan<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.queue.len())?;
        for (library, books) in self.queue.iter() {
            writeln!(f, "{} {}", library.id, books.len())?;
            let book_list = books
                .iter()
                .map(|book| book.id().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(f, "{}", book_list)?;
        }
        Ok(())
    }
}
