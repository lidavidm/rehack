use std::cell::RefCell;
use std::rc::Rc;
use voodoo::window::{Point, TermCell};

use program::{Program, ProgramRef};

pub struct Level {
    pub layout: Vec<String>,
    pub player_programs: Vec<ProgramRef>,
}

impl Level {
    pub fn new(description: &[&str; 22]) -> Level {
        let mut layout = Vec::new();
        for s in description.iter() {
            layout.push(s.to_string());
        }
        Level {
            layout: layout,
            player_programs: Vec::new(),
        }
    }

    pub fn add_player_program(&mut self, program: Program) {
        self.player_programs.push(Rc::new(RefCell::new(program)));
    }

    pub fn passable(&self, point: Point) -> bool {
        let cell = self.layout[(point.y - 1) as usize].chars().nth((point.x - 1) as usize);
        if cell != Some('.') {
            return false;
        }

        for program in self.player_programs.iter() {
            if program.borrow().intersects(point) {
                return false;
            }
        }

        return true;
    }

    // TODO: need char -> Tile -> DisplayChar

    pub fn convert(c: char) -> Option<TermCell> {
        match c {
            '.' => Some('·'.into()),
            'o' => Some('O'.into()),
            _ => None,
        }
    }
}
