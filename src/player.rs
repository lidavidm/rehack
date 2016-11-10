use program::Program;

pub struct Player {
    pub name: String,
    pub programs: Vec<Program>,
}

impl Player {
    pub fn new<S: Into<String>>(name: S) -> Player {
        Player {
            name: name.into(),
            programs: Vec::new(),
        }
    }
}
