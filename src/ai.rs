use voodoo::window::{Point};

use map_view::MapView;
use level::{self, Level};
use program::{Ability, ProgramRef, Team};

enum AIChoice {
    Ability {
        ability: Ability,
        target: ProgramRef,
    },
    Move(Point),
}

#[derive(Clone,Copy,Debug)]
pub enum AIState {
    WaitingAnimation,
    Plotting,
    Done,
}

pub fn ai_tick(level: &Level, _map: &mut MapView) -> AIState {
    let mut result = AIState::Done;
    for program in level.programs.iter() {
        if program.borrow().team != Team::Enemy {
            continue;
        }
        let position = { program.borrow().position };
        let Point { x, y } = position;
        let abilities = { program.borrow().abilities.clone() };
        let ability_used = { program.borrow().turn_state.ability_used };
        let mut choices = vec![];

        if !ability_used {
            for (_, ability) in abilities {
                for reachable in ability.reachable_tiles(position) {
                    match level.contents_of(reachable) {
                        level::CellContents::Program(target) => {
                            if target.borrow().team == Team::Player {
                                choices.push((100, AIChoice::Ability {
                                    ability: ability,
                                    target: target,
                                }));
                            }
                        }
                        _ => {},
                    }
                }
            }
        }
        
        let east = Point::new(x + 1, y);
        if level.passable(east) {
            choices.push((50, AIChoice::Move(east)));
        }
        let west = Point::new(x - 1, y);
        if level.passable(west) {
            choices.push((50, AIChoice::Move(west)));
        }
        let north = Point::new(x, y - 1);
        if level.passable(north) {
            choices.push((50, AIChoice::Move(north)));
        }
        let south = Point::new(x, y + 1);
        if level.passable(south) {
            choices.push((50, AIChoice::Move(south)));
        }

        choices.sort_by(|&(s1, _), &(s2, _)| { s2.cmp(&s1) });
        if let Some(&(_, ref choice)) = choices.first() {
            match choice {
                &AIChoice::Ability { ability, ref target } => {
                    program.borrow_mut().turn_state.ability_used = true;
                    ability.apply(&mut target.borrow_mut());
                    result = AIState::WaitingAnimation;
                }
                &AIChoice::Move(point) => {
                    if program.borrow().can_move() {
                        program.borrow_mut().move_to(point);
                        if let AIState::Done = result {
                            result = AIState::Plotting;
                        }
                    }
                }
            }
        }
    }
    result
}
