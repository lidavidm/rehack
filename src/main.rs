extern crate ncurses;

use std::io::Write;

use ncurses::*;

fn main() {
    let locale = LcCategory::all;
    setlocale(locale, "en_US.UTF-8");

    initscr();
    cbreak();
    noecho();

    keypad(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();

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
        match poll_event() {
            Some(WchResult::KeyCode(KEY_MOUSE)) => {
                let event = get_mouse_state();
                mvwprintw(map, event.y, event.x - 20, "y");
                wrefresh(map);
            }

            Some(WchResult::Char(10)) => {
                break;
            }

            Some(WchResult::Char(a)) => {
                // mvwprintw(map, 0, 0, &keyname(a as i32));
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

pub fn poll_event() -> Option<WchResult> {
    // Can't poll non-root screen for key events - it doesn't work
    // anymore (dead link:
    // http://blog.chris.tylers.info/index.php?/archives/212-Using-the-Mouse-with-ncurses.html)
    // Need to call refresh() or getch() will call it for us, clearing
    // the screen
    match getch() {
        ERR => {
            None
        }
        v => {
            if v >= KEY_MIN {
                Some(WchResult::KeyCode(v))
            }
            else {
                Some(WchResult::Char(v as u32))
            }
        }
    }
}
