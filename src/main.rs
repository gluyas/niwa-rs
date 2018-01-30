mod world;

use std::io::{self, Read};
use std::error::Error;

use world::*;

fn main() {
    let room = {
        let mut room = Room {
            tiles: Grid::default((4, 4)),
        };

        for x in 0..room.size().0  {
            for y in 0..room.size().1 {
                room.tiles[(x, y)] = Option::Some(
                    Tile {
                        material: Material::Grass,
                        prop: None,
                        height: 0,
                    }
                );
            }
        }

        room.tiles[(1, 0)].as_mut().unwrap().prop = Some(Prop::Rock);
        room.tiles[(2, 2)].as_mut().unwrap().prop = Some(Prop::Rock);
        room
    };

    let mut player = (1, 1);

    let mut move_player = |dir| {
        {
            let old_pos = player;
            nudge_bounded(&mut player, dir, &room.tiles.size());

            if room.tiles[(player.0, player.1)].is_none()
            || room.tiles[(player.0, player.1)].as_ref().unwrap().prop.is_some()
            {
                player = old_pos
            }
        }

        for y in 0..room.size().0 {
            for x in 0..room.size().1 {
                if (x, y) == player {
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

fn nudge_bounded(pos: &mut (usize, usize), dir: Direction, bounds: &(usize, usize)) {
    fn sub_bounded(val: &mut usize) {
        if *val > 0 { *val -= 1 };
    }

    fn add_bounded(val: &mut usize, bound: &usize) {
        if *val < bound - 1 { *val += 1 };
    }

    match dir {
        Direction::North => sub_bounded(&mut pos.1),
        Direction::East  => add_bounded(&mut pos.0, &bounds.0),
        Direction::South => add_bounded(&mut pos.1, &bounds.1),
        Direction::West  => sub_bounded(&mut pos.0),
    }
}