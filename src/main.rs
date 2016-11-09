extern crate termion;
extern crate thread_scoped;
extern crate time;
extern crate voodoo;

mod ai;
mod info_view;
mod map_view;
mod level;
mod program;

use std::io::{Write};
use std::sync::mpsc::channel;

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead};

use voodoo::color::ColorValue;
use voodoo::window::{Point};

use info_view::InfoView;
use map_view::MapView;
use level::Level;
use program::{Ability, Program, ProgramRef, StatusEffect, Team};

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

#[derive(Clone,Copy,Debug)]
enum UiState {
    Unselected,
    Selected,
    SelectTarget(Ability),
    // Damage(ProgramRef, usize),
    Animating,
}

enum UiEvent {
    Quit,
    Tick,
    ClickMap(Point),
    ClickInfo(Point),
    EndTurn,
}

enum GameState {
    // Setup,
    PlayerTurn,
    AITurnTransition,
    AITurn,
    // AIExecute,
    Quit,
}

struct UiModelView {
    info: InfoView,
    map: MapView,
}

impl UiState {
    fn select_program(point: Point, level: &Level, map: &mut MapView, info: &mut InfoView) -> UiState {
        use UiState::*;

        for program in level.programs.iter() {
            if program.borrow().intersects(point) && program.borrow().team == Team::Player {
                map.highlight(program.clone(), &level);
                info.display_program(&program.borrow());
                map.set_help("Click arrows to move; click ability at left to use");
                return Selected;
            }
        }
        Unselected
    }

    fn next(self, event: UiEvent, level: &mut Level, mv: &mut UiModelView) -> UiState {
        use UiEvent::*;
        use UiState::*;

        let UiModelView { ref mut info, ref mut map } = *mv;

        let result = match (self, event) {
            (Unselected, ClickMap(p)) => {
                Self::select_program(p, level, map, info)
            }
            (Selected, ClickMap(p)) => {
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
                    Self::select_program(p, level, map, info)
                }
            }
            (Unselected, ClickInfo(_)) => Unselected,
            (Selected, ClickInfo(p)) => {
                if let Some(ability) = info.translate_click(p) {
                    match ability {
                        Ability::Destroy { damage, range } => {
                            map.set_help(format!("Select target. Damage: 0x{:x} Range: 0x{:x}", damage, range));
                            map.highlight_range(range, level);
                        }
                    }
                    return SelectTarget(ability);
                }
                Selected
            }
            (SelectTarget(ability), ClickMap(p)) => {
                let result = map.translate_click(p);
                info.clear_ability();
                map.clear_range();
                map.update_highlight(level);
                if let Some(p) = result {
                    match level.contents_of(p) {
                        level::CellContents::Program(p) => {
                            ability.apply(&mut p.borrow_mut());
                            Animating
                        },
                        _ => Selected,
                    }
                }
                else {
                    Selected
                }
            }
            (SelectTarget(_), ClickInfo(p)) => {
                let result = info.translate_click(p);
                if let Some(ability) = result {
                    // TODO: refactor this out
                    match ability {
                        Ability::Destroy { damage, range } => {
                            map.set_help(format!("Select target. Damage: 0x{:x} Range: 0x{:x}", damage, range));
                            map.highlight_range(range, level);
                        }
                    }
                    SelectTarget(ability)
                }
                else {
                    info.clear_ability();
                    map.clear_range();
                    map.update_highlight(level);
                    Selected
                }
            }
            (state, Tick) => {
                let modified = update_programs(level, map);

                match state {
                    Animating => {
                        if !modified {
                            if map.get_highlight().is_some() {
                                Selected
                            }
                            else {
                                info.clear();
                                Unselected
                            }
                        }
                        else {
                            state
                        }
                    }
                    _ => state,
                }
            }
            (Animating, _) => Animating,
            (state, Quit) | (state, EndTurn) => { state },
        };

        if let Unselected = result {
            map.set_help("Click program to control it");
        }

        result
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
            (&State(PlayerTurn, _), Event::Mouse(MouseEvent::Press(_, x, y))) => {
                if let Some(p) = mv.ui_modelview.map.from_global_frame(Point::new(x, y)) {
                    Some(UiEvent::ClickMap(p))
                }
                else if let Some(p) = mv.ui_modelview.info.from_global_frame(Point::new(x, y)) {
                    if p.y == 23 {
                        Some(UiEvent::EndTurn)
                    }
                    else {
                        Some(UiEvent::ClickInfo(p))
                    }
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
            match (self, event) {
                (State(PlayerTurn, ui), UiEvent::EndTurn) => State(AITurnTransition, UiState::Unselected),
                (State(PlayerTurn, ui), event) => Self::next_player_turn(ui, event, level, mv),
                (State(AITurnTransition, ui), _) => State(AITurnTransition, ui),
                (State(AITurn, ui), _) => State(AITurn, ui),
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
            State(AITurnTransition, _) => {
                begin_turn(level, mv);
                State(AITurn, UiState::Unselected)
            }
            State(AITurn, UiState::Animating) => {
                let modified = update_programs(level, &mut mv.ui_modelview.map);

                if !modified {
                    State(PlayerTurn, UiState::Unselected)
                }
                else {
                    State(AITurn, UiState::Unselected)
                }
            }
            State(AITurn, _) => {
                ai::ai_tick(level, &mut mv.ui_modelview.map);
                State(AITurn, UiState::Animating)
            },
            _ => unimplemented!(),
        }
    }

    fn next_player_turn(ui_state: UiState, event: UiEvent, level: &mut Level, mv: &mut GameModelView) -> State {
        use GameState::*;
        match event {
            UiEvent::Quit => State(Quit, ui_state),
            click@UiEvent::ClickMap(_) => {
                State(PlayerTurn, ui_state.next(click, level, &mut mv.ui_modelview))
            }
            click@UiEvent::ClickInfo(_) => {
                State(PlayerTurn, ui_state.next(click, level, &mut mv.ui_modelview))
            }
            UiEvent::Tick => {
                State(PlayerTurn, ui_state.next(event, level, &mut mv.ui_modelview))
            }
            UiEvent::EndTurn => unreachable!(),
        }
    }
}

fn begin_turn(level: &mut Level, mv: &mut GameModelView) {
    mv.ui_modelview.info.clear();
    mv.ui_modelview.map.clear_range();
    mv.ui_modelview.map.clear_highlight();
    mv.ui_modelview.map.update_highlight(level);
    level.begin_turn();
}

fn update_programs(level: &mut Level, map: &mut MapView) -> bool {
    let mut modified = false;
    let mut killed = vec![];
    for program in level.programs.iter_mut() {
        let mut p = program.borrow_mut();
        let position = p.position;
        let mut damaged = false;
        for effect in p.status_effects.iter_mut() {
            match *effect {
                StatusEffect::Damage(damage) => {
                    modified = true;
                    damaged = true;
                    *effect = StatusEffect::Damage(damage - 1);
                }
            }
        }
        p.status_effects.retain(|effect| {
            match *effect {
                StatusEffect::Damage(0) => false,
                StatusEffect::Damage(_) => true,
            }
        });

        if damaged {
            let lived = p.damage();
            if !lived {
                killed.push(position);
                map.clear_highlight();
            }
        }
    }

    for position in killed {
        level.remove_program_at(position);
    }

    modified
}

fn main() {
    use std::sync::mpsc::TryRecvError::*;
    use std::thread;
    use std::time::Duration;

    use voodoo::terminal::{Mode, Terminal};

    let mut level = Level::new(&LEVEL_DESCR);
    level.add_program(Program::new(Team::Player, Point::new(11, 9), "Hack"));
    level.add_program(Program::new(Team::Player, Point::new(5, 10), "Hack"));
    level.add_program(Program::new(Team::Enemy, Point::new(7, 10), "Hack"));

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

    let ui_modelview = UiModelView {
        info: info_view,
        map: map_view,
    };
    let ui_state = UiState::Unselected;

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
