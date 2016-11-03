extern crate termion;
extern crate voodoo;

use std::io::{Write};

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead};
use voodoo::window::{Point, TermCell, Window};

const LEVEL_DESCR: [&'static str; 22] = [
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "          ..                                              ",
    "          ..                                              ",
    "          ..                                              ",
    "          ..                                              ",
    "          ..                                              ",
    "          ..                                              ",
    "   o................                                      ",
    "   o................                                      ",
    "          ..                                              ",
    "          ..                                              ",
    "          ..                                              ",
    "          ..                                              ",
    "          ..                                              ",
    "          .......................                         ",
    "          .......................                         ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
];

struct Program {
    position: Point,
}

impl Program {
    fn new(position: Point) -> Program {
        Program {
            position: position,
        }
    }

    fn render(&self) -> (Point, TermCell) {
        (self.position, '◘'.into())
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

    fn display_for(&self, y: usize, x: usize) -> Option<TermCell> {
        Self::convert(self.layout[y].chars().nth(x).unwrap())
    }

    // TODO: need char -> Tile -> DisplayChar

    fn convert(c: char) -> Option<TermCell> {
        match c {
            '.' => Some('·'.into()),
            'o' => Some('O'.into()),
            _ => None,
        }
    }

    fn display(&self, map: &mut Window) {
        for (y, line) in self.layout.iter().enumerate() {
            let y = y + 1;
            for (x, tile) in line.chars().enumerate() {
                let x = x + 1;
                match Self::convert(tile) {
                    Some(c) => map.put_at(Point::new(x as u16, y as u16), c),
                    None => {},
                }
            }
        }

        for program in self.player_programs.iter() {
            let (point, c) = program.render();
            map.put_at(point, c);
        }
    }
}

enum UiState {
    Unselected,
    Selected,
}

enum UiEvent {
    Click(Point),
}

enum GameState {
    Setup,
    PlayerTurn,
    AITurn,
    AIExecute,
}

struct UiModelView {

}

impl UiState {
    fn next(self, event: UiEvent, level: &mut Level// , info: &mut Window, map: &mut Window
    ) -> UiState {
        use UiEvent::*;
        use UiState::*;

        match (self, event) {
            (Unselected, Click(p)) => {
                for program in level.player_programs.iter() {
                    if intersects(&program, p) {
                        // let (y, x, c) = program.render();
                        return Selected;
                    }
                }
                Unselected
            }
            (Selected, Click(_)) => {
                Unselected
            }
        }
    }
}

fn intersects(program: &Program, point: Point) -> bool {
    program.position == point
}

fn main() {
    use voodoo::color::ColorValue;
    use voodoo::terminal::{Mode, Terminal};
    let mut level = Level::new(&LEVEL_DESCR);
    level.player_programs.push(Program::new(Point::new(4, 4)));

    let mut terminal = Terminal::new();
    terminal.cursor(Mode::Disabled);
    terminal.clear_color(ColorValue::Black);
    let Terminal { ref mut stdin, ref mut stdout } = terminal;

    stdout.flush().unwrap();

    let mut info = voodoo::window::Window::new(Point::new(0, 0), 20, 24);
    let mut map = voodoo::window::Window::new(Point::new(20, 0), 60, 24);
    info.border();
    map.border();
    level.display(&mut map);
    info.refresh(stdout);
    map.refresh(stdout);

    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(Key::Char('q')) => break,
            Event::Mouse(me) => {
                match me {
                    MouseEvent::Press(_, x, y) => {
                        if let Some(p) = map.position.from_global_frame(Point::new(x, y)) {
                            for program in level.player_programs.iter() {
                                if intersects(program, p) {
                                    let (_, mut tc) = program.render();
                                    tc.bg = Some(ColorValue::Blue);
                                    map.put_at(p, tc);
                                }
                            }
                        }
                    },
                    _ => (),
                }
            }
            _ => {}
        }
        info.refresh(stdout);
        map.refresh(stdout);
    }
 }
