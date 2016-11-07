extern crate termion;
extern crate thread_scoped;
extern crate time;
extern crate voodoo;

mod info_view;
mod map_view;
mod level;
mod program;

use std::io::{Write};
use std::sync::mpsc::channel;

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead};

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

use info_view::InfoView;
use map_view::MapView;
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
    Damage(ProgramRef, usize),
}

enum UiEvent {
    Quit,
    Tick,
    Test,
    Click(Point),
    // Movement
}

enum GameState {
    Setup,
    PlayerTurn,
    AITurn,
    AIExecute,
    Quit,
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
                    if program.borrow().intersects(p) {
                        map.highlight(program.clone(), &level);
                        info.display_program(&program.borrow());
                        return Selected;
                    }
                }
                Unselected
            }
            (Selected, Click(p)) => {
                let result = map.translate_click(p);
                if let Some(p) = result {
                    if let Some(ref mut program) = map.get_highlight() {
                        program.borrow_mut().move_to(p);
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
            (Damage(program, damage), Tick) => {
                if damage == 0 {
                    if map.get_highlight().is_some() {
                        Selected
                    }
                    else {
                        info.clear();
                        Unselected
                    }
                }
                else {
                    let new_ref = program.clone();
                    let position = { program.borrow().position };
                    let lived = { program.borrow_mut().damage() };
                    if lived {
                        Damage(new_ref, damage - 1)
                    }
                    else {
                        level.remove_program_at(position);
                        map.clear_highlight();
                        Damage(new_ref, 0)
                    }
                }
            }
            (Damage(program, damage), _) => {
                Damage(program, damage)
            }
            (state, Tick) => { state }
            (Selected, Test) => {
                if let Some(ref program) = map.get_highlight() {
                    Damage(program.clone(), 2)
                }
                else {
                    Selected
                }
            }
            (state, Test) => { state },
            (state, Quit) => { state },
        }
    }
}

struct GameModelView {
    ui_modelview: UiModelView,
}

struct State(GameState, UiState);

impl State {
    fn translate_event(&self, event: termion::event::Event, level: &mut Level, mv: &mut GameModelView) -> Option<UiEvent> {
        use GameState::*;
        match (self, event) {
            (_, Event::Key(Key::Char('q'))) => Some(UiEvent::Quit),
            (&State(PlayerTurn, _), Event::Key(Key::Char(' '))) => Some(UiEvent::Test),
            (&State(PlayerTurn, _), Event::Mouse(MouseEvent::Press(_, x, y))) => {
                if let Some(p) = mv.ui_modelview.map.from_global_frame(Point::new(x, y)) {
                    Some(UiEvent::Click(p))
                }
                else {
                    None
                }
            }
            _ => None,
        }
    }

    fn next(self, event: termion::event::Event, level: &mut Level, mv: &mut GameModelView) -> State {
        use GameState::*;
        if let Some(event) = self.translate_event(event, level, mv) {
            match self {
                State(PlayerTurn, ui) => Self::next_player_turn(ui, event, level, mv),
                _ => unimplemented!(),
            }
        }
        else {
            self
        }
    }

    fn tick(self, level: &mut Level, mv: &mut GameModelView) -> State {
        use GameState::*;
        match self {
            State(PlayerTurn, ui) => Self::next_player_turn(ui, UiEvent::Tick, level, mv),
            _ => unimplemented!(),
        }
    }

    fn next_player_turn(ui_state: UiState, event: UiEvent, level: &mut Level, mv: &mut GameModelView) -> State {
        use GameState::*;
        match event {
            UiEvent::Quit => State(Quit, ui_state),
            click@UiEvent::Click(_) => {
                State(PlayerTurn, ui_state.next(click, level, &mut mv.ui_modelview))
            }
            UiEvent::Tick | UiEvent::Test => {
                State(PlayerTurn, ui_state.next(event, level, &mut mv.ui_modelview))
            }
        }
    }
}

fn main() {
    use std::sync::mpsc::TryRecvError::*;
    use std::thread;
    use std::time::Duration;

    use voodoo::terminal::{Mode, Terminal};

    let mut level = Level::new(&LEVEL_DESCR);
    level.add_player_program(Program::new(Point::new(11, 11), "Hack"));
    level.add_player_program(Program::new(Point::new(5, 12), "Hack"));
    level.add_enemy_program(Program::new(Point::new(7, 12), "Hack"));

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

    let mut state = State(GameState::PlayerTurn, ui_state);
    let mut mv = GameModelView {
        ui_modelview: ui_modelview,
    };

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
                    state = state.next(evt, &mut level, &mut mv);
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

        // TODO: use constant
        while dt >= 100000000 {
            state = state.tick(&mut level, &mut mv);
            dt -= 100000000;
        }

        mv.ui_modelview.info.refresh(stdout);
        mv.ui_modelview.map.display(&level);
        mv.ui_modelview.map.refresh(stdout);
        t = now;

        thread::sleep(Duration::from_millis(100 - dt / 1000000));
    }
    guard.join();
}
