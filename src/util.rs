use std::ops::{Index, IndexMut};

pub enum Direction {
    North = 0,
    East  = 1,
    South = 2,
    West  = 3,
}

#[derive(Copy, Clone, Debug, Default, Eq)]
pub struct Pos {
    pub x: u8,
    pub y: u8,
}

impl Pos {
    pub fn nudge <P: Into<Pos>> (mut self, dir: Direction, bound: P) -> Pos {
        fn sub_bounded(val: &mut u8) {
            if *val > 0 { *val -= 1 };
        }

        fn add_bounded(val: &mut u8, bound: &u8) {
            if *val < bound - 1 { *val += 1 };
        }

        let bound = bound.into();
        match dir {
            Direction::North => sub_bounded(&mut self.y),
            Direction::East  => add_bounded(&mut self.x, &bound.x),
            Direction::South => add_bounded(&mut self.y, &bound.y),
            Direction::West  => sub_bounded(&mut self.x),
        }
        self
    }
}

impl From<(u8, u8)> for Pos {
    fn from(pos: (u8, u8)) -> Self {
        Pos { x: pos.0, y: pos.1 }
    }
}

impl <P> PartialEq<P> for Pos where P: Into<Pos> + Copy {
    fn eq(&self, other: &P) -> bool {
        let other = (*other).into();
        self.x == other.x && self.y == other.y
    }
}

pub struct Grid<T> {
    data: Vec<T>,
    size: Pos,
}

impl <T> Grid<T> {
    pub fn size(&self) -> Pos {
        self.size
    }

    pub fn check <P: Into<Pos>> (&self, index: P) -> bool {
        let index = index.into();
        index.x < self.size.x && index.y < self.size.y
    }

    fn linear_index (&self, index: Pos) -> usize {
        (index.y * self.size.x + index.x) as usize
    }
}

impl <T: Default> Grid<T> {
    pub fn default <P: Into<Pos>> (size: P) -> Grid<T> {
        let size = size.into();
        let capacity = (size.x * size.y) as usize;

        let mut vec = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            vec.push(T::default());
        }

        Grid {
            data: vec,
            size,
        }
    }
}

impl <T: Clone> Grid<T> {
    pub fn of <P: Into<Pos>> (size: P, val: T) -> Grid<T> {
        let size = size.into();
        Grid {
            data: vec![val; (size.x * size.y) as usize],
            size,
        }
    }
}

impl <P, T> Index<P> for Grid<T> where P: Into<Pos> {
    type Output = T;

    fn index(&self, index: P) -> &Self::Output {
        let index = index.into();
        if !self.check(index) {
            panic!("Grid access at invalid index");
        } else {
            &self.data[self.linear_index(index.into())]
        }
    }
}

impl <P, T> IndexMut<P> for Grid<T> where P: Into<Pos> {
    fn index_mut(&mut self, index: P) -> &mut Self::Output {
        let index = index.into();
        if !self.check(index) {
            panic!("Grid access at invalid index");
        } else {
            let linear_index = self.linear_index(index.into());
            &mut self.data[linear_index]
        }
    }
}