use std::collections::HashMap;

use voodoo::window::Point;

use level;
use program::{Ability, Program, ProgramBuilder, Team};

const LEVELS: [[&'static str; 20]; 2] = [
    [
        "                                                          ",
        "                                                          ",
        "          ......................p..                       ",
        "          .........................                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "   o...............s             ..f..                    ",
        "   o...............s             ..f..                    ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          .........................                       ",
        "          ......................p..                       ",
        "                                                          ",
        "                                                          ",
        "                                                          ",
    ],
    [
        "                                                          ",
        "                                                          ",
        "          ......................p..                       ",
        "          ..........    ...........                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          o........s  ..........f..                       ",
        "          o........s  p.........f..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..                     ..                       ",
        "          ..........    ...........                       ",
        "          ......................p..                       ",
        "                                                          ",
        "                                                          ",
        "                                                          ",
    ],
];

lazy_static! {
    static ref PROGRAMS: HashMap<String, ProgramBuilder> = {
        let mut m = HashMap::new();

        m.insert("s".to_owned(),
                 ProgramBuilder::new("Sprinter")
                 .ability("Overflow", Ability::Destroy { damage: 1, range: 3 })
                 .max_tail(2)
                 .max_moves(10));

        m.insert("p".to_owned(),
                 ProgramBuilder::new("Patrol")
                 .ability("Delete", Ability::Destroy { damage: 4, range: 1 })
                 .max_tail(6)
                 .max_moves(2));

        m.insert("f".to_owned(),
                 ProgramBuilder::new("Firewall")
                 .ability("Reject", Ability::Destroy { damage: 6, range: 2 })
                 .max_tail(1)
                 .max_moves(0));

        m
    };
}

pub fn load_level(id: usize) -> Option<level::Level> {
    if let Some(desc) = LEVELS.get(id) {
        let mut level = level::Level::new(desc);

        for (row_offset, row) in LEVELS[id].iter().enumerate() {
            for (col_offset, c) in row.chars().enumerate() {
                let s: String = c.to_string();
                if let Some(builder) = PROGRAMS.get(&s) {
                    let mut instance = builder.instance(Team::Enemy);
                    instance.position = Point::new(col_offset as u16 + 1, row_offset as u16 + 1);
                    level.add_program(instance);
                }
            }
        }

        Some(level)
    }
    else {
        None
    }
}

// sprinters (s) - very fast, low damage, low health
