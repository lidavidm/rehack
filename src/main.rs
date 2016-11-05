extern crate termion;
extern crate voodoo;

mod level;
mod program;

use std::io::{Write};

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead};

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

use level::Level;
use program::{Program, ProgramRef};

const LEVEL_DESCR: [&'static str; 22] = [
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "          .........................                       ",
    "          .........................                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "   o................             .....                    ",
    "   o................             .....                    ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          .........................                       ",
    "          .........................                       ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
];

enum UiState {
    Unselected,
    Selected,
}

enum UiEvent {
    Click(Point),
    // Movement
}

enum GameState {
    Setup,
    PlayerTurn,
    AITurn,
    AIExecute,
}

struct InfoView {
    window: Window,
}

impl InfoView {
    fn new(window: Window) -> InfoView {
        InfoView {
            window: window,
        }
    }

    fn refresh(&mut self, stdout: &mut std::io::Stdout) {
        self.window.refresh(stdout);
    }

    fn clear(&mut self) {
        for col in 2..self.window.width - 2 {
            self.window.put_at(Point::new(col, 2), ' ');
        }
    }

    fn display_program(&mut self, program: &Program) {
        self.window.print_at(Point::new(2, 2), &program.name);
    }
}

struct MapView {
    window: Window,
    highlight: Option<ProgramRef>,
    overlay: Vec<(Point, TermCell)>,
}

impl MapView {
    fn new(window: Window) -> MapView {
        MapView {
            window: window,
            highlight: None,
            overlay: Vec::new(),
        }
    }

    fn from_global_frame(&self, p: Point) -> Option<Point> {
        self.window.position.from_global_frame(p)
    }

    fn display(&mut self, level: &Level) {
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
            let (point, mut c) = program.borrow().render();
            if let Some(ref p) = self.highlight {
                if p.borrow().position == point {
                    c.bg = Some(ColorValue::Blue);
                }
            }
            self.window.put_at(point, c);
        }

        for &(p, c) in self.overlay.iter() {
            self.window.put_at(p, c);
        }
    }

    fn refresh(&mut self, stdout: &mut std::io::Stdout) {
        self.window.refresh(stdout);
    }

    fn highlight(&mut self, program: ProgramRef, level: &Level) {
        self.highlight = Some(program.clone());
        self.update_highlight(level);
    }

    fn update_highlight(&mut self, level: &Level) {
        if let Some(ref program) = self.highlight {
            self.overlay.clear();
            let Point { x, y } = program.borrow().position;
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

    fn clear_highlight(&mut self) {
        self.highlight = None;
        self.overlay.clear();
    }
}

struct UiModelView {
    info: InfoView,
    map: MapView,
}

impl UiState {
    fn translate_click(click: Point, map: &MapView) -> Option<Point> {
        for &(point, _) in map.overlay.iter() {
            if click == point {
                return Some(point);
            }
        }
        None
    }

    fn next(self, event: UiEvent, level: &mut Level, mv: &mut UiModelView) -> UiState {
        use UiEvent::*;
        use UiState::*;

        let UiModelView { ref mut info, ref mut map } = *mv;

        match (self, event) {
            (Unselected, Click(p)) => {
                for program in level.player_programs.iter() {
                    if intersects(&program.borrow(), p) {
                        map.highlight(program.clone(), &level);
                        info.display_program(&program.borrow());
                        return Selected;
                    }
                }
                Unselected
            }
            (Selected, Click(p)) => {
                let result = Self::translate_click(p, map);
                if let Some(p) = result {
                    if let Some(ref mut program) = map.highlight {
                        program.borrow_mut().position = p;
                    }
                    map.update_highlight(&level);
                    Selected
                }
                else {
                    map.clear_highlight();
                    info.clear();
                    Unselected
                }
            }
        }
    }
}

fn intersects(program: &Program, point: Point) -> bool {
    program.position == point
}

fn main() {
    use voodoo::terminal::{Mode, Terminal};
    let mut level = Level::new(&LEVEL_DESCR);
    level.add_player_program(Program::new(Point::new(11, 11), "Hack"));
    level.add_player_program(Program::new(Point::new(5, 12), "Hack"));

    let mut terminal = Terminal::new();
    terminal.cursor(Mode::Disabled);
    terminal.clear_color(ColorValue::Black);
    let Terminal { ref mut stdin, ref mut stdout } = terminal;

    stdout.flush().unwrap();

    let mut info = voodoo::window::Window::new(Point::new(0, 0), 20, 24);
    let mut map = voodoo::window::Window::new(Point::new(20, 0), 60, 24);
    info.border();
    map.border();

    let mut info_view = InfoView::new(info);
    let mut map_view = MapView::new(map);
    info_view.refresh(stdout);
    map_view.display(&level);
    map_view.refresh(stdout);

    let mut ui_modelview = UiModelView {
        info: info_view,
        map: map_view,
    };
    let mut ui_state = UiState::Unselected;

    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Char('q')) => break,
            Event::Mouse(me) => {
                match me {
                    MouseEvent::Press(_, x, y) => {
                        if let Some(p) = ui_modelview.map.from_global_frame(Point::new(x, y)) {
                            // TODO: if movement controls are active, translate the event
                            // use a method on MapView to get the actual UiEvent
                            ui_state = ui_state.next(
                                UiEvent::Click(p),
                                &mut level,
                                &mut ui_modelview,
                            );
                        }
                    },
                    _ => (),
                }
            }
            _ => {}
        }
        ui_modelview.info.refresh(stdout);
        ui_modelview.map.display(&level);
        ui_modelview.map.refresh(stdout);
    }
 }
