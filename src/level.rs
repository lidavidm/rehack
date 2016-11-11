use std::cell::RefCell;
use std::rc::Rc;

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell};

use program::{Program, ProgramRef};

pub struct Level {
    pub layout: Vec<Vec<char>>,
    pub programs: Vec<ProgramRef>,
}

pub enum CellContents {
    Unpassable,
    Empty,
    Program(ProgramRef),
    Uplink,
}

impl Level {
    pub fn new(description: &[&str; 20]) -> Level {
        let mut layout = Vec::new();
        for s in description.iter() {
            layout.push(s.chars().collect());
        }
        Level {
            layout: layout,
            programs: Vec::new(),
        }
    }

    pub fn remove_uplink_at(&mut self, point: Point) {
        if self.layout[(point.y - 1) as usize][(point.x - 1) as usize] == 'o' {
            self.layout[(point.y - 1) as usize][(point.x - 1) as usize] = '.';
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
        let cell = self.layout[(point.y - 1) as usize][(point.x - 1) as usize];
        if cell != '.' {
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

        match self.layout[(point.y - 1) as usize][(point.x - 1) as usize] {
            '.' => CellContents::Empty,
            'o' => CellContents::Uplink,
            _ => CellContents::Unpassable,
        }
    }

    // TODO: need char -> Tile -> DisplayChar

    pub fn convert(c: char) -> Option<TermCell> {
        match c {
            '.' => Some('·'.into()),
            'o' => {
                let mut tc: TermCell = 'Θ'.into();
                tc.bg = Some(ColorValue::Green);
                Some(tc)
            }
            _ => None,
        }
    }
}
