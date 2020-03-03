struct Student {
    id: usize,
}

struct Class<'a> {
    enrolled: Vec<&'a Student>,
}

struct School<'a> {
    students: Vec<Student>,
    classes: Vec<Class<'a>>,
}

impl School<'_> {
    fn new() -> Self {
        Self {
            students: Vec::new(),
            classes: Vec::new(),
        }
    }
    fn add_student(&mut self) {
        let id = self.students.len();
        self.students.push(Student { id });
    }
    fn enrol(&mut self, student_id: usize) {
        if let Some(student) = self.students.get(student_id) {
            self.classes.push(Class {
                enrolled: vec![], // student <== lifetime conflict
            });
        }
    }
    fn enrol_in_new_class(&mut self, student_id: usize) {
        let student = Student { id: student_id };
        self.students.push(student);
        self.classes.push(Class {
            enrolled: vec![], // &student <== lifetime conflict
        });
    }
    fn add_class(&mut self, student_ids: Vec<usize>) {
        let enrolled = self
            .students
            .iter()
            .filter(|s| student_ids.contains(&s.id))
            .collect::<Vec<&Student>>();
        self.classes.push(Class {
            enrolled: vec![], // <== lifetime conflict
        });
    }
}

fn run3() {
    let mut school = School::new();
    school.add_student();
    school.add_class(vec![0]);
    println!(
        "School has {} students, {} classes",
        school.students.len(),
        school.classes.len()
    )
}

fn run2() {
    let anna = Student { id: 0 };
    let mut school = School {
        students: vec![anna],
        classes: vec![],
    };
    school.add_class(vec![0]);
    println!(
        "School has {} students, {} classes",
        school.students.len(),
        school.classes.len()
    )
}

fn run1() {
    let anna = Student { id: 0 };
    let math = Class {
        enrolled: vec![&anna],
    };
    let school = School {
        students: vec![], // vec![anna] <== cannot move out of anna
        classes: vec![math],
    };
    println!(
        "School has {} students, {} classes",
        school.students.len(),
        school.classes.len()
    )
}

pub fn run() {
    let anna = Student { id: 0 };
    let bill = Student { id: 1 };
    let chris = Student { id: 2 };
    let math = Class {
        enrolled: vec![&anna, &bill],
    };
    let physics = Class {
        enrolled: vec![&anna, &chris],
    };
    println!("Math has {} students", math.enrolled.len());
    println!("Physics has {} students", physics.enrolled.len());
    run1();
    run2();
    run3();
}
