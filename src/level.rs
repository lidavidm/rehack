use std::cell::RefCell;
use std::rc::Rc;
use voodoo::window::{Point, TermCell};

use program::{Program, ProgramRef};

pub struct Level {
    pub layout: Vec<String>,
    pub programs: Vec<ProgramRef>,
}

pub enum CellContents {
    Unpassable,
    Empty,
    Program(ProgramRef),
}

impl Level {
    pub fn new(description: &[&str; 20]) -> Level {
        let mut layout = Vec::new();
        for s in description.iter() {
            layout.push(s.to_string());
        }
        Level {
            layout: layout,
            programs: Vec::new(),
        }
    }

    pub fn add_program(&mut self, program: Program) {
        self.programs.push(Rc::new(RefCell::new(program)));
    }

    pub fn remove_program_at(&mut self, point: Point) {
        self.programs.retain(|p| { p.borrow().position != point });
    }

    pub fn begin_turn(&mut self) {
        for program in self.programs.iter() {
            program.borrow_mut().begin_turn();
        }
    }

    pub fn passable(&self, point: Point) -> bool {
        let cell = self.layout[(point.y - 1) as usize].chars().nth((point.x - 1) as usize);
        if cell != Some('.') {
            return false;
        }

        for program in self.programs.iter() {
            if program.borrow().intersects(point) {
                return false;
            }
        }

        return true;
    }

    pub fn contents_of(&self, point: Point) -> CellContents {
        for program in self.programs.iter() {
            if program.borrow().intersects(point) {
                return CellContents::Program(program.clone());
            }
        }

        match self.layout[(point.y - 1) as usize].chars().nth((point.x - 1) as usize) {
            Some('.') => CellContents::Empty,
            _ => CellContents::Unpassable,
        }
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
