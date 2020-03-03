use super::{BookRef, Library, ScanningTask};
use num_format::{Locale, ToFormattedString};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use std::cmp::Ordering;
use std::collections::HashSet;
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
        }
    }

    pub fn build(&self) -> ScanningPlan {
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
                library.update_score(
                    days_left,
                    self.idle_exp,
                    signup_exp.next().unwrap(),
                );
            }

            // Remove libraries with max score zero
            pending_libraries.retain(|library| library.score > 0_f32);

            if let Some(next_lib) = pending_libraries.iter_mut().max() {
                // Sign up next library and select books for scanning
                let scanned_books = next_lib.scan_books(days_left);
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
    idle_days: u64,
}

impl<'a> PendingLibrary<'a> {
    fn new(library: &'a Library) -> Self {
        let mut books = library.books.iter().cloned().collect::<Vec<_>>();
        books.sort_unstable();
        books.reverse();
        Self {
            library,
            books,
            score: 0_f32,
            idle_days: 0,
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

    fn update_score(&mut self, days_left: u64, idle_exp: f32, signup_exp: f32) {
        let max_scans = self.max_scans(days_left);
        if self.library.signup_days >= days_left {
            self.score = 0_f32;
            return;
        }
        self.score = self
            .books
            .iter()
            .take(max_scans)
            .map(|book| book.score())
            .sum::<u64>() as f32;
        if self.score == 0_f32 {
            return;
        }
        self.score /= (self.library.signup_days as f32).powf(signup_exp);
        if idle_exp == 0_f32 {
            return;
        }
        self.idle_days = if max_scans > self.books.len() {
            let scan_days = (self.books.len() as f32
                / self.library.scan_rate as f32)
                .ceil() as u64;
            days_left - self.library.signup_days - scan_days
        } else {
            0
        };
        if self.idle_days > 0 {
            self.score /= (self.idle_days as f32).powf(idle_exp);
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
