use std::rc::Rc;

#[derive(Debug)]
struct Student {
    id: usize,
}

struct Class {
    id: usize,
    enrolled: Vec<Rc<Student>>,
}

struct School {
    students: Vec<Rc<Student>>,
    classes: Vec<Class>,
}

impl School {
    fn new() -> Self {
        Self {
            students: Vec::new(),
            classes: Vec::new(),
        }
    }
    fn add_student(&mut self) -> usize {
        let id = self.students.len();
        self.students.push(Rc::new(Student { id }));
        id
    }
    fn add_class(&mut self, student_ids: Vec<usize>) -> usize {
        let id = self.classes.len();
        let enrolled = self
            .students
            .iter()
            .filter(|s| student_ids.contains(&s.id))
            .cloned()
            .collect();
        self.classes.push(Class { id, enrolled });
        id
    }
}

pub fn run() {
    let mut school = School::new();
    let anna = school.add_student();
    let bill = school.add_student();
    let chris = school.add_student();
    let dan = school.add_student();
    let math = school.add_class(vec![anna, bill]);
    let history = school.add_class(vec![bill, chris]);
    println!("{} students, {} classes",school.students.len(),school.classes.len());
    println!("Enrolled in math: {:?}", school.classes[math].enrolled);
    println!("Enrolled in history: {:?}",school.classes[history].enrolled);
}
