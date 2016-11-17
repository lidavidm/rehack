use game_state::{self, UiEvent, UiState, ModelView};
use info_view::{ChoiceList, InfoView};
use map_view::MapView;
use level::{CellContents, Level};
use player::Player;
use program::{Ability, Team};

pub enum Transition {
    Ui(UiState),
    Level(Level),
    // Augument
}

#[derive(Debug)]
enum Menu {
    Main,
    MissionSelect,
}

pub struct State {
    main_menu: ChoiceList<String>, // missions, augument, logoff
}

impl ::std::fmt::Debug for State {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "mission_select::State")
    }
}

pub fn next(mission_state: &mut State, ui_state: UiState, event: UiEvent, mv: &mut ModelView) -> Transition {
    use game_state::UiEvent::*;
    use game_state::UiState::*;

    let ModelView { ref mut info, ref mut map, ref mut player, .. } = *mv;

    let result = match (ui_state, event) {
        _ => unimplemented!(),
    };

    Transition::Ui(result)
}

pub fn display(mission_state: &mut State, stdout: &mut ::std::io::Stdout, mv: &mut ModelView) {
}
