extern crate termion;
extern crate voodoo;

use std::io::{Write};

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead};

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

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

struct Program {
    position: Point,
    name: String,
    abilities: Vec<String>,
}

impl Program {
    fn new(position: Point, name: &str) -> Program {
        Program {
            position: position,
            name: name.to_owned(),
            abilities: vec![],
        }
    }

    fn render(&self) -> (Point, TermCell) {
        let mut tc: TermCell = '◘'.into();
        tc.bg = Some(ColorValue::Green);
        (self.position, tc)
    }
}

struct Level {
    layout: Vec<String>,
    player_programs: Vec<Program>,
}

impl Level {
    fn new(description: &[&str; 22]) -> Level {
        let mut layout = Vec::new();
        for s in description.iter() {
            layout.push(s.to_string());
        }
        Level {
            layout: layout,
            player_programs: Vec::new(),
        }
    }

    fn passable(&self, point: Point) -> bool {
        // TODO: check programs too
        let cell = self.layout[(point.y - 2) as usize].chars().nth((point.x - 2) as usize);
        cell == Some('.')
    }

    // TODO: need char -> Tile -> DisplayChar

    fn convert(c: char) -> Option<TermCell> {
        match c {
            '.' => Some('·'.into()),
            'o' => Some('O'.into()),
            _ => None,
        }
    }
}

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
    highlight: Option<Point>,
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
            let (point, mut c) = program.render();
            if let Some(p) = self.highlight {
                if p == point {
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

    fn highlight(&mut self, p: Point, level: &Level) {
        self.highlight = Some(p);
        let Point { x, y } = p;
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
    fn next(self, event: UiEvent, level: &mut Level, mv: &mut UiModelView) -> UiState {
        use UiEvent::*;
        use UiState::*;

        let UiModelView { ref mut info, ref mut map } = *mv;

        match (self, event) {
            (Unselected, Click(p)) => {
                for program in level.player_programs.iter() {
                    if intersects(&program, p) {
                        map.highlight(p, &level);
                        info.display_program(program);
                        return Selected;
                    }
                }
                Unselected
            }
            (Selected, Click(_)) => {
                for program in level.player_programs.iter() {
                    let (p, mut tc) = program.render();
                    map.clear_highlight();
                }
                info.clear();
                Unselected
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
    level.player_programs.push(Program::new(Point::new(11, 11), "Hack"));
    level.player_programs.push(Program::new(Point::new(5, 12), "Hack"));

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
