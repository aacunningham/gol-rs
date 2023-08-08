use std::{
    io::{Read as IORead, Seek},
    iter::Flatten,
};

use crate::{
    io::GoLFile,
    read::Read as GoLRead,
    utils::{is_alive, neighbor_coordinates},
};

/// An in-memory representation of Conway's Game of Life as a fixed size board with dead and alive
/// cells.
///
/// For a memory efficient equivalent that uses the file system, see [GoLFile].
#[derive(Debug, PartialEq, Eq)]
pub struct GameOfLife {
    width: usize,
    height: usize,
    inner: Vec<Vec<bool>>,
}

impl GameOfLife {
    /// Run the ruleset against the board and create the next iteration.
    pub fn transition(&self) -> Self {
        let mut next = Vec::with_capacity(self.height);
        for y in 0..self.height {
            let mut row = Vec::with_capacity(self.width);
            for x in 0..self.width {
                let cell = self.inner_read_cell(x, y);
                let neighbors = self.inner_read_neighbors(x, y);
                row.push(is_alive(cell, &neighbors));
            }
            next.push(row);
        }
        Self {
            inner: next,
            ..*self
        }
    }

    /// Iterate over the cells of the board, from (0, 0) to `(self.width(), self.height())`
    pub fn iter(&self) -> impl Iterator<Item = bool> + '_ {
        self.inner.iter().flatten().copied()
    }

    fn inner_read_cell(&self, x: usize, y: usize) -> bool {
        self.inner[y][x]
    }

    fn inner_read_neighbors(&self, x: usize, y: usize) -> [Option<bool>; 8] {
        neighbor_coordinates(x, y).map(|b_coord| {
            b_coord.and_then(|coord| {
                self.inner
                    .get(coord.1)
                    .and_then(|column| column.get(coord.0))
                    .cloned()
            })
        })
    }
}

impl From<Vec<Vec<bool>>> for GameOfLife {
    fn from(value: Vec<Vec<bool>>) -> Self {
        let width = value.get(0).map(|row| row.len()).unwrap_or(0);
        let height = value.len();
        GameOfLife {
            inner: value,
            width,
            height,
        }
    }
}

impl<R: IORead + Seek> TryFrom<GoLFile<R>> for GameOfLife {
    type Error = &'static str;

    fn try_from(value: GoLFile<R>) -> Result<Self, Self::Error> {
        let width = value.width();
        let height = value.height();
        let mut inner = Vec::with_capacity(height);
        let mut iter = value.into_iter();
        for _ in 0..height {
            let mut buf = Vec::with_capacity(width);
            for _ in 0..width {
                buf.push(iter.next().ok_or("Iterator ended unexpectedly")?);
            }
            inner.push(buf);
        }
        Ok(Self {
            width,
            height,
            inner,
        })
    }
}

impl GoLRead for GameOfLife {
    fn read_cell(&mut self, x: usize, y: usize) -> Result<bool, &'static str> {
        Ok(self.inner_read_cell(x, y))
    }

    fn read_neighbors(&mut self, x: usize, y: usize) -> Result<[Option<bool>; 8], &'static str> {
        Ok(self.inner_read_neighbors(x, y))
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl IntoIterator for GameOfLife {
    type IntoIter = Flatten<std::vec::IntoIter<Vec<bool>>>;
    type Item = bool;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().flatten()
    }
}
