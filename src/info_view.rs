use voodoo::color::ColorValue;
use voodoo::window::{FormattedString, Point, Window};

use program::{Ability, Program, Team};

pub struct ChoiceList<T> {
    y: u16,
    list: Vec<(String, T)>,
    selected: Option<u16>,
}

impl<T> ChoiceList<T> {
    pub fn new(y: u16) -> ChoiceList<T> {
        ChoiceList {
            y: y,
            list: vec![],
            selected: None,
        }
    }

    pub fn choices(&mut self) -> &mut Vec<(String, T)> {
        &mut self.list
    }

    pub fn handle_click(&mut self, point: Point) -> Option<&T> {
        if point.y < self.y || point.y >= self.y + self.list.len() as u16 {
            self.selected = None;
        }
        else if let Some(offset) = self.selected {
            if offset == point.y {
                self.selected = None;
            }
            else {
                self.selected = Some(point.y);
            }
        }
        else {
            self.selected = Some(point.y);
        }

        self.get_selection()
    }

    pub fn get_selection_index(&self) -> Option<u16> {
        self.selected.map(|y| y - self.y)
    }

    pub fn get_selection(&self) -> Option<&T> {
        self.selected.map(move |y| &self.list[(y - self.y) as usize].1)
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.selected = None;
    }

    pub fn display(&self, window: &mut Window) {
        let mut y = self.y;
        for (number, &(ref label, _)) in self.list.iter().enumerate() {
            let number = number as u16 + self.y;
            let mut f: FormattedString = label.into();
            f.bg = if let Some(offset) = self.selected {
                if offset == number {
                    Some(ColorValue::Red)
                }
                else {
                    Some(ColorValue::Magenta)
                }
            } else { Some(ColorValue::Magenta) };
            window.print_at(Point::new(2, y), f);
            y += 1;
        }
    }
}

pub struct InfoView {
    pub window: Window,
    ability_list: ChoiceList<Ability>,
    team: Team,
    pub primary_action: String,
}

impl InfoView {
    pub fn new(window: Window) -> InfoView {
        let info = InfoView {
            window: window,
            ability_list: ChoiceList::new(6),
            team: Team::Player,
            primary_action: "     End Turn     ".to_owned(),
        };

        info
    }

    pub fn from_global_frame(&self, p: Point) -> Option<Point> {
        self.window.position.from_global_frame(p)
    }

    pub fn refresh(&mut self, stdout: &mut ::std::io::Stdout) {
        self.window.refresh(stdout);
    }

    pub fn display_end_turn(&mut self) {
        let mut f: FormattedString = (&self.primary_action).into();
        f.bg = Some(ColorValue::Magenta);
        self.window.print_at(Point::new(2, 23), f);
    }

    pub fn set_team(&mut self, team: Team) {
        self.team = team;
    }

    pub fn clear(&mut self) {
        self.ability_list.clear();
        self.window.clear();
        self.window.border();
        self.window.print_at(Point::new(2, 1), match self.team {
            Team::Player => "﻿PLAYER TURN",
            Team::Enemy => "﻿AI TURN",
        });
        if let Team::Player = self.team {
            self.display_end_turn();
        }
    }

    pub fn display_abilities(&mut self) {
        self.ability_list.display(&mut self.window);
    }

    pub fn display_program(&mut self, program: &Program) {
        self.window.print_at(Point::new(2, 2), &program.name);
        self.update_program(program);

        self.ability_list.clear();
        if program.turn_state.ability_used {
            self.window.print_at(Point::new(2, 4), "Ability used");
        }
        else {
            self.window.print_at(Point::new(2, 4), "Abilities:");
            self.ability_list.choices().extend(program.abilities.iter().cloned());

            self.display_abilities();
        }
    }

    pub fn update_program(&mut self, program: &Program) {
        self.window.print_at(
            Point::new(2, 3),
            &format!("Moves: {}/{}", program.max_moves - program.turn_state.moves_made, program.max_moves));
    }

    pub fn translate_click(&mut self, click: Point) -> Option<Ability> {
        let result = self.ability_list.handle_click(click).cloned();
        self.display_abilities();

        result
    }

    pub fn clear_ability(&mut self) {
        self.ability_list.clear_selection();
        self.display_abilities();
    }
}
