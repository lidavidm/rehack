extern crate termion;
extern crate thread_scoped;
extern crate time;
extern crate voodoo;

mod ai;
mod game_state;
mod info_view;
mod map_view;
mod mission_select;
mod level;
mod player;
mod player_turn;
mod program;

use std::io::{Write};
use std::sync::mpsc::channel;

use termion::event::{Key, Event};
use termion::input::{TermRead};

use voodoo::color::ColorValue;
use voodoo::window::{Point};

use info_view::InfoView;
use map_view::MapView;
use level::Level;
use player::Player;

const LEVEL_DESCR: [&'static str; 20] = [
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

const MS: u64 = 1_000_000;
const TICK_TIME: u64 = 250;

fn main() {
    use std::sync::mpsc::TryRecvError::*;
    use std::thread;
    use std::time::Duration;

    use game_state::{ModelView, GameState, State, UiState};

    use voodoo::terminal::{Mode, Terminal};

    let mut level = Level::new(&LEVEL_DESCR);

    let mut terminal = Terminal::new();
    terminal.cursor(Mode::Disabled);
    terminal.clear_color(ColorValue::Black);
    let Terminal { ref mut stdin, ref mut stdout } = terminal;

    stdout.flush().unwrap();

    let info = voodoo::window::Window::new(Point::new(0, 0), 20, 24);
    let map = voodoo::window::Window::new(Point::new(20, 0), 60, 24);
    let mut info_view = InfoView::new(info);
    let mut map_view = MapView::new(map);
    let player = Player::new("David");

    info_view.refresh(stdout);
    map_view.refresh(stdout);

    let mut mv = ModelView {
        info: info_view,
        map: map_view,
        player: player,
        program_list: info_view::ChoiceList::new(4),
        level: level,
    };
    let ui_state = UiState::Unselected;

    let mut state = State(GameState::SetupTransition, ui_state);

    let (tx, rx) = channel();
    let guard = unsafe {
        thread_scoped::scoped(move || {
            for c in stdin.events() {
                let evt = c.unwrap();
                if let Event::Key(Key::Char('q')) = evt {
                    break;
                }
                tx.send(evt).unwrap();
            }
        })
    };

    let mut t = time::precise_time_ns();
    let mut dt = 0;

    'main: loop {
        loop {
            // Handle all pending events
            let msg = rx.try_recv();
            match msg {
                Ok(evt) => {
                    state = state.next(evt, &mut mv);
                    if let State(GameState::Quit, _) = state {
                        break 'main;
                    }
                },
                Err(Disconnected) => break 'main,
                Err(Empty) => break,
            }
        }

        let now = time::precise_time_ns();
        dt += now - t;

        while dt >= TICK_TIME * MS {
            state = state.tick(&mut mv);
            if let State(GameState::Quit, _) = state {
                break 'main;
            }
            dt -= TICK_TIME * MS;
        }

        mv.info.refresh(stdout);
        mv.map.display(&mv.level);
        mv.map.refresh(stdout);
        t = now;

        thread::sleep(Duration::from_millis((TICK_TIME - dt / MS) / 2));
    }
    guard.join();
}
