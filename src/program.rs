use std::cell::RefCell;
use std::rc::Rc;

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

#[derive(Clone,Copy,Debug)]
pub enum Ability {
    Destroy { damage: usize, range: usize },
}

impl Ability {
    pub fn reachable_tiles(&self, center: Point) -> Vec<Point> {
        let mut result = vec![];
        let Point { x, y } = center;
        match *self {
            Ability::Destroy { range, .. } => {
                let range = range as isize;
                for dx in -range..range + 1 {
                    for dy in -range..range + 1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        if dx.abs() + dy.abs() <= range {
                            result.push(Point::new((x as isize + dx) as u16, (y as isize + dy) as u16));
                        }
                    }
                }
            }
        }

        return result;
    }
}

pub struct ProgramTurnState {
    pub moves_made: usize,
    pub ability_used: bool,
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Team {
    Player,
    Enemy,
}

pub struct Program {
    pub team: Team,
    pub position: Point,
    tail: Vec<Point>,
    pub name: String,
    pub abilities: Vec<(String, Ability)>,
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
    pub fn new(team: Team, position: Point, name: &str) -> Program {
        Program {
            team: team,
            position: position,
            tail: vec![],
            name: name.to_owned(),
            abilities: vec![("Bitblast".to_owned(), Ability::Destroy { damage: 2, range: 1 })],
            max_tail: 4,
            max_moves: 3,
            turn_state: ProgramTurnState::new(),
        }
    }

    pub fn begin_turn(&mut self) {
        self.turn_state.moves_made = 0;
        self.turn_state.ability_used = false;
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
