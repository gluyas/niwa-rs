mod util;
use util::*;

mod world;
use world::*;

mod puzzle;
use puzzle::*;

use std::io;
use std::error::Error;

fn main() {
    let room = make_room();
    let mut puzzle = make_puzzle();

    let mut player = GridIndex::default();

    let mut input = String::new();
    let stdin = io::stdin();

    let mut next_input_spell_dir = false;

    render(player, &room, &puzzle);

    'main: loop {
        match stdin.read_line(&mut input) {
            Ok(_) => {
                for c in input.chars() {
                    if !next_input_spell_dir {
                        match c {
                            'w' => move_player(Direction::North, &mut player, &room, &puzzle),
                            'd' => move_player(Direction::East, &mut player, &room, &puzzle),
                            's' => move_player(Direction::South, &mut player, &room, &puzzle),
                            'a' => move_player(Direction::West, &mut player, &room, &puzzle),
                            'c' => next_input_spell_dir = true,
                            'q' => break 'main,
                            _   => {},
                        };
                    } else {
                        next_input_spell_dir = false;
                        match c {
                            'w' => cast_spell(Direction::North, &player, &room, &mut puzzle),
                            'd' => cast_spell(Direction::East, &player, &room, &mut puzzle),
                            's' => cast_spell(Direction::South, &player, &room, &mut puzzle),
                            'a' => cast_spell(Direction::West, &player, &room, &mut puzzle),
                            'q' => break 'main,
                            _   => next_input_spell_dir = true,
                        };
                    }
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

fn move_player(dir: Direction, player: &mut GridIndex, room: &Room, puzzle: &PuzzleGrid) {
    if let Some(new_pos) = player.step(dir, room.size()) {
        if room.tiles[new_pos].is_some()
        && room.tiles[new_pos].as_ref().unwrap().prop.is_none()
        && puzzle.cells[new_pos].as_ref()
            .map_or(true, |cell| { cell.plant.is_none() })
        {
            *player = new_pos;
        }
    }

    render(*player, room, puzzle);
}

fn cast_spell(dir: Direction, player: &GridIndex, room: &Room, puzzle: &mut PuzzleGrid) {
    let mut puzzle_temp = puzzle.clone();

    for pos in player.iter_line(dir, room.size()) {
        // check that spell can pass through terrain
        if !room.tiles[pos].as_ref().map(|tile| {
            match tile.material {
                Material::Grass | Material::Dirt => true,
                _ => false
            }
        }).unwrap_or(false) {
            break;
        }

        let mut absorbed = false;

        if puzzle_temp.cells[pos].is_some() {
            let cell = puzzle_temp.cells[pos].as_mut().unwrap();

            if let Some(_plant) = cell.plant.as_ref() {
                // TODO: implement plant specific behaviour here
                cell.hits += 1;
            };

            let virgin = match puzzle_temp.regions[cell.region] {
                PuzzleStatus::Virgin => true,
                _ => false,
            };

            if cell.has_wall(dir) && virgin {
                puzzle_temp.regions[cell.region] = PuzzleStatus::Exhausted;
                absorbed = true;
            }
        }
        if absorbed {
            *puzzle = puzzle_temp;
            break;
        }
    }

    render(*player, room, puzzle);
}

fn render(player: GridIndex, room: &Room, puzzle: &PuzzleGrid) {
    for y in 0..room.size().x {
        for x in 0..room.size().y {
            if player == (x, y) {
                print!("!");
            } else if let Some(cell) = puzzle.cells[(x, y)].as_ref() {
                if let Some(_plant) = cell.plant.as_ref() {
                    if cell.is_sprouted() {
                        print!("{}", cell.hits);
                    } else {
                        print!("*");
                    }
                } else {
                    if let PuzzleStatus::Virgin = puzzle.regions[cell.region] {
                        print!("{}", cell.get_symbol_virgin());
                    } else {
                        print!("{}", cell.get_symbol_exhausted());
                    }
                }
            } else if let Some(tile) = room.tiles[(x, y)].as_ref() {
                if tile.prop.is_some() {
                    print!("o");
                } else {
                    print!(".");
                }
            } else {
                print!(" ");
            }
        }
        println!();
    }
}

fn make_puzzle() -> PuzzleGrid {
    let mut puzzle = PuzzleGrid {
        cells: Grid::of_default((4, 4)),
        regions: vec![PuzzleStatus::default()],
    };

    puzzle.set_cell((3,0), Some(PuzzleCell::new(0, None)));
    puzzle.set_cell((2,0), Some(PuzzleCell::new(0, None)));
    puzzle.set_cell((3,1), Some(PuzzleCell::new(0, Some(Plant::Default))));
    puzzle.set_cell((2,1), Some(PuzzleCell::new(0, None)));
    puzzle.set_cell((1,1), Some(PuzzleCell::new(0, Some(Plant::Default))));

    puzzle
}

fn make_room() -> Room {
    let mut room = Room {
        tiles: Grid::of_default((4, 4)),
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
    room.tiles[(3, 3)].as_mut().unwrap().prop = Some(Prop::Rock);
    room.tiles[(1, 3)] = None;
    room
}