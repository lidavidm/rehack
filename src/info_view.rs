use voodoo::color::ColorValue;
use voodoo::window::{FormattedString, Point, Window};

use program::{Ability, Program};

pub struct InfoView {
    window: Window,
    ability_list: Vec<(String, Ability)>,
    selected_ability: Option<(usize, Ability)>,
}

impl InfoView {
    pub fn new(mut window: Window) -> InfoView {
        let mut f: FormattedString = "     End Turn     ".into();
        f.bg = Some(ColorValue::Magenta);
        window.print_at(Point::new(2, 23), f);

        InfoView {
            window: window,
            ability_list: Vec::new(),
            selected_ability: None,
        }
    }

    pub fn from_global_frame(&self, p: Point) -> Option<Point> {
        self.window.position.from_global_frame(p)
    }

    pub fn refresh(&mut self, stdout: &mut ::std::io::Stdout) {
        self.window.refresh(stdout);
    }

    pub fn clear(&mut self) {
        self.ability_list.clear();
        self.selected_ability = None;
        self.window.clear();
        self.window.border();
        let mut f: FormattedString = "     End Turn     ".into();
        f.bg = Some(ColorValue::Magenta);
        self.window.print_at(Point::new(2, 23), f);
    }

    pub fn display_abilities(&mut self) {
        let mut y = 6;
        for (ability_number, &(ref name, _)) in self.ability_list.iter().enumerate() {
            let mut f: FormattedString = name.into();
            f.bg = if let Some((offset, _)) = self.selected_ability {
                if offset == ability_number {
                    Some(ColorValue::Red)
                }
                else {
                    Some(ColorValue::Magenta)
                }
            } else { Some(ColorValue::Magenta) };
            self.window.print_at(Point::new(2, y), f);
            y += 1;
        }
    }

    // TODO: take a ProgramRef and store it (maybe a weak reference)
    pub fn display_program(&mut self, program: &Program) {
        self.window.print_at(Point::new(2, 2), &program.name);
        self.update_program(program);

        self.window.print_at(Point::new(2, 4), "Abilities:");
        self.ability_list.extend(program.abilities.iter().cloned());

        self.display_abilities();
    }

    pub fn update_program(&mut self, program: &Program) {
        self.window.print_at(
            Point::new(2, 3),
            &format!("Moves: {}/{}", program.max_moves - program.turn_state.moves_made, program.max_moves));
    }

    // TODO: return the ability range or something? Ability descriptor
    pub fn translate_click(&mut self, click: Point) -> Option<Ability> {
        for (offset, &(_, ability)) in self.ability_list.iter().enumerate() {
            if click.y == 6 + offset as u16 {
                self.selected_ability = Some((offset, ability));
                break;
            }
        }
        self.display_abilities();

        self.selected_ability.map(|(_, ability)| ability)
    }

    pub fn clear_ability(&mut self) {
        self.selected_ability = None;
        self.display_abilities();
    }
}
