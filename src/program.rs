use std::cell::RefCell;
use std::rc::Rc;

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

pub struct Program {
    pub position: Point,
    tail: Vec<Point>,
    pub name: String,
    pub abilities: Vec<String>,
}

pub type ProgramRef = Rc<RefCell<Program>>;

impl Program {
    pub fn new(position: Point, name: &str) -> Program {
        Program {
            position: position,
            tail: vec![],
            name: name.to_owned(),
            abilities: vec![],
        }
    }

    pub fn move_to(&mut self, point: Point) {
        self.tail.push(self.position);
        self.position = point;
    }

    pub fn display_color(&self, color: ColorValue, window: &mut Window) {
        let mut tc: TermCell = '◘'.into();
        tc.bg = Some(color);
        window.put_at(self.position, tc);
        for point in self.tail.iter() {
            window.put_at(*point, tc);
        }
    }

    pub fn intersects(&self, point: Point) -> bool {
        for t in self.tail.iter() {
            if *t == point {
                return true;
            }
        }
        self.position == point
    }
}
