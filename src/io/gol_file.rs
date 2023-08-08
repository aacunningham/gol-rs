use std::io::{self, Read as IORead, Seek, SeekFrom};

use crate::read::Read as GoLRead;
use crate::utils::neighbor_coordinates;

const DEFAULT_BUF_SIZE: usize = 3000;

/// A memory-efficient way to read and interact with a Game Of Life file (.gol) currently saved to
/// disk. Similar to buffered writer, the struct will efficiently use a buffer to try to limit the
/// number of reads made to disk while avoiding bringing the entire file into memory.
#[derive(Debug)]
pub struct GoLFile<R> {
    inner: R,
    cursor: Option<(usize, usize)>,
    buffer: Vec<Vec<Option<bool>>>,
    pub width: usize,
    pub height: usize,
    file_read_count: usize,
    _temp: Vec<u8>,
}
impl<R: IORead + Seek> GoLFile<R> {
    const HEADER_SIZE: usize = 12;

    /// Creates a new GoLFile from a reader. Defaults to a maximum buffer size of 3KB. Actual size
    /// depends on the width/height of the board.
    pub fn new(inner: R) -> Result<Self, &'static str> {
        Self::with_max_capacity(inner, DEFAULT_BUF_SIZE)
    }

    /// Creates a new GoLFile from a reader, capping the buffer size to the provided number of
    /// bytes. Actual size of the buffer will depend on the width/height of the board.
    pub fn with_max_capacity(mut inner: R, max_capacity: usize) -> Result<Self, &'static str> {
        let mut header = [0; 12];
        inner
            .read_exact(&mut header)
            .or(Err("Failed to read header"))?;
        if !header.starts_with(b"GOFL") {
            return Err("File isn't a Game of Life file");
        }
        let width = u32::from_be_bytes(header[4..8].try_into().expect("File should be long enough"))
            as usize;
        let height = u32::from_be_bytes(
            header[8..12]
                .try_into()
                .expect("File should be long enough"),
        ) as usize;
        let (b_width, b_height) = if max_capacity / width >= 3 {
            (width, max_capacity / width)
        } else {
            (max_capacity / 3, 3)
        };

        let mut buffer = Vec::with_capacity(b_height);
        for _ in 0..b_height {
            buffer.push(vec![None; b_width]);
        }
        Ok(GoLFile {
            inner,
            cursor: None,
            buffer,
            width,
            height,
            file_read_count: 0,
            _temp: vec![2; b_width],
        })
    }

    fn load_buffer(&mut self, mut x: usize, mut y: usize) -> io::Result<()> {
        x = usize::min(x, self.width.saturating_sub(self.buffer[0].len()));
        y = usize::min(y, self.height.saturating_sub(self.buffer.len()));
        self.buffer[0].iter_mut().for_each(|v| *v = None);
        self.buffer[1].iter_mut().for_each(|v| *v = None);
        self.buffer[2].iter_mut().for_each(|v| *v = None);
        for i in 0..self.buffer.len() {
            if y + i >= self.height {
                break;
            }
            self.read_row(x, y + i, i)?;
        }
        self.cursor = Some((x, y));
        Ok(())
    }

    fn read_row(&mut self, x: usize, y: usize, buf_index: usize) -> io::Result<()> {
        let buffer_write_range = {
            let end = usize::min(self.width - x, self.buffer[0].len());
            0..end
        };
        self.inner
            .seek(SeekFrom::Start(self.compute_index(x, y) as u64))?;
        self._temp.fill(2);
        self.inner
            .read_exact(&mut self._temp[buffer_write_range.clone()])?;
        let buffer = &mut self.buffer[buf_index];
        for i in buffer_write_range {
            buffer[i] = if self._temp[i] == 2 {
                None
            } else {
                Some(self._temp[i] == 1)
            }
        }
        self.file_read_count += 1;
        Ok(())
    }

    fn compute_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x + Self::HEADER_SIZE
    }

    fn get_buffer_coordinates(
        &mut self,
        x: usize,
        y: usize,
    ) -> Result<(usize, usize), &'static str> {
        if !self.buffer_contains(x, y) {
            self.load_buffer(x.saturating_sub(1), y.saturating_sub(1))
                .or(Err("Failed to read file contents"))?;
        }
        let (c_x, c_y) = self.cursor.expect("Cursor should have been set");
        Ok((x - c_x, y - c_y))
    }

    fn buffer_contains(&self, x: usize, y: usize) -> bool {
        if let Some((c_x, c_y)) = self.cursor {
            let x_in_buffer = {
                if x == 0 || x == self.width - 1 {
                    c_x <= x && x < c_x + self.buffer[0].len()
                } else {
                    c_x < x && x < c_x + self.buffer[0].len() - 1
                }
            };
            let y_in_buffer = {
                if y == 0 || y == self.height - 1 {
                    c_y <= y && y < c_y + self.buffer.len()
                } else {
                    c_y < y && y < c_y + self.buffer.len() - 1
                }
            };
            x_in_buffer && y_in_buffer
        } else {
            false
        }
    }
}

impl<R: IORead + Seek> IntoIterator for GoLFile<R> {
    type IntoIter = IntoIter<R>;
    type Item = bool;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self,
            x: 0,
            y: 0,
        }
    }
}

impl<R: IORead + Seek> GoLRead for GoLFile<R> {
    fn read_cell(&mut self, x: usize, y: usize) -> Result<bool, &'static str> {
        let (b_x, b_y) = self.get_buffer_coordinates(x, y)?;
        self.buffer
            .get(b_y)
            .and_then(|column| column.get(b_x))
            .ok_or("Cell was not in range")?
            .ok_or("Cell shouldn't be empty")
    }

    fn read_neighbors(&mut self, x: usize, y: usize) -> Result<[Option<bool>; 8], &'static str> {
        let (b_x, b_y) = self.get_buffer_coordinates(x, y)?;
        let neighbors = neighbor_coordinates(b_x, b_y).map(|b_coord| {
            b_coord.and_then(|coord| {
                self.buffer
                    .get(coord.1)
                    .and_then(|column| column.get(coord.0))
                    .cloned()
                    .unwrap_or_default()
            })
        });
        Ok(neighbors)
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

/// An iterator over the board.
pub struct IntoIter<R> {
    inner: GoLFile<R>,
    x: usize,
    y: usize,
}
impl<R: IORead + Seek> Iterator for IntoIter<R> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.inner.height() {
            return None;
        }
        let curr = (self.x, self.y);
        let cell = self.inner.read_cell(curr.0, curr.1).ok()?;

        if self.x + 1 == self.inner.width() {
            self.x = 0;
            self.y += 1;
        } else {
            self.x += 1;
        }
        Some(cell)
    }
}
