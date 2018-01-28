use std::io::{self, Read};
use std::error::Error;

fn main() {
    let room = {
        let mut new_room = Room {
            tiles: Grid::default(),
            props: Grid::default(),
            size: (3, 3),
        };

        for x in 0..3  {
            for y in 0..3 {
                new_room.tiles[x][y] = Option::Some((Tile::Grass, 0));
            }
        }

        new_room.props[1][0] = Option::Some(Prop::Rock);
        new_room.props[2][2] = Option::Some(Prop::Rock);
        new_room
    };

    let mut player = (1, 1);

    let mut move_player = |dir| {
        let pos = match dir {
            Direction::North => (player.0, player.1 - 1),
            Direction::East =>  (player.0 + 1, player.1),
            Direction::South => (player.0, player.1 + 1),
            Direction::West =>  (player.0 - 1, player.1),
        };

        if pos.0 >= 0 && pos.0 < room.size.0 && pos.1 >= 0 && pos.1 < room.size.1 &&
                room.tiles[pos.0][pos.1].is_some() && room.props[pos.0][pos.1].is_none()
        {
            player = pos as (usize, usize);
            println!("{:?}", player);
        }

        for y in 0..room.size.0 {
            for x in 0..room.size.1 {
                if (x, y) == player {
                    print!("P");
                } else if room.props[x][y].is_some() {
                    print!("o");
                } else if room.tiles[x][y].is_some() {
                    print!("_");
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

const ROOM_MAX_SIZE: usize = 16;
// TODO: make data contiguous; allocate on heap
type Grid<T> = [[T; ROOM_MAX_SIZE]; ROOM_MAX_SIZE];

struct Room {
    tiles:  Grid<Option<(Tile, u8)>>,
    props:  Grid<Option<Prop>>,

    size: (usize, usize),
}

enum Tile {
    Grass,
    Dirt,
    Sand,
    Stone,
    Water,
}

enum Prop {
    Rock,
}

enum Direction {
    North,
    East,
    South,
    West,
}