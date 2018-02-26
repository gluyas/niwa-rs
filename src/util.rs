use std::ops::{Index, IndexMut, Add, AddAssign, Sub, SubAssign, Neg};

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
pub struct GridVector {
    pub x: i8,
    pub y: i8,
}

impl GridVector {
    #[inline]
    pub fn nudge(self, dir: Direction) {
        self + dir;
    }
}

impl <P> Add<P> for GridVector where P: Into<GridVector> {
    type Output = GridVector;

    fn add(self, rhs: P) -> Self::Output {
        let rhs = rhs.into();
        (self.x + rhs.x, self.y + rhs.y).into()
    }
}

impl <P> AddAssign<P> for GridVector where P: Into<GridVector> {
    fn add_assign(&mut self, rhs: P) {
        let rhs = rhs.into();
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl <P> Sub<P> for GridVector where P: Into<GridVector> {
    type Output = GridVector;

    fn sub(self, rhs: P) -> Self::Output {
        self + (-rhs.into())
    }
}

impl <P> SubAssign<P> for GridVector where P: Into<GridVector> {
    fn sub_assign(&mut self, rhs: P) {
        *self += (-rhs.into());
    }
}

impl Neg for GridVector {
    type Output = GridVector;

    fn neg(self) -> Self::Output {
        (-self.x, -self.y).into()
    }
}

impl <P> PartialEq<P> for GridVector where P: Into<GridVector> + Copy {
    fn eq(&self, other: &P) -> bool {
        let other = (*other).into();
        self.x == other.x && self.y == other.y
    }
}

impl From<Direction> for GridVector {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => (0, -1),
            Direction::East  => (1, 0),
            Direction::South => (0, 1),
            Direction::West  => (-1, 0),
        }.into()
    }
}

impl From<(i8, i8)> for GridVector {
    #[inline]
    fn from(tuple: (i8, i8)) -> Self {
        GridVector { x: tuple.0, y: tuple.1 }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq)]
pub struct GridIndex {
    pub x: u8,
    pub y: u8,
}

impl GridIndex {
    pub fn step <P: Into<GridIndex>> (mut self, dir: Direction, bound: P) -> Option<GridIndex> {
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

    pub fn snap_to_edge <P: Into<GridIndex>> (mut self, dir: Direction, bounds: P) -> GridIndex {
        let bounds = bounds.into();
        match dir {
            Direction::North => self.y = 0,
            Direction::East  => self.x = bounds.x - 1,
            Direction::South => self.y = bounds.y - 1,
            Direction::West  => self.x = 0,
        }
        self
    }

    pub fn iter_line <P: Into<GridIndex>> (self, dir: Direction, bounds: P) -> LineIterator {
        LineIterator::new(self, dir, bounds)
    }

    pub fn iter_rect(self, dir_primary: Direction, dir_secondary: Direction) -> RectIterator {
        RectIterator::new(self, dir_primary, dir_secondary)
    }

    pub fn contains <P: Into<GridIndex>> (self, other: P) -> bool {
        let other = other.into();
        self.x > other.x && self.y > other.y
    }

    #[inline]
    pub fn is_within <P: Into<GridIndex>> (self, bound: P) -> bool {
        bound.into().contains(self)
    }

    pub fn component_add <P: Into<GridIndex>> (self, other: P) -> GridIndex {
        let other = other.into();
        (self.x + other.x, self.y + other.y).into()
    }

    pub fn component_diff <P: Into<GridIndex>> (self, other: P) -> GridIndex {
        let diff = |a, b| if a >= b { a - b } else { b - a };

        let other = other.into();
        (diff(self.x, other.x), diff(self.y, other.y)).into()
    }
}

impl From<(u8, u8)> for GridIndex {
    #[inline]
    fn from(tuple: (u8, u8)) -> Self {
        GridIndex { x: tuple.0, y: tuple.1 }
    }
}

impl <P> PartialEq<P> for GridIndex where P: Into<GridIndex> + Copy {
    fn eq(&self, other: &P) -> bool {
        let other = (*other).into();
        self.x == other.x && self.y == other.y
    }
}

pub struct LineIterator {
    next: GridIndex,
    dir: Direction,
    bound: GridIndex,
}

impl Iterator for LineIterator {
    type Item = GridIndex;

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
    pub fn new <P0: Into<GridIndex>, P1: Into<GridIndex>> (origin: P0, dir: Direction, bound: P1) -> LineIterator {
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
    type Item = GridIndex;

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
    pub fn new <P: Into<GridIndex>> (range: P, dir_primary: Direction, dir_secondary: Direction) -> Self {
        if (dir_primary as u8 + dir_secondary as u8) % 2 == 0 {
            panic!("Attempted to make RectIterator with non-orthogonal directions")
        }
        let range = range.into();

        let origin = GridIndex::default()
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
    size: GridIndex,
}

impl <T> Grid<T> {
    pub fn size(&self) -> GridIndex {
        self.size
    }

    pub fn bounds_check<P: Into<GridIndex>> (&self, index: P) -> bool {
        let index = index.into();
        index.x < self.size.x && index.y < self.size.y
    }

    fn linear_index (&self, index: GridIndex) -> usize {
        (index.y * self.size.x + index.x) as usize
    }
}

impl <T: Default> Grid<T> {
    pub fn of_default <P: Into<GridIndex>> (size: P) -> Self {
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
    pub fn of <P: Into<GridIndex>> (size: P, val: T) -> Self {
        let size = size.into();
        Grid {
            data: vec![val; (size.x * size.y) as usize],
            size,
        }
    }
}

impl <P, T> Index<P> for Grid<T> where P: Into<GridIndex> {
    type Output = T;

    fn index(&self, index: P) -> &Self::Output {
        let index = index.into();
        if !self.bounds_check(index) {
            panic!("Grid access at invalid index");
        } else {
            unsafe {    // our bounds check is more specific than Vec's
                self.data.get_unchecked(self.linear_index(index.into()))
            }
        }
    }
}

impl <P, T> IndexMut<P> for Grid<T> where P: Into<GridIndex> {
    fn index_mut(&mut self, index: P) -> &mut Self::Output {
        let index = index.into();
        if !self.bounds_check(index) {
            panic!("Grid access at invalid index");
        } else {
            unsafe {    // our bounds check is more specific than Vec's
                let linear_index = self.linear_index(index.into());
                self.data.get_unchecked_mut(linear_index)
            }
        }
    }
}