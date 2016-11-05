use std::cell::RefCell;
use std::rc::Rc;

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell};

pub struct Program {
    pub position: Point,
    pub name: String,
    pub abilities: Vec<String>,
}

pub type ProgramRef = Rc<RefCell<Program>>;

impl Program {
    pub fn new(position: Point, name: &str) -> Program {
        Program {
            position: position,
            name: name.to_owned(),
            abilities: vec![],
        }
    }

    pub fn render(&self) -> (Point, TermCell) {
        let mut tc: TermCell = '◘'.into();
        tc.bg = Some(ColorValue::Green);
        (self.position, tc)
    }
}
