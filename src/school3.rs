use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Debug, Eq)]
struct Student {
    id: usize,
}

impl Hash for Student {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Student {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StudentRef(Rc<Student>);

impl StudentRef {
    fn new(id: usize) -> Self {
        Self(Rc::new(Student { id }))
    }
    fn id(&self) -> usize {
        self.0.id
    }
}

impl Borrow<usize> for StudentRef {
    fn borrow(&self) -> &usize {
        &self.0.id
    }
}

struct Class {
    id: usize,
    enrolled: HashSet<StudentRef>,
}

struct School {
    students: HashSet<StudentRef>,
    classes: Vec<Class>,
}

impl School {
    fn new() -> Self {
        Self {
            students: HashSet::new(),
            classes: Vec::new(),
        }
    }
    fn add_student(&mut self) -> usize {
        let id = self.students.len();
        self.students.insert(StudentRef::new(id));
        id
    }
    fn add_class(&mut self, student_ids: Vec<usize>) -> usize {
        let id = self.classes.len();
        let enrolled = self
            .students
            .iter()
            .filter(|s| student_ids.contains(&s.id()))
            .cloned()
            .collect();
        self.classes.push(Class { id, enrolled });
        id
    }
}

pub fn run() {
    let mut school = School::new();
    let id_anna = school.add_student();
    let id_bill = school.add_student();
    let id_chris = school.add_student();
    let _id_dan = school.add_student();
    let id_math = school.add_class(vec![id_anna, id_bill]);
    let id_history = school.add_class(vec![id_bill, id_chris]);
    println!(
        "{} students, {} classes",
        school.students.len(),
        school.classes.len()
    );
    for &class_id in [id_math, id_history].iter() {
        println!(
            "Enrolled in class {}: {:?}",
            school.classes[class_id].id, school.classes[class_id].enrolled
        );
    }
}
