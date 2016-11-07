extern crate termion;
extern crate thread_scoped;
extern crate time;
extern crate voodoo;

mod level;
mod program;

use std::io::{Write};
use std::sync::mpsc::channel;

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead};

use voodoo::color::ColorValue;
use voodoo::window::{Point, TermCell, Window};

use level::Level;
use program::{Program, ProgramRef};

const LEVEL_DESCR: [&'static str; 22] = [
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
    "          .........................                       ",
    "          .........................                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "   o................             .....                    ",
    "   o................             .....                    ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          ..                     ..                       ",
    "          .........................                       ",
    "          .........................                       ",
    "                                                          ",
    "                                                          ",
    "                                                          ",
];

enum UiState {
    Unselected,
    Selected,
    Damage(ProgramRef, usize),
}

enum UiEvent {
    Quit,
    Tick,
    Test,
    Click(Point),
    // Movement
}

enum GameState {
    Setup,
    PlayerTurn,
    AITurn,
    AIExecute,
    Quit,
}

struct InfoView {
    window: Window,
}

impl InfoView {
    fn new(window: Window) -> InfoView {
        InfoView {
            window: window,
        }
    }

    fn refresh(&mut self, stdout: &mut std::io::Stdout) {
        self.window.refresh(stdout);
    }

    fn clear(&mut self) {
        for col in 2..self.window.width - 2 {
            self.window.put_at(Point::new(col, 2), ' ');
        }
    }

    fn display_program(&mut self, program: &Program) {
        self.window.print_at(Point::new(2, 2), &program.name);
    }
}

struct MapView {
    window: Window,
    highlight: Option<ProgramRef>,
    overlay: Vec<(Point, TermCell)>,
}

impl MapView {
    fn new(window: Window) -> MapView {
        MapView {
            window: window,
            highlight: None,
            overlay: Vec::new(),
        }
    }

    fn from_global_frame(&self, p: Point) -> Option<Point> {
        self.window.position.from_global_frame(p)
    }

    fn display(&mut self, level: &Level) {
        for (y, line) in level.layout.iter().enumerate() {
            let y = y + 1;
            for (x, tile) in line.chars().enumerate() {
                let x = x + 1;
                match Level::convert(tile) {
                    Some(c) => self.window.put_at(Point::new(x as u16, y as u16), c),
                    None => {},
                }
            }
        }

        for program in level.player_programs.iter() {
            program.borrow().display_color(ColorValue::Green, &mut self.window);
        }

        for program in level.enemy_programs.iter() {
            program.borrow().display_color(ColorValue::Red, &mut self.window);
        }

        if let Some(ref program) = self.highlight {
            program.borrow().display_color(ColorValue::Blue, &mut self.window);
        }

        for &(p, c) in self.overlay.iter() {
            self.window.put_at(p, c);
        }
    }

    fn refresh(&mut self, stdout: &mut std::io::Stdout) {
        self.window.refresh(stdout);
    }

    fn highlight(&mut self, program: ProgramRef, level: &Level) {
        self.highlight = Some(program.clone());
        self.update_highlight(level);
    }

    fn update_highlight(&mut self, level: &Level) {
        if let Some(ref program) = self.highlight {
            self.overlay.clear();
            let program = program.borrow();

            if !program.can_move() {
                return;
            }

            let Point { x, y } = program.position;
            let east = Point::new(x + 1, y);
            if level.passable(east) {
                self.overlay.push((east, '→'.into()));
            }
            let west = Point::new(x - 1, y);
            if level.passable(west) {
                self.overlay.push((west, '←'.into()));
            }
            let north = Point::new(x, y - 1);
            if level.passable(north) {
                self.overlay.push((north, '↑'.into()));
            }
            let south = Point::new(x, y + 1);
            if level.passable(south) {
                self.overlay.push((south, '↓'.into()));
            }
        }
    }

    fn clear_highlight(&mut self) {
        self.highlight = None;
        self.overlay.clear();
    }
}

struct UiModelView {
    info: InfoView,
    map: MapView,
}

impl UiState {
    fn translate_click(click: Point, map: &MapView) -> Option<Point> {
        for &(point, _) in map.overlay.iter() {
            if click == point {
                return Some(point);
            }
        }
        None
    }

    fn next(self, event: UiEvent, level: &mut Level, mv: &mut UiModelView) -> UiState {
        use UiEvent::*;
        use UiState::*;

        let UiModelView { ref mut info, ref mut map } = *mv;

        match (self, event) {
            (Unselected, Click(p)) => {
                for program in level.player_programs.iter() {
                    if program.borrow().intersects(p) {
                        map.highlight(program.clone(), &level);
                        info.display_program(&program.borrow());
                        return Selected;
                    }
                }
                Unselected
            }
            (Selected, Click(p)) => {
                let result = Self::translate_click(p, map);
                if let Some(p) = result {
                    if let Some(ref mut program) = map.highlight {
                        program.borrow_mut().move_to(p);
                    }
                    map.update_highlight(&level);
                    Selected
                }
                else {
                    map.clear_highlight();
                    info.clear();
                    Unselected
                }
            }
            (Damage(program, damage), Tick) => {
                if damage == 0 {
                    if map.highlight.is_some() {
                        Selected
                    }
                    else {
                        Unselected
                    }
                }
                else {
                    if program.borrow_mut().damage() {
                        Damage(program.clone(), damage - 1)
                    }
                    else {
                        level.remove_program_at(program.borrow().position);
                        map.clear_highlight();
                        Damage(program.clone(), 0)
                    }
                }
            }
            (Damage(program, damage), _) => {
                Damage(program, damage)
            }
            (state, Tick) => { state }
            (Selected, Test) => {
                if let Some(ref program) = map.highlight {
                    Damage(program.clone(), 2)
                }
                else {
                    Selected
                }
            }
            (state, Test) => { state },
            (state, Quit) => { state },
        }
    }
}

struct GameModelView {
    ui_modelview: UiModelView,
}

struct State(GameState, UiState);

impl State {
    fn translate_event(&self, event: termion::event::Event, level: &mut Level, mv: &mut GameModelView) -> Option<UiEvent> {
        use GameState::*;
        match (self, event) {
            (_, Event::Key(Key::Char('q'))) => Some(UiEvent::Quit),
            (&State(PlayerTurn, _), Event::Mouse(MouseEvent::Press(_, x, y))) => {
                if let Some(p) = mv.ui_modelview.map.from_global_frame(Point::new(x, y)) {
                    Some(UiEvent::Click(p))
                }
                else {
                    None
                }
            }
            _ => None,
        }
    }

    fn next(self, event: termion::event::Event, level: &mut Level, mv: &mut GameModelView) -> State {
        use GameState::*;
        if let Some(event) = self.translate_event(event, level, mv) {
            match self {
                State(PlayerTurn, ui) => Self::next_player_turn(ui, event, level, mv),
                _ => unimplemented!(),
            }
        }
        else {
            self
        }
    }

    fn next_player_turn(ui_state: UiState, event: UiEvent, level: &mut Level, mv: &mut GameModelView) -> State {
        use GameState::*;
        match event {
            UiEvent::Quit => State(Quit, ui_state),
            click@UiEvent::Click(_) => {
                State(PlayerTurn, ui_state.next(click, level, &mut mv.ui_modelview))
            }
            _ => State(PlayerTurn, ui_state),
        }
    }
}

fn main() {
    use std::sync::mpsc::TryRecvError::*;
    use std::thread;
    use std::time::Duration;

    use voodoo::terminal::{Mode, Terminal};

    let mut level = Level::new(&LEVEL_DESCR);
    level.add_player_program(Program::new(Point::new(11, 11), "Hack"));
    level.add_player_program(Program::new(Point::new(5, 12), "Hack"));
    level.add_enemy_program(Program::new(Point::new(7, 12), "Hack"));

    let mut terminal = Terminal::new();
    terminal.cursor(Mode::Disabled);
    terminal.clear_color(ColorValue::Black);
    let Terminal { ref mut stdin, ref mut stdout } = terminal;

    stdout.flush().unwrap();

    let mut info = voodoo::window::Window::new(Point::new(0, 0), 20, 24);
    let mut map = voodoo::window::Window::new(Point::new(20, 0), 60, 24);
    info.border();
    map.border();

    let mut info_view = InfoView::new(info);
    let mut map_view = MapView::new(map);
    info_view.refresh(stdout);
    map_view.display(&level);
    map_view.refresh(stdout);

    let mut ui_modelview = UiModelView {
        info: info_view,
        map: map_view,
    };
    let mut ui_state = UiState::Unselected;

    let mut state = State(GameState::PlayerTurn, ui_state);
    let mut mv = GameModelView {
        ui_modelview: ui_modelview,
    };

    let (tx, rx) = channel();
    let guard = unsafe {
        thread_scoped::scoped(move || {
            for c in stdin.events() {
                let evt = c.unwrap();
                if let Event::Key(Key::Char('q')) = evt {
                    break;
                }
                tx.send(evt).unwrap();
            }
        })
    };

    let mut t = time::precise_time_ns();
    let mut dt = 0;

    'main: loop {
        loop {
            // Handle all pending events
            let msg = rx.try_recv();
            match msg {
                Ok(evt) => {
                    state = state.next(evt, &mut level, &mut mv);
                    if let State(GameState::Quit, _) = state {
                        break 'main;
                    }
                },
                Err(Disconnected) => break 'main,
                Err(Empty) => break,
            }
        }

        let now = time::precise_time_ns();
        dt += now - t;

        // TODO: use constant
        while dt >= 100000000 {
            // state = state.next(UiEvent::Tick, &mut level, &mut mv);
            // ui_state = ui_state.next(UiEvent::Tick, &mut level, &mut ui_modelview);
            dt -= 100000000;
        }

        mv.ui_modelview.info.refresh(stdout);
        mv.ui_modelview.map.display(&level);
        mv.ui_modelview.map.refresh(stdout);
        t = now;

        thread::sleep(Duration::from_millis(100 - dt / 1000000));
    }
    guard.join();
}
