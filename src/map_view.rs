use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

use level::Level;
use program::{Program, ProgramRef};

pub struct MapView {
    window: Window,
    highlight: Option<ProgramRef>,
    overlay: Vec<(Point, TermCell)>,
}

impl MapView {
    pub fn new(window: Window) -> MapView {
        MapView {
            window: window,
            highlight: None,
            overlay: Vec::new(),
        }
    }

    pub fn from_global_frame(&self, p: Point) -> Option<Point> {
        self.window.position.from_global_frame(p)
    }

    pub fn display(&mut self, level: &Level) {
        for (y, line) in level.layout.iter().enumerate() {
            let y = y + 1;
            for (x, tile) in line.chars().enumerate() {
                let x = x + 1;
                match Level::convert(tile) {
                    Some(c) => self.window.put_at(Point::new(x as u16, y as u16), c),
                    None => {},
                }
            }
        }

        for program in level.player_programs.iter() {
            program.borrow().display_color(ColorValue::Green, &mut self.window);
        }

        for program in level.enemy_programs.iter() {
            program.borrow().display_color(ColorValue::Red, &mut self.window);
        }

        if let Some(ref program) = self.highlight {
            program.borrow().display_color(ColorValue::Blue, &mut self.window);
        }

        for &(p, c) in self.overlay.iter() {
            self.window.put_at(p, c);
        }
    }

    pub fn refresh(&mut self, stdout: &mut ::std::io::Stdout) {
        self.window.refresh(stdout);
    }

    pub fn highlight(&mut self, program: ProgramRef, level: &Level) {
        self.highlight = Some(program.clone());
        self.update_highlight(level);
    }

    pub fn update_highlight(&mut self, level: &Level) {
        if let Some(ref program) = self.highlight {
            self.overlay.clear();
            let program = program.borrow();

            if !program.can_move() {
                return;
            }

            let Point { x, y } = program.position;
            let east = Point::new(x + 1, y);
            if level.passable(east) {
                self.overlay.push((east, '→'.into()));
            }
            let west = Point::new(x - 1, y);
            if level.passable(west) {
                self.overlay.push((west, '←'.into()));
            }
            let north = Point::new(x, y - 1);
            if level.passable(north) {
                self.overlay.push((north, '↑'.into()));
            }
            let south = Point::new(x, y + 1);
            if level.passable(south) {
                self.overlay.push((south, '↓'.into()));
            }
        }
    }

    pub fn clear_highlight(&mut self) {
        self.highlight = None;
        self.overlay.clear();
    }

    pub fn get_highlight(&self) -> Option<ProgramRef> {
        self.highlight.as_ref().map(Clone::clone)
    }

    pub fn translate_click(&self, click: Point) -> Option<Point> {
        for &(point, _) in self.overlay.iter() {
            if click == point {
                return Some(point);
            }
        }
        None
    }
}
