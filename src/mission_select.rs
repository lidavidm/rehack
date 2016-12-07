use game_state::{self, UiEvent, UiState, ModelView};
use info_view::{ChoiceList, InfoView};
use map_view::MapView;
use level::{CellContents, Level};
use player::Player;
use program::{Ability, Team};

use voodoo::window::{Point, Window};

const TITLE: [&'static str; 6] = [
    "██████╗ ███████╗    ██╗██╗  ██╗ █████╗  ██████╗██╗  ██╗",
    "██╔══██╗██╔════╝   ██╔╝██║  ██║██╔══██╗██╔════╝██║ ██╔╝",
    "█████╔╝█████╗    ██╔╝ ███████║███████║██║     █████╔╝",
    "██╔══██╗██╔══╝   ██╔╝  ██╔══██║██╔══██║██║     ██╔═██╗",
    "█║  ██║███████╗██╔╝   ██║  ██║██║  ██║╚██████╗██║  ██╗",
    "╚═╝  ╚═╝╚══════╝╚═╝    ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝",
];

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

pub fn next(mission_state: &mut State, ui_state: UiState, event: UiEvent, mv: &mut ModelView) -> Transition {
    use game_state::UiEvent::*;
    use game_state::UiState::*;

    let result = match (ui_state, event) {
        _ => Unselected,
    };

    Transition::Ui(result)
}

pub fn display(mission_state: &mut State, stdout: &mut ::std::io::Stdout, mv: &mut ModelView) {
    for (offset, line) in TITLE.iter().enumerate() {
        mission_state.window.print_at(Point::new(13, 6 + offset as u16), *line);
    }
    mission_state.window.print_at(Point::new(30, 14), "PRESS ANY KEY TO BEGIN");
    mission_state.window.refresh(stdout);
}
