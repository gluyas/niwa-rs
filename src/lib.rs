extern crate cgmath;

pub mod tilemap {
    use cgmath::Point2;

    use std::ops::{ Index, IndexMut };
    use std::iter::IntoIterator;

    pub type TilePos = Point2<u8>;

    #[derive(Debug, Clone)]
    pub struct TileMap<T> {
        pub data: Box<[T]>,
        pub width: u8,
        pub height: u8,
    }

    impl<T> TileMap<T> {
        #[inline]
        fn get_linear_index(&self, index: TilePos) -> u16 {
            ((self.height - index.y - 1) * self.width + index.x) as u16
        }

        #[inline]
        fn get_tile_pos(&self, index: u16) -> TilePos {
            TilePos {
                x: (index % self.width as u16)               as u8,
                y: self.height - (index / self.width as u16) as u8,
            }
        }
    }

    impl<T, I: Into<TilePos>> Index<I> for TileMap<T> {
        type Output = T;

        #[inline]
        fn index(&self, index: I) -> &Self::Output {
            let index = self.get_linear_index(index.into());
            &self.data[index as usize]
        }
    }

    impl<T, I: Into<TilePos>> IndexMut<I> for TileMap<T> {
        #[inline]
        fn index_mut(&mut self, index: I) -> &mut Self::Output {
            let index = self.get_linear_index(index.into());
            &mut self.data[index as usize]
        }
    }
}