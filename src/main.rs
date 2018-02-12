mod util;
use util::*;

mod world;
use world::*;

use std::io;
use std::error::Error;

fn main() {
    let room = {
        let mut room = Room {
            tiles: Grid::default((4, 4)),
        };

        for x in 0..room.size().x  {
            for y in 0..room.size().y {
                room.tiles[(x, y)] = Option::Some(
                    Tile {
                        material: Material::Grass,
                        prop: None,
                        elevation: 0,
                    }
                );
            }
        }

        room.tiles[(1, 0)].as_mut().unwrap().prop = Some(Prop::Rock);
        room.tiles[(2, 2)].as_mut().unwrap().prop = Some(Prop::Rock);
        room
    };

    let mut player = Pos::default();

    let mut move_player = |dir| {
        {
            let new_pos = player.nudge(dir, room.size());

            if room.tiles[new_pos].is_some()
            && room.tiles[new_pos].as_ref().unwrap().prop.is_none()
            {
                player = new_pos;
            }
        }

        for y in 0..room.size().x {
            for x in 0..room.size().y {
                if player == (x, y) {
                    print!("!");
                } else if room.tiles[(x, y)].is_some() {
                    if room.tiles[(x, y)].as_ref().unwrap().prop.is_some() {
                        print!("o");
                    } else {
                        print!("_");
                    }
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    };

    let mut input = String::new();
    let stdin = io::stdin();

    'main: loop {
        match stdin.read_line(&mut input) {
            Ok(_) => {
                for c in input.chars() {
                    match c {
                        'w' => move_player(Direction::North),
                        'd' => move_player(Direction::East),
                        's' => move_player(Direction::South),
                        'a' => move_player(Direction::West),
                        'q' => break 'main,
                        _   => {},
                    };
                }
                input.clear();
            },
            Err(err) => {
                println!("{}", err.description());
                break;
            },
        }
    }
}