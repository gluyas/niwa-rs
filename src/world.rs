use std::ops::{Index, IndexMut};

pub struct Room {
    pub tiles: Grid<Option<Tile>>,
}

impl Room {
    pub fn size(&self) -> &(usize, usize) {
        return &self.tiles.size;
    }
}

pub struct Tile {
    pub material: Material,
    pub prop: Option<Prop>,
    pub height: u8,
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

pub enum Direction {
    North = 1,
    East =  2,
    South = 4,
    West =  8,
}

pub struct Grid<T> {
    data: Vec<T>,
    size: (usize, usize),
}

impl <T> Grid<T> {
    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn check(&self, index: (usize, usize)) -> bool {
        index.0 < self.size.0 && index.1 < self.size.1
    }

    fn get_index(&self, index: (usize, usize)) -> usize {
        index.1 * self.size.0 + index.0
    }
}

impl <T> Grid<T> where T: Default {
    pub fn default(size: (usize, usize)) -> Grid<T> {
        let mut vec = Vec::<T>::default();
        let cap = size.0 * size.1;

        vec.reserve(cap);
        for _ in 0..cap {
            vec.push(T::default());
        }

        Grid {
            data: vec,
            size,
        }
    }
}

impl <T> Grid<T> where T: Clone {
    pub fn of(size: (usize, usize), val: T) -> Grid<T> {
        Grid {
            data: vec![val; size.0 * size.1],
            size,
        }
    }
}

impl <T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        if !self.check(index) {
            panic!("Grid access at invalid index");
        } else {
            &self.data[self.get_index(index)]
        }
    }
}

impl <T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        if !self.check(index) {
            panic!("Grid access at invalid index");
        } else {
            let linear_index = self.get_index(index);
            &mut self.data[linear_index]
        }
    }
}