use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

use level::{CellContents, Level};
use program::{ProgramRef};

pub struct MapView {
    window: Window,
    highlight: Option<ProgramRef>,
    highlight_range: Option<usize>,
    overlay: Vec<(Point, TermCell)>,
    help: Option<String>,
}

impl MapView {
    pub fn new(window: Window) -> MapView {
        MapView {
            window: window,
            highlight: None,
            highlight_range: None,
            overlay: Vec::new(),
            help: None,
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

        // TODO:
        self.window.print_at(Point::new(2, 23), "                                                         ");
        if let Some(ref help) = self.help {
            self.window.print_at(Point::new(2, 23), help);
        }
    }

    pub fn refresh(&mut self, stdout: &mut ::std::io::Stdout) {
        self.window.refresh(stdout);
    }

    pub fn highlight(&mut self, program: ProgramRef, level: &Level) {
        self.highlight = Some(program.clone());
        self.update_highlight(level);
    }

    pub fn highlight_range(&mut self, range: usize, level: &Level) {
        self.highlight_range = Some(range);
        self.update_highlight(level);
    }

    pub fn clear_range(&mut self) {
        self.highlight_range = None;
    }

    pub fn update_highlight(&mut self, level: &Level) {
        if let Some(ref program) = self.highlight {
            self.overlay.clear();
            let position = { program.borrow().position };
            let Point { x, y } = position;

            if let Some(range) = self.highlight_range {
                let range = range as isize;

                // TODO: change this to use refactored function
                for dx in -range..range + 1 {
                    for dy in -range..range + 1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        if dx.abs() + dy.abs() <= range {
                            let p = Point::new((x as isize + dx) as u16, (y as isize + dy) as u16);
                            if let Some(tc) = match level.contents_of(p) {
                                CellContents::Empty => Some('·'.into()),
                                CellContents::Unpassable => None,
                                CellContents::Program(p) => {
                                    if p.borrow().position == position {
                                        None
                                    }
                                    else {
                                        Some('X'.into())
                                    }
                                },
                            } {
                                let mut tc: TermCell = tc;
                                tc.bg = Some(ColorValue::Magenta);
                                self.overlay.push((p, tc));
                            }
                        }
                    }
                }
            }
            else {
                if !program.borrow().can_move() {
                    return;
                }

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

    pub fn set_help<S: Into<String>>(&mut self, s: S) {
        self.help = Some(s.into());
    }

    pub fn clear_help(&mut self) {
        self.help = None;
    }
}
