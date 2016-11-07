use voodoo::color::ColorValue;
use voodoo::window::{FormattedString, Point, Window};

use program::Program;

pub struct InfoView {
    window: Window,
    ability_list: Vec<String>,
    selected_ability: Option<usize>,
}

impl InfoView {
    pub fn new(window: Window) -> InfoView {
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
    }

    pub fn display_abilities(&mut self) {
        let mut y = 5;
        for (ability_number, ability) in self.ability_list.iter().enumerate() {
            let mut f: FormattedString = ability.into();
            f.bg = if let Some(offset) = self.selected_ability {
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

    pub fn display_program(&mut self, program: &Program) {
        self.window.print_at(Point::new(2, 2), &program.name);

        self.window.print_at(Point::new(2, 4), "Abilities:");
        for ability in program.abilities.iter() {
            self.ability_list.push(ability.to_owned());
        }

        self.display_abilities();
    }

    pub fn translate_click(&mut self, click: Point) {
        for (offset, _) in self.ability_list.iter().enumerate() {
            if click.y == 5 + offset as u16 {
                self.selected_ability = Some(offset);
                break;
            }
        }
        self.display_abilities();
    }
}
