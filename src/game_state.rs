use termion;
use termion::event::{Key, Event, MouseEvent};

use voodoo;
use voodoo::color::{ColorValue};
use voodoo::window::{Point};

use ai;
use info_view::{self, InfoView};
use map_view::MapView;
use mission_select;
use level::Level;
use player::Player;
use player_turn;
use program::{Ability, Program, StatusEffect, Team};


#[derive(Clone,Copy,Debug)]
pub enum UiState {
    Unselected,
    Selected,
    SelectTarget(Ability),
    Animating,
}

#[derive(Clone,Copy,Debug)]
pub enum UiEvent {
    Quit,
    Tick,
    ClickMap(Point),
    ClickInfo(Point),
    EndTurn,
}

#[derive(Debug)]
pub enum GameState {
    Setup,
    PlayerTurn,
    AITurn,
    SetupTransition,
    AITurnTransition,
    PlayerTurnTransition,
    Quit,
    MissionSelect(mission_select::State),
}

pub struct ModelView {
    pub info: InfoView,
    pub map: MapView,
    pub player: Player,
    pub program_list: info_view::ChoiceList<Program>,
    pub level: Level,
}

#[derive(Debug)]
pub struct State(pub GameState, pub UiState);

impl State {
    pub fn translate_event(&self, event: Event, mv: &mut ModelView) -> Option<UiEvent> {
        use self::GameState::*;

        match (self, event) {
            (_, Event::Key(Key::Char('q'))) => Some(UiEvent::Quit),
            (&State(PlayerTurn, _), Event::Mouse(MouseEvent::Press(_, x, y))) |
            (&State(Setup, _), Event::Mouse(MouseEvent::Press(_, x, y))) => {
                if let Some(p) = mv.map.from_global_frame(Point::new(x, y)) {
                    Some(UiEvent::ClickMap(p))
                }
                else if let Some(p) = mv.info.from_global_frame(Point::new(x, y)) {
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

    pub fn next(self, event: termion::event::Event, mv: &mut ModelView) -> State {
        use self::GameState::*;

        if let Some(event) = self.translate_event(event, mv) {
            match self {
                State(Setup, ui) => Self::next_setup_turn(ui, event, mv),
                State(PlayerTurn, ui) => match event {
                    UiEvent::EndTurn => {
                        if mv.level.check_victory().is_some() {
                            State(Quit, UiState::Unselected)
                        }
                        else {
                            State(AITurnTransition, UiState::Unselected)
                        }
                    },
                    _ => Self::next_player_turn(ui, event, mv)
                },
                State(MissionSelect(_), _) => self,
                State(SetupTransition, _) |
                State(AITurnTransition, _) | State(PlayerTurnTransition, _) |
                State(AITurn, _) | State(Quit, _) => self,
            }
        }
        else {
            match (self, event) {
                (State(MissionSelect(ms), ui), Event::Key(_)) => Self::next_mission_turn(ms, ui, mission_select::UiEvent::KeyPressed, mv),
                (s, _) => s
            }
        }
    }

    pub fn tick(self, mv: &mut ModelView) -> State {
        use self::GameState::*;

        match self {
            State(Setup, ui) => Self::next_setup_turn(ui, UiEvent::Tick, mv),
            State(PlayerTurn, ui) => Self::next_player_turn(ui, UiEvent::Tick, mv),
            State(MissionSelect(ms), ui) => Self::next_mission_turn(ms, ui, mission_select::UiEvent::Tick, mv),
            State(AITurnTransition, _) => {
                begin_turn(Team::Enemy, mv);
                State(AITurn, UiState::Unselected)
            }
            State(PlayerTurnTransition, _) => {
                if mv.level.check_victory().is_some() {
                    State(Quit, UiState::Unselected)
                }
                else {
                    begin_turn(Team::Player, mv);
                    State(PlayerTurn, UiState::Unselected)
                }
            }
            State(AITurn, UiState::Animating) => {
                let modified = update_programs(&mut mv.level, &mut mv.map);

                if !modified {
                    State(AITurn, UiState::Unselected)
                }
                else {
                    State(AITurn, UiState::Animating)
                }
            }
            State(AITurn, _) => {
                let ai_state = ai::ai_tick(&mut mv.level, &mut mv.map);
                mv.map.set_help(format!("AI STATUS: {:?}", ai_state));
                match ai_state {
                    ai::AIState::Done => State(PlayerTurnTransition, UiState::Unselected),
                    ai::AIState::Plotting => State(AITurn, UiState::Unselected),
                    ai::AIState::WaitingAnimation => State(AITurn, UiState::Animating),
                }
            }
            State(SetupTransition, _) => {
                let mut enemy1 = Program::new(Team::Enemy, Point::new(7, 10), "Hack");
                enemy1.abilities.push(("Bitblast".to_owned(), Ability::Destroy { damage: 2, range: 1 }));
                let mut enemy2 = enemy1.clone();
                enemy2.position = Point::new(7, 9);

                mv.level.add_program(enemy1);
                mv.level.add_program(enemy2);

                let mut prog1 = Program::new(Team::Player, Point::new(0, 0), "Hack 1");
                prog1.abilities.push(("Bitblast".to_owned(), Ability::Destroy { damage: 2, range: 1 }));
                let mut prog2 = prog1.clone();
                prog2.name = "Hack 2".to_owned();
                mv.player.programs.push(prog1);
                mv.player.programs.push(prog2);

                mv.info.primary_action = ">Launch Intrusion<".to_owned();
                mv.info.clear();
                mv.map.display(&mv.level);
                mv.program_list.choices().extend(mv.player.programs.iter().map(|x| {
                    (x.name.to_owned(), x.clone())
                }));
                State(Setup, UiState::Unselected)
            }
            State(Quit, _) => self,
        }
    }

    pub fn display(&mut self, stdout: &mut ::std::io::Stdout, mv: &mut ModelView) {
        use self::GameState::*;

        match self.0 {
            MissionSelect(ref mut state) => {
                mission_select::display(state, stdout, mv);
            }
            _ => {
                mv.info.refresh(stdout);
                mv.map.display(&mv.level);
                mv.map.refresh(stdout);
            }
        }
    }

    pub fn next_player_turn(ui_state: UiState, event: UiEvent, mv: &mut ModelView) -> State {
        use self::GameState::*;

        match event {
            UiEvent::ClickMap(_) | UiEvent::ClickInfo(_) | UiEvent::Tick => {
                State(PlayerTurn, player_turn::next(ui_state, event, mv))
            }
            UiEvent::EndTurn | UiEvent::Quit => unreachable!(),
        }
    }

    pub fn next_mission_turn(mut mission_state: mission_select::State, ui_state: UiState, event: mission_select::UiEvent, mv: &mut ModelView) -> State {
        use self::GameState::*;

        match mission_select::next(&mut mission_state, ui_state, event, mv) {
            mission_select::Transition::Ui(ui) => State(MissionSelect(mission_state), ui),
            mission_select::Transition::Level(level) => {
                voodoo::terminal::clear_color(ColorValue::Black);
                mv.level = level;
                State(SetupTransition, UiState::Unselected)
            }
        }
    }

    pub fn next_setup_turn(ui_state: UiState, event: UiEvent, mv: &mut ModelView) -> State {
        use self::GameState::*;

        match event {
            UiEvent::ClickMap(_) | UiEvent::ClickInfo(_) | UiEvent::Tick => {
                State(Setup, player_turn::next_setup(ui_state, event, mv))
            }
            UiEvent::EndTurn => {
                // TODO: reset
                mv.info.primary_action = ">    End Turn    <".to_owned();
                State(PlayerTurnTransition, UiState::Unselected)
            }
            UiEvent::Quit => unreachable!(),
        }
    }
}

pub fn begin_turn(team: Team, mv: &mut ModelView) {
    mv.info.set_team(team);
    mv.info.clear();
    mv.map.clear_range();
    mv.map.clear_highlight();
    mv.map.update_highlight(&mut mv.level);
    mv.level.begin_turn();
}

pub fn update_programs(level: &mut Level, map: &mut MapView) -> bool {
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
