use std::collections::HashMap;

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

use level::{CellContents, Level};
use program::{ProgramRef, Team};

pub struct MapView {
    window: Window,
    highlight: Option<ProgramRef>,
    highlight_range: Option<usize>,
    overlay: Vec<(Point, TermCell)>,
    named_overlay: HashMap<String, (Point, TermCell)>,
    help: Option<String>,
}

impl MapView {
    pub fn new(mut window: Window) -> MapView {
        window.border();
        MapView {
            window: window,
            highlight: None,
            highlight_range: None,
            overlay: Vec::new(),
            named_overlay: HashMap::new(),
            help: None,
        }
    }

    pub fn reset(&mut self) {
        self.overlay.clear();
        self.named_overlay.clear();
        self.clear_help();
        self.clear_highlight();
    }

    pub fn get_overlay(&mut self) -> &mut HashMap<String, (Point, TermCell)> {
        &mut self.named_overlay
    }

    pub fn from_global_frame(&self, p: Point) -> Option<Point> {
        self.window.position.from_global_frame(p)
    }

    pub fn display(&mut self, level: &Level) {
        for (y, line) in level.layout.iter().enumerate() {
            let y = y + 2;
            for (x, tile) in line.iter().enumerate() {
                let x = x + 2;
                self.window.put_at(Point::new(x as u16, y as u16), match Level::convert(*tile) {
                    Some(c) => c,
                    None => ' '.into(),
                });
            }
        }

        for program in level.programs.iter() {
            let color = match program.borrow().team {
                Team::Player => ColorValue::Green,
                Team::Enemy => ColorValue::Red,
            };
            for (p, tc) in program.borrow().display_color(color) {
                self.window.put_at(Point::new(p.x + 1, p.y + 1), tc);
            }
        }

        if let Some(ref program) = self.highlight {
            for (p, tc) in program.borrow().display_color(ColorValue::Blue) {
                self.window.put_at(Point::new(p.x + 1, p.y + 1), tc);
            }
        }

        for &(p, c) in self.overlay.iter() {
            self.window.put_at(Point::new(p.x + 1, p.y + 1), c);
        }

        for &(p, c) in self.named_overlay.values() {
            self.window.put_at(Point::new(p.x + 1, p.y + 1), c);
        }

        // TODO:
        self.window.print_at(Point::new(2, 23), "                                                         ");
        if let Some(ref help) = self.help {
            self.window.print_at(Point::new(2, 23), help);
        }
    }

    pub fn refresh(&mut self, compositor: &mut ::voodoo::compositor::Compositor) {
        self.window.refresh(compositor);
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

                            // Guard at map edges
                            if p.x <= 0 || p.y <= 0 {
                                continue;
                            }

                            if let Some(tc) = match level.contents_of(p) {
                                CellContents::Empty => Some('·'.into()),
                                CellContents::Unpassable | CellContents::Uplink => None,
                                CellContents::Program(p) => {
                                    if p.borrow().position != position && p.borrow().team == Team::Enemy {
                                        Some('X'.into())
                                    }
                                    else {
                                        None
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
