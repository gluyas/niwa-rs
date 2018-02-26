use std::default::Default;

use util::*;

#[derive(Clone, Default)]
pub struct PuzzleGrid {
    pub cells: Grid<Option<PuzzleCell>>,
    pub regions: Vec<PuzzleStatus>,
}

impl PuzzleGrid {
    pub fn is_blocked <P: Into<GridIndex>> (&self, pos: P, dir: Direction) -> Option<PuzzleId> {
        self.cells[pos].as_ref().and_then(|cell| {
            if cell.has_wall(dir) {
                Some(cell.region)
            } else {
                None
            }
        })
    }

    pub fn set_cell <P: Into<GridIndex>> (&mut self, pos: P, mut new_cell: Option<PuzzleCell>) {
        let pos = pos.into();

        // naively expand regions vector
        if let Some(new_cell) = new_cell.as_ref() {
            if new_cell.region >= self.regions.len() {
                let additional = new_cell.region - self.regions.len() + 1;

                self.regions.reserve(additional);
                for _ in 0..additional {
                    self.regions.push(PuzzleStatus::default());
                }
            }
        }

        // determine which borders on the cell must be closed
        for &dir in Direction::all() {

            // find the adjacent cell to the new one, if it exists
            let mut adjacent_cell = {
                let step = pos.step(dir, self.cells.size());

                step.map(|adjacent_pos| {
                    self.cells[adjacent_pos].as_mut()
                }).unwrap_or_default()
            };

            // determine whether or not to place a wall on the border
            let mut wall = true;
            if let Some(new_cell) = new_cell.as_ref() {
                if let Some(adjacent_cell) = adjacent_cell.as_ref() {
                    if new_cell.region == adjacent_cell.region {
                        wall = false;
                    }
                }
            }

            if let Some(new_cell) = new_cell.as_mut() {
                new_cell.set_wall(dir, wall);
            }

            if let Some(adjacent_cell) = adjacent_cell.as_mut() {
                adjacent_cell.set_wall(dir.opposite(), wall);
            }
        }
        self.cells[pos] = new_cell;     // finally, transfer ownership to the Grid
    }
}

pub type PuzzleId = usize;

#[derive(Copy, Clone)]
pub enum PuzzleStatus {
    Virgin,
    Exhausted,
    Complete,
}

impl Default for PuzzleStatus {
    fn default() -> Self {
        PuzzleStatus::Virgin
    }
}

#[derive(Clone)]
pub enum Plant {
    Default,
}

#[derive(Clone, Default)]
pub struct PuzzleCell {
    pub plant: Option<Plant>,
    pub hits: u8,

    pub region: PuzzleId,
    wall_data: u8,   // using Direction enum as flags
}

impl PuzzleCell {
    pub fn new(region: PuzzleId, plant: Option<Plant>) -> PuzzleCell {
        PuzzleCell {
            plant,
            hits: 0,
            region,
            wall_data: 0,
        }
    }

    pub fn is_sprouted(&self) -> bool {
        self.hits > 0
    }

    pub fn has_wall(&self, dir: Direction) -> bool {
        self.wall_data & dir.as_flag() != 0
    }

    pub fn set_wall(&mut self, dir: Direction, val: bool) {
        if val {
            self.wall_data |= dir.as_flag();
        } else {
            self.wall_data &= !dir.as_flag();
        }
    }

    pub fn set_walls(&mut self, dirs: &[Direction]) {
        self.wall_data = 0;
        for &dir in dirs {
            self.set_wall(dir, true);
        }
    }

    pub fn get_symbol_exhausted(&self) -> char {
        let symbols = [
            '┼','┬','┤','┐',
            '┴','─','┘','╴',
            '├','┌','│','╷',    // IntelliJ font renders chars 11 and 14 in reverse
            '└','╶','╵',' ',
        ];
        symbols[self.wall_data as usize]
    }

    pub fn get_symbol_virgin(&self) -> char {
        let symbols = [
            '╋','┳','┫','┓',
            '┻','━','┛','╸',
            '┣','┏','┃','╻',
            '┗','╺','╹',' ',
        ];
        symbols[self.wall_data as usize]
    }
}