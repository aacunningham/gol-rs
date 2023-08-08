/// A generic way to interact with Game of Life representations.
pub trait Read {
    /// Reads the cell at the coordinates provided.
    fn read_cell(&mut self, x: usize, y: usize) -> Result<bool, &'static str>;
    /// Reads all eight surrounding cells. If on the board's border, None is returned.
    fn read_neighbors(&mut self, x: usize, y: usize) -> Result<[Option<bool>; 8], &'static str>;

    /// The width of the board.
    fn width(&self) -> usize;
    /// The height of the board.
    fn height(&self) -> usize;
}
