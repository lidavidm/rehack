use voodoo::window::{Point, Window};

use data;
use game_state::{self, UiState, ModelView};
use info_view::{ChoiceList, InfoView};
use map_view::MapView;
use level::{CellContents, Level};
use player::Player;
use program::{Ability, Team};

const TITLE: [&'static str; 6] = [
    "██████╗ ███████╗    ██╗██╗  ██╗ █████╗  ██████╗██╗  ██╗",
    "██╔══██╗██╔════╝   ██╔╝██║  ██║██╔══██╗██╔════╝██║ ██╔╝",
    "█████╔╝█████╗    ██╔╝ ███████║███████║██║     █████╔╝",
    "██╔══██╗██╔══╝   ██╔╝  ██╔══██║██╔══██║██║     ██╔═██╗",
    "█║  ██║███████╗██╔╝   ██║  ██║██║  ██║╚██████╗██║  ██╗",
    "╚═╝  ╚═╝╚══════╝╚═╝    ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝",
];

pub enum UiEvent {
    KeyPressed,
    Tick,
}

pub enum Transition {
    Ui(UiState),
    Level(Level),
}

pub struct State {
    window: Window,
}

impl State {
    pub fn new(window: Window) -> State {
        State {
            window: window,
        }
    }
}

impl ::std::fmt::Debug for State {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "mission_select::State")
    }
}

pub fn next(mission_state: &mut State, event: UiEvent, mv: &mut ModelView) -> Transition {
    use self::UiEvent::*;
    match event {
        KeyPressed => Transition::Level(data::load_level(0)),
        Tick => Transition::Ui(UiState::Unselected),
    }
}

pub fn display(mission_state: &mut State, stdout: &mut ::std::io::Stdout, mv: &mut ModelView) {
    for (offset, line) in TITLE.iter().enumerate() {
        mission_state.window.print_at(Point::new(13, 6 + offset as u16), *line);
    }
    mission_state.window.print_at(Point::new(30, 14), "PRESS ANY KEY TO BEGIN");
    mission_state.window.refresh(stdout);
}
