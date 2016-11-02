extern crate ncurses;
extern crate voodoo;

use ncurses::*;

fn main() {
    use voodoo::terminal::{Mode, Terminal};

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

    let info = newwin(24, 20, 0, 0);
    let map = newwin(24, 60, 0, 20);

    box_(info, 0, 0);
    box_(map, 0, 0);

    refresh();
    wrefresh(info);
    wrefresh(map);

    print!("\x1B[?1003h\n"); // Makes the terminal report mouse movement events

    loop {
        match voodoo::poll_event() {
            Some(voodoo::Event::Mouse) => {
                let event = get_mouse_state();
                mvwprintw(map, event.y, event.x - 20, "y");
                wrefresh(map);
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
