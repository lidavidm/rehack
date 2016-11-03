extern crate ncurses;
extern crate voodoo;

use ncurses::*;
use voodoo::terminal::{Mode, Terminal};
use voodoo::window::{DisplayChar, Window, WindowLike};

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

struct Level {
    layout: Vec<String>,
}

impl Level {
    fn display_for(&self, y: usize, x: usize) -> Option<DisplayChar> {
        Self::convert(self.layout[y].chars().nth(x).unwrap())
    }

    // TODO: need char -> Tile -> DisplayChar

    fn convert(c: char) -> Option<DisplayChar> {
        match c {
            '.' => Some(Into::<DisplayChar>::into(ACS_BULLET()).dim()), // '·'
            'o' => Some(Into::<DisplayChar>::into('O')),
            _ => None,
        }
    }

    fn display(&self, map: &mut Window) {
        for (y, line) in self.layout.iter().enumerate() {
            let y = y + 1;
            for (x, tile) in line.chars().enumerate() {
                let x = x + 1;
                match tile {
                    '.' => map.put_at(y as i32, x as i32, Into::<DisplayChar>::into(ACS_BULLET()).dim()), // '·'
                    'o' => map.put_at(y as i32, x as i32, 'O'),
                    _ => {}
                }
            }
        }
    }
}

fn main() {
    let mut layout = Vec::new();
    for s in LEVEL_DESCR.iter() {
        layout.push(s.to_string());
    }
    let level = Level {
        layout: layout,
    };

    // let locale = LcCategory::all;
    // setlocale(locale, "en_US.UTF-8");

    let term = Terminal::new();
    term.cbreak(Mode::Enabled).unwrap();
    term.echo(Mode::Disabled).unwrap();

    keypad(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    mousemask((ALL_MOUSE_EVENTS | REPORT_MOUSE_POSITION) as u32, None);

    start_color();
    init_pair(1, COLOR_BLACK, COLOR_WHITE);

    wbkgd(stdscr(), 1);

    refresh();

    let mut info = Window::new(0, 0, 20, 24);
    let mut map = Window::new(20, 0, 60, 24);
    info.box_(0, 0);
    map.box_(0, 0);

    level.display(&mut map);

    info.refresh();
    map.refresh();

    print!("\x1B[?1003h\n"); // Makes the terminal report mouse movement events

    loop {
        match voodoo::poll_event() {
            Some(voodoo::Event::Mouse) => {
                let event = get_mouse_state();
                let x = event.x - 20;
                let y = event.y - 1;
                if y <= 0 || y >= 59 || x <= 0 || x >= 19 {

                }
                else if let Some(c) = level.display_for(event.y as usize - 1, event.x as usize - 21) {
                    map.put_at(event.y, event.x - 20, c.bold());
                    map.refresh();
                }
            }

            Some(voodoo::Event::Char('\n')) => {
                break;
            }

            _ => {}
        }
    }

    print!("\x1B[?1003l\n"); // Disable mouse movement events, as l = low
    let a = ACS_BULLET();
    // initscr();
    endwin();
    println!("{}", a);
    // ncurses::constants::acsmap();
}

fn get_mouse_state() -> MEVENT {
    let mut event = MEVENT { id: 0, x: 0, y: 0, z: 0, bstate: 0 };
    let res = getmouse(&mut event);
    if res != 0 {
        panic!("getmouse");
    }
    event
}
