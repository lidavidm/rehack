use voodoo::color::ColorValue;
use voodoo::window::{FormattedString, Point, Window};

use program::Program;

pub struct InfoView {
    window: Window,
}

impl InfoView {
    pub fn new(window: Window) -> InfoView {
        InfoView {
            window: window,
        }
    }

    pub fn refresh(&mut self, stdout: &mut ::std::io::Stdout) {
        self.window.refresh(stdout);
    }

    pub fn clear(&mut self) {
        for col in 2..self.window.width - 2 {
            self.window.put_at(Point::new(col, 2), ' ');
        }
    }

    pub fn display_program(&mut self, program: &Program) {
        self.window.print_at(Point::new(2, 2), &program.name);

        self.window.print_at(Point::new(2, 4), "Abilities:");
        let mut y = 5;
        for ability in program.abilities.iter() {
            let mut f: FormattedString = ability.into();
            f.bg = Some(ColorValue::Magenta);
            self.window.print_at(Point::new(2, y), f);
            y += 1;
        }
    }
}
