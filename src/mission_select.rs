use game_state::{self, UiEvent, UiState, ModelView};
use info_view::InfoView;
use map_view::MapView;
use level::{CellContents, Level};
use player::Player;
use program::{Ability, Team};

pub fn next(state: UiState, event: UiEvent, level: &mut Level, mv: &mut ModelView) -> UiState {
    use game_state::UiEvent::*;
    use game_state::UiState::*;

    let ModelView { ref mut info, ref mut map, ref mut player, .. } = *mv;

    let result = match (state, event) {
        _ => unimplemented!(),
    };

    result
}
