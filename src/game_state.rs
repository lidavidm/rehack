use termion;
use termion::event::{Key, Event, MouseEvent};

use voodoo;
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
    Setup(UiState),
    PlayerTurn(UiState),
    AITurn(UiState),
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

impl GameState {
    pub fn translate_event(&self, event: Event, mv: &mut ModelView) -> Option<UiEvent> {
        match (self, event) {
            (_, Event::Key(Key::Char('q'))) => Some(UiEvent::Quit),
            (&GameState::PlayerTurn(_), Event::Mouse(MouseEvent::Press(_, x, y))) |
            (&GameState::Setup(_), Event::Mouse(MouseEvent::Press(_, x, y))) => {
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

    pub fn next(self, event: termion::event::Event, mv: &mut ModelView) -> GameState {
        if let Some(event) = self.translate_event(event, mv) {
            match self {
                GameState::Setup(ui) => Self::next_setup_turn(ui, event, mv),
                GameState::PlayerTurn(ui) => match event {
                    UiEvent::EndTurn => {
                        match mv.level.check_victory() {
                            // TODO: transition to victory/defeat screens
                            Some(Team::Player) => GameState::Quit,
                            Some(Team::Enemy) => GameState::Quit,
                            None => GameState::AITurnTransition
                        }
                    },
                    _ => Self::next_player_turn(ui, event, mv)
                },
                GameState::MissionSelect(_) => self,
                GameState::SetupTransition |
                GameState::AITurnTransition | GameState::PlayerTurnTransition |
                GameState::AITurn(_) | GameState::Quit => self,
            }
        }
        else {
            match (self, event) {
                (GameState::MissionSelect(ms), Event::Key(_)) => Self::next_mission_turn(ms, mission_select::UiEvent::KeyPressed, mv),
                (s, _) => s
            }
        }
    }

    pub fn tick(self, mv: &mut ModelView) -> GameState {
        match self {
            GameState::Setup(ui) => Self::next_setup_turn(ui, UiEvent::Tick, mv),
            GameState::PlayerTurn(ui) => Self::next_player_turn(ui, UiEvent::Tick, mv),
            GameState::MissionSelect(ms) => Self::next_mission_turn(ms, mission_select::UiEvent::Tick, mv),
            GameState::AITurnTransition => {
                begin_turn(Team::Enemy, mv);
                GameState::AITurn(UiState::Unselected)
            }
            GameState::PlayerTurnTransition => {
                if mv.level.check_victory().is_some() {
                    GameState::Quit
                }
                else {
                    begin_turn(Team::Player, mv);
                    GameState::PlayerTurn(UiState::Unselected)
                }
            }
            GameState::AITurn(UiState::Animating) => {
                let modified = update_programs(&mut mv.level, &mut mv.map);

                if !modified {
                    GameState::AITurn(UiState::Unselected)
                }
                else {
                    GameState::AITurn(UiState::Animating)
                }
            }
            GameState::AITurn(_) => {
                let ai_state = ai::ai_tick(&mut mv.level, &mut mv.map);
                mv.map.set_help(format!("AI STATUS: {:?}", ai_state));
                match ai_state {
                    ai::AIState::Done => GameState::PlayerTurnTransition,
                    ai::AIState::Plotting => GameState::AITurn(UiState::Unselected),
                    ai::AIState::WaitingAnimation => GameState::AITurn(UiState::Animating),
                }
            }
            GameState::SetupTransition => {
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
                GameState::Setup(UiState::Unselected)
            }
            GameState::Quit => self,
        }
    }

    pub fn display(&mut self, compositor: &mut voodoo::compositor::Compositor, mv: &mut ModelView) {
        use self::GameState::*;

        match self {
            &mut MissionSelect(ref mut state) => {
                mission_select::display(state, compositor, mv);
            }
            _ => {
                mv.info.refresh(compositor);
                mv.map.display(&mv.level);
                mv.map.refresh(compositor);
            }
        }
    }

    pub fn next_player_turn(ui_state: UiState, event: UiEvent, mv: &mut ModelView) -> GameState {
        match event {
            UiEvent::ClickMap(_) | UiEvent::ClickInfo(_) | UiEvent::Tick => {
                GameState::PlayerTurn(player_turn::next(ui_state, event, mv))
            }
            UiEvent::EndTurn | UiEvent::Quit => unreachable!(),
        }
    }

    pub fn next_mission_turn(mut mission_state: mission_select::State, event: mission_select::UiEvent, mv: &mut ModelView) -> GameState {
        match mission_select::next(&mut mission_state, event, mv) {
            mission_select::Transition::Ui(_) => GameState::MissionSelect(mission_state),
            mission_select::Transition::Level(level) => {
                mv.level = level;
                GameState::SetupTransition
            }
        }
    }

    pub fn next_setup_turn(ui_state: UiState, event: UiEvent, mv: &mut ModelView) -> GameState {
        match event {
            UiEvent::ClickMap(_) | UiEvent::ClickInfo(_) | UiEvent::Tick => {
                GameState::Setup(player_turn::next_setup(ui_state, event, mv))
            }
            UiEvent::EndTurn => {
                // TODO: reset
                mv.info.primary_action = ">    End Turn    <".to_owned();
                GameState::PlayerTurnTransition
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
