use voodoo::window::{Point, Window};

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
    }
}
