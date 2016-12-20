use voodoo::window::{Point, Window};

use game_state::{UiState, ModelView};

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
    Level(usize),
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

pub fn next(_state: &mut State, event: UiEvent, _mv: &mut ModelView) -> Transition {
    use self::UiEvent::*;
    match event {
        KeyPressed => Transition::Level(0),
        Tick => Transition::Ui(UiState::Unselected),
    }
}

pub fn display(mission_state: &mut State, compositor: &mut ::voodoo::compositor::Compositor, _mv: &mut ModelView) {
    for (offset, line) in TITLE.iter().enumerate() {
        mission_state.window.print_at(Point::new(13, 6 + offset as u16), *line);
    }
    mission_state.window.print_at(Point::new(30, 14), "PRESS ANY KEY TO BEGIN");
    mission_state.window.print_at(Point::new(33, 15), "PRESS Q TO QUIT");
    mission_state.window.refresh(compositor);
}
