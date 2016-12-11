use voodoo::window::{Point, Window};

use data;
use game_state::{UiState, ModelView};
use level::Level;
use program::Team;

const DEFEAT: [&'static str; 6] = [
    "██████╗ ███████╗███████╗███████╗ █████╗ ████████╗",
    "██╔══██╗██╔════╝██╔════╝██╔════╝██╔══██╗╚══██╔══╝",
    "██║  ██║█████╗  █████╗  █████╗  ███████║   ██║   ",
    "██║  ██║██╔══╝  ██╔══╝  ██╔══╝  ██╔══██║   ██║   ",
    "██████╔╝███████╗██║     ███████╗██║  ██║   ██║   ",
    "╚═════╝ ╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ",
];

const VICTORY: [&'static str; 6] = [
    "██╗   ██╗██╗ ██████╗████████╗ ██████╗ ██████╗ ██╗   ██╗",
    "██║   ██║██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗╚██╗ ██╔╝",
    "██║   ██║██║██║        ██║   ██║   ██║██████╔╝ ╚████╔╝ ",
    "╚██╗ ██╔╝██║██║        ██║   ██║   ██║██╔══██╗  ╚██╔╝  ",
    " ╚████╔╝ ██║╚██████╗   ██║   ╚██████╔╝██║  ██║   ██║   ",
    "  ╚═══╝  ╚═╝ ╚═════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ",
];

pub enum UiEvent {
    KeyPressed,
    Tick,
}

pub struct State {
    window: Window,
    winning_team: Team,
}

impl State {
    pub fn new(winning_team: Team) -> State {
        State {
            window: Window::new(Point::new(0, 0), 80, 24),
            winning_team: winning_team,
        }
    }
}

impl ::std::fmt::Debug for State {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "level_transition::State")
    }
}

pub fn next(_state: &mut State, event: UiEvent, _mv: &mut ModelView) -> Option<Level> {
    use self::UiEvent::*;
    match event {
        KeyPressed => Some(data::load_level(0)),
        Tick => None,
    }
}

pub fn display(state: &mut State, compositor: &mut ::voodoo::compositor::Compositor, _mv: &mut ModelView) {
    let (string, size) = match state.winning_team {
        Team::Player => (VICTORY, 55),
        Team::Enemy => (DEFEAT, 49),
    };
    let left_offset = (80 - size) / 2;
    for (offset, line) in string.iter().enumerate() {
        state.window.print_at(Point::new(left_offset, 6 + offset as u16), *line);
    }

    state.window.print_at(Point::new(30, 14), "PRESS ANY KEY TO CONTINUE");
    state.window.refresh(compositor);
}
