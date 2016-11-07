use std::cell::RefCell;
use std::rc::Rc;

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

pub struct ProgramTurnState {
    pub moves_made: usize,
    pub ability_used: bool,
}

pub struct Program {
    pub position: Point,
    tail: Vec<Point>,
    pub name: String,
    pub abilities: Vec<String>,
    pub max_tail: usize,
    pub max_moves: usize,
    pub turn_state: ProgramTurnState,
}

pub type ProgramRef = Rc<RefCell<Program>>;

impl ProgramTurnState {
    fn new() -> ProgramTurnState {
        ProgramTurnState {
            moves_made: 0,
            ability_used: false,
        }
    }
}

impl Program {
    pub fn new(position: Point, name: &str) -> Program {
        Program {
            position: position,
            tail: vec![],
            name: name.to_owned(),
            abilities: vec!["Bitblast".to_owned()],
            max_tail: 4,
            max_moves: 3,
            turn_state: ProgramTurnState::new(),
        }
    }

    pub fn can_move(&self) -> bool {
        self.turn_state.moves_made < self.max_moves
    }

    pub fn move_to(&mut self, point: Point) {
        if !self.can_move() {
            return;
        }

        self.turn_state.moves_made += 1;

        self.tail.push(self.position);
        if self.tail.len() >= self.max_tail {
            self.tail.remove(0);
        }
        self.position = point;
    }

    pub fn damage(&mut self) -> bool {
        if self.tail.len() > 0 {
            self.tail.remove(0);
            true
        }
        else {
            false
        }
    }

    pub fn display_color(&self, color: ColorValue, window: &mut Window) {
        let mut prev: Option<Point> = None;
        let mut cells = self.tail.to_vec();
        cells.push(self.position);
        for w in cells.windows(2) {
            let (cur, next) = (w[0], w[1]);
            let (x1, y1) = (cur.x as i32, cur.y as i32);
            let (x2, y2) = (next.x as i32, next.y as i32);
            let dx = x2 - x1;
            let dy = y2 - y1;

            let pdx = prev.map(|p| x1 - p.x as i32);
            let pdy = prev.map(|p| y1 - p.y as i32);

            let mut c: TermCell = match (pdx, pdy, dx, dy) {
                (None, None, 1, 0) | (None, None, -1, 0) | (Some(1), _, 1, 0) | (Some(-1), _, -1, 0) => '═',
                (None, None, 0, 1) | (None, None, 0, -1) | (_, Some(1), 0, 1) | (_, Some(-1), 0, -1) => '║',
                (Some(1), _, 0, 1) | (_, Some(-1), -1, 0) => '╗',
                (Some(1), _, 0, -1) | (_, Some(1), -1, 0) => '╝',
                (Some(-1), _, 0, -1) | (_, Some(1), 1, 0) => '╚',
                (Some(-1), _, 0, 1) | (_, Some(-1), 1, 0) => '╔',
                _ => '+',
            }.into();
            c.bg = Some(color);
            window.put_at(cur, c);
            prev = Some(cur);
        }

        let mut tc: TermCell = '◘'.into();
        tc.bg = Some(color);
        window.put_at(self.position, tc);
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
