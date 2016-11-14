use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell};

use game_state::{self, UiEvent, UiState, ModelView};
use info_view::InfoView;
use map_view::MapView;
use level::{CellContents, Level};
use player::Player;
use program::{Ability, Team};

fn select_program(point: Point, level: &Level, map: &mut MapView, info: &mut InfoView) -> UiState {
    use game_state::UiState::*;

    for program in level.programs.iter() {
        if program.borrow().intersects(point) && program.borrow().team == Team::Player {
            map.highlight(program.clone(), &level);
            info.display_program(&program.borrow());
            map.set_help("Click arrows to move; click ability at left to use");
            return Selected;
        }
    }
    Unselected
}

pub fn next(state: UiState, event: UiEvent, mv: &mut ModelView) -> UiState {
    use game_state::UiEvent::*;
    use game_state::UiState::*;

    let ModelView { ref mut info, ref mut map, ref mut player, ref mut level, .. } = *mv;

    let result = match (state, event) {
        (Unselected, ClickMap(p)) => {
            select_program(p, level, map, info)
        }
        (Selected, ClickMap(p)) => {
            let result = map.translate_click(p);
            if let Some(p) = result {
                if let Some(ref mut program) = map.get_highlight() {
                    program.borrow_mut().move_to(p);
                    info.update_program(&program.borrow());
                }
                map.update_highlight(&level);
                Selected
            }
            else {
                map.clear_highlight();
                info.clear();
                select_program(p, level, map, info)
            }
        }
        (Unselected, ClickInfo(_)) => Unselected,
        (Selected, ClickInfo(p)) => {
            if let Some(ability) = info.translate_click(p) {
                match ability {
                    Ability::Destroy { damage, range } => {
                        map.set_help(format!("Select target. Damage: 0x{:x} Range: 0x{:x}", damage, range));
                        map.highlight_range(range, level);
                    }
                }
                return SelectTarget(ability);
            }
            Selected
        }
        (SelectTarget(ability), ClickMap(p)) => {
            let result = map.translate_click(p);
            info.clear_ability();

            map.clear_range();
            map.update_highlight(level);
            if let Some(p) = result {
                match level.contents_of(p) {
                    CellContents::Program(p) => {
                        ability.apply(&mut p.borrow_mut());
                        if let Some(caster) = map.get_highlight() {
                            caster.borrow_mut().turn_state.ability_used = true;
                            info.clear();
                            info.display_program(&caster.borrow());
                        }
                        Animating
                    },
                    _ => Selected,
                }
            }
            else {
                Selected
            }
        }
        (SelectTarget(_), ClickInfo(p)) => {
            let result = info.translate_click(p);
            if let Some(ability) = result {
                // TODO: refactor this out
                match ability {
                    Ability::Destroy { damage, range } => {
                        map.set_help(format!("Select target. Damage: 0x{:x} Range: 0x{:x}", damage, range));
                        map.highlight_range(range, level);
                    }
                }
                SelectTarget(ability)
            }
            else {
                info.clear_ability();
                map.clear_range();
                map.update_highlight(level);
                Selected
            }
        }
        (state, Tick) => {
            let modified = game_state::update_programs(level, map);

            match state {
                Animating => {
                    if !modified {
                        if map.get_highlight().is_some() {
                            Selected
                        }
                        else {
                            info.clear();
                            Unselected
                        }
                    }
                    else {
                        state
                    }
                }
                _ => state,
            }
        }
        (Animating, _) => Animating,
        (state, Quit) | (state, EndTurn) => { state },
    };

    if let Unselected = result {
        map.set_help("Click program to control it");
    }

    result
}

pub fn next_setup(state: UiState, event: UiEvent, mv: &mut ModelView) -> UiState {
    use game_state::UiState::*;
    use game_state::UiEvent::*;

    let new_state = match (state, event) {
        (state, Quit) => state,
        (state, Tick) => state,

        (Unselected, ClickMap(p)) | (Selected, ClickMap(p)) => {
            match mv.level.contents_of(p) {
                CellContents::Uplink => {
                    let overlay = mv.map.get_overlay();
                    if overlay.contains_key("uplink") {
                        Unselected
                    }
                    else {
                        overlay.insert(
                            "uplink".to_owned(),
                            (p, TermCell::new_with_bg('O', ColorValue::Cyan)));
                        Selected
                    }
                }
                _ => {
                    Unselected
                }
            }
        },
        (Unselected, ClickInfo(_)) => Unselected,
        (Selected, ClickInfo(p)) => {
            if let Some(ref program) = mv.program_list.handle_click(p) {
                let mut p = (*program).clone();
                let uplink = mv.map.get_overlay().get("uplink").unwrap().0;
                p.position = uplink;
                mv.level.remove_uplink_at(uplink);
                mv.level.add_program(p);
            }
            if let Some(idx) = mv.program_list.get_selection_index() {
                mv.program_list.choices().remove(idx as usize);
            }
            mv.program_list.clear_selection();

            Unselected
        },

        (SelectTarget(_), _) | (Animating, _) | (_, EndTurn) => unreachable!(),
    };

    match new_state {
        Unselected => {
            mv.map.get_overlay().remove("uplink");
            mv.info.clear();
            mv.map.set_help("Choose uplink Θ to load program")
        },
        Selected => {
            mv.program_list.display(&mut mv.info.window);
            mv.info.window.print_at(Point::new(2, 2), "Programs:");
            mv.map.set_help("Choose program to load at left")
        },
        _ => unreachable!(),
    };

    new_state
}
