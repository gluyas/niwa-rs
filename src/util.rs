use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub enum Direction {
    North = 0,
    East  = 1,
    South = 2,
    West  = 3,
}

impl Direction {
    pub fn all() -> &'static [Direction] {
        &[Direction::North, Direction::East, Direction::South, Direction::West]
    }
    pub fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East  => Direction::West,
            Direction::South => Direction::North,
            Direction::West  => Direction::East,
        }
    }

    pub fn as_flag(self) -> u8 {
        1 << (self as u8)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq)]
pub struct Pos {
    pub x: u8,
    pub y: u8,
}

impl Pos {
    pub fn step <P: Into<Pos>> (mut self, dir: Direction, bound: P) -> Option<Pos> {
        let sub_bounded = |val: &mut u8| -> bool {
            if *val > 0 { *val -= 1; true } else { false }
        };

        let add_bounded = |val: &mut u8, bound: &u8| -> bool {
            if *val < bound - 1 { *val += 1; true } else { false }
        };

        let bound = bound.into();
        if match dir {
            Direction::North => sub_bounded(&mut self.y),
            Direction::East  => add_bounded(&mut self.x, &bound.x),
            Direction::South => add_bounded(&mut self.y, &bound.y),
            Direction::West  => sub_bounded(&mut self.x),
        } {
            Some(self)
        } else {
            None
        }
    }

    pub fn snap_to_edge <P: Into<Pos>> (mut self, dir: Direction, bounds: P) -> Pos {
        let bounds = bounds.into();
        match dir {
            Direction::North => self.y = 0,
            Direction::East  => self.x = bounds.x - 1,
            Direction::South => self.y = bounds.y - 1,
            Direction::West  => self.x = 0,
        }
        self
    }

    pub fn iter_line <P: Into<Pos>> (self, dir: Direction, bounds: P) -> LineIterator {
        LineIterator::new(self, dir, bounds)
    }

    pub fn iter_rect(self, dir_primary: Direction, dir_secondary: Direction) -> RectIterator {
        RectIterator::new(self, dir_primary, dir_secondary)
    }

    pub fn contains <P: Into<Pos>> (self, other: P) -> bool {
        let other = other.into();
        self.x > other.x && self.y > other.y
    }

    #[inline]
    pub fn is_within <P: Into<Pos>> (self, bound: P) -> bool {
        bound.into().contains(self)
    }

    pub fn component_add <P: Into<Pos>> (self, other: P) -> Pos {
        let other = other.into();
        (self.x + other.x, self.y + other.y).into()
    }

    pub fn component_diff <P: Into<Pos>> (self, other: P) -> Pos {
        let diff = |a, b| if a >= b { a - b } else { b - a };

        let other = other.into();
        (diff(self.x, other.x), diff(self.y, other.y)).into()
    }
}

impl From<(u8, u8)> for Pos {
    #[inline]
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

pub struct LineIterator {
    next: Pos,
    dir: Direction,
    bound: Pos,
}

impl Iterator for LineIterator {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bound == (0, 0) {
            None
        } else {
            let val = self.next;

            if let Some(step) = self.next.step(self.dir, self.bound) {
                self.next = step;
            } else {
                self.bound = (0, 0).into();
            }
            Some(val)
        }
    }
}

impl LineIterator {
    pub fn new <P0: Into<Pos>, P1: Into<Pos>> (origin: P0, dir: Direction, bound: P1) -> LineIterator {
        let origin = origin.into();

        let bound = {
            let bound = bound.into();
            if bound.contains(origin) {
                bound
            } else {
                (0, 0).into()
            }
        };

        LineIterator { next: origin, dir, bound }
    }
}

pub struct RectIterator {
    primary: LineIterator,
    secondary: LineIterator,
}

impl Iterator for RectIterator {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.primary.next();

        if next.is_none() {
            self.secondary.next().map(|next| {
                self.primary = LineIterator {
                    next,
                    dir: self.primary.dir,
                    bound: self.secondary.bound
                };
                self.primary.next()
            }).unwrap_or_default()
        } else {
            next
        }
    }
}

impl RectIterator {
    pub fn new <P: Into<Pos>> (range: P, dir_primary: Direction, dir_secondary: Direction) -> RectIterator {
        if (dir_primary as u8 + dir_secondary as u8) % 2 == 0 {
            panic!("Attempted to make RectIterator with non-orthogonal directions")
        }
        let range = range.into();

        let origin = Pos::default()
            .snap_to_edge(dir_primary.opposite(), range)
            .snap_to_edge(dir_secondary.opposite(), range);

        let mut secondary = LineIterator::new(origin, dir_secondary, range);

        // pull the first value from secondary so the next time it is called it yields origin + dir
        let primary = if let Some(origin) = secondary.next() {
            LineIterator::new(origin, dir_primary, range)
        } else {
            LineIterator::new(origin, dir_secondary, (0, 0))
        };

        RectIterator { primary, secondary }
    }
}

#[derive(Clone, Default)]
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
    pub fn of_default <P: Into<Pos>> (size: P) -> Grid<T> {
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