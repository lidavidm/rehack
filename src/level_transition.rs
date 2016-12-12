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
    level_index: usize,
    window: Window,
    winning_team: Team,
}

impl State {
    pub fn new(level_index: usize, winning_team: Team) -> State {
        State {
            level_index: level_index,
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

pub fn next(state: &mut State, event: UiEvent, _mv: &mut ModelView) -> Option<usize> {
    use self::UiEvent::*;
    match event {
        KeyPressed => {
            match state.winning_team {
                Team::Player => Some(state.level_index + 1),
                Team::Enemy => Some(state.level_index),
            }
        },
        Tick => None,
    }
}

pub fn display(state: &mut State, compositor: &mut ::voodoo::compositor::Compositor, _mv: &mut ModelView) {
    let (string, size, message) = match state.winning_team {
        Team::Player => (VICTORY, 55, "PRESS ANY KEY TO CONTINUE"),
        Team::Enemy => (DEFEAT, 49, "PRESS ANY KEY TO RETRY"),
    };
    let left_offset = (80 - size) / 2;
    for (offset, line) in string.iter().enumerate() {
        state.window.print_at(Point::new(left_offset, 6 + offset as u16), *line);
    }

    state.window.print_at(Point::new(30, 14), message);
    state.window.refresh(compositor);
}
