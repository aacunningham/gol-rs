//! # .gol File Format
//! The .gol file format is straightforward and easy to read/write.
//!
//! It begins with a 4 byte header, the ASCII values for GOFL, followed by two 32 bit big-endian
//! integers. The first integer is the width of the board and the second integer is the height.
//!
//! All data remaining after those initial 12 bytes are the cells of the board. Each byte is either
//! a 0 (for dead) or 1 (for alive), and there should be width * height bytes.
//!
//! There is no terminator for the file.

pub mod gol_file;

pub use gol_file::GoLFile;

use std::io::{self, Write};

use crate::game_of_life::GameOfLife;
use crate::read::Read;

/// Create a GameOfLife by reading .gol formatted data.
pub fn read_gol(mut input: impl io::Read) -> Result<GameOfLife, &'static str> {
    let (width, height) = {
        let mut header = [0; 12];
        input
            .read_exact(&mut header)
            .or(Err("Failed to read header"))?;
        if !header.starts_with(b"GOFL") {
            return Err("File isn't a Game of Life file");
        }
        let w = u32::from_be_bytes(header[4..8].try_into().expect("File should be long enough"))
            as usize;
        let h = u32::from_be_bytes(
            header[8..12]
                .try_into()
                .expect("File should be long enough"),
        ) as usize;
        (w, h)
    };
    let mut result = Vec::with_capacity(height);
    let mut buffer = vec![0; width];
    while let Ok(()) = input.read_exact(&mut buffer) {
        result.push(buffer.iter().map(|i| *i == 1).collect())
    }
    Ok(result.into())
}

/// Writes a .gol file to output.
pub fn write_gol(mut output: impl io::Write, state: &GameOfLife) -> Result<(), &'static str> {
    let width = state.width() as u32;
    let height = state.height() as u32;
    let mut output_vec = Vec::with_capacity(12 + width as usize * height as usize);
    output_vec.extend_from_slice(b"GOFL");
    output_vec.extend_from_slice(&width.to_be_bytes());
    output_vec.extend_from_slice(&height.to_be_bytes());
    output_vec.extend(state.iter().map(u8::from));
    output
        .write_all(&output_vec)
        .or(Err("Failed writing the data"))
}

/// Writes a .gol file to output, but using an iterator as input. This is useful for [GoLFile]
/// and the [TransitionIter](crate::iter::TransitionIter)
pub fn write_gol_iterator(
    output: impl io::Write,
    width: u32,
    height: u32,
    data: impl Iterator<Item = bool>,
) -> Result<(), &'static str> {
    let mut output = io::BufWriter::new(output);
    output
        .write(b"GOFL")
        .and_then(|_| output.write(&width.to_be_bytes()))
        .and_then(|_| output.write(&height.to_be_bytes()))
        .and_then(|_| {
            data.map(|b| output.write(&[u8::from(b)]))
                .collect::<Result<Vec<_>, _>>()
        })
        .and(Ok(()))
        .or(Err("Failed to write file"))
}

#[allow(unused)]
fn generate_gol(width: u32, height: u32) -> GameOfLife {
    let mut data = Vec::with_capacity(height as usize);
    for _ in 0..10000 {
        data.push(vec![false; width as usize]);
    }
    data.into()
}
