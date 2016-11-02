extern crate ncurses;
extern crate voodoo;

use ncurses::*;
use voodoo::window::WindowLike;

const level: [&'static str; 22] = [
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "   o....                                                  ",
    "   o....                                                  ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
];

fn main() {
    use voodoo::terminal::{Mode, Terminal};
    use voodoo::window::Window;

    let locale = LcCategory::all;
    setlocale(locale, "en_US.UTF-8");

    let term = Terminal::new();
    term.cbreak(Mode::Enabled).unwrap();
    term.echo(Mode::Disabled).unwrap();

    keypad(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    mousemask((ALL_MOUSE_EVENTS | REPORT_MOUSE_POSITION) as u64, None);

    start_color();
    init_pair(1, COLOR_BLACK, COLOR_WHITE);

    wbkgd(stdscr(), 1);

    refresh();

    let mut info = Window::new(0, 0, 20, 24);
    let mut map = Window::new(20, 0, 60, 24);
    info.box_(0, 0);
    map.box_(0, 0);

    for (y, line) in level.iter().enumerate() {
        let y = (y + 1) as i32;
        for (x, tile) in line.chars().enumerate() {
            let x = (x + 1) as i32;
            match tile {
                '.' => map.put_at(y, x, '.'), // '·'
                'o' => map.put_at(y, x, 'O'),
                _ => {}
            }
        }
    }

    info.refresh();
    map.refresh();

    print!("\x1B[?1003h\n"); // Makes the terminal report mouse movement events

    loop {
        match voodoo::poll_event() {
            Some(voodoo::Event::Mouse) => {
                let event = get_mouse_state();
                map.put_at(event.y, event.x - 20, 'y');
                map.refresh();
            }

            Some(voodoo::Event::Char('\n')) => {
                break;
            }

            _ => {}
        }
    }

    print!("\x1B[?1003l\n"); // Disable mouse movement events, as l = low

    endwin();
}

fn get_mouse_state() -> MEVENT {
    let mut event = MEVENT { id: 0, x: 0, y: 0, z: 0, bstate: 0 };
    let res = getmouse(&mut event);
    if res != 0 {
        panic!("getmouse");
    }
    event
}
