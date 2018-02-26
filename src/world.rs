use util::{Grid, GridIndex};

pub struct Room {
    pub tiles: Grid<Option<Tile>>,
}

impl Room {
    pub fn size(&self) -> GridIndex {
        self.tiles.size()
    }
}

pub struct Tile {
    pub material: Material,
    pub prop: Option<Prop>,

    pub elevation: u8,
}

pub enum Material {
    Grass,
    Dirt,
    Sand,
    Stone,
    Water,
}

pub enum Prop {
    Rock,
}