use crate::{read::Read, utils::is_alive};

pub trait IntoTransitionIter<I> {
    fn into_transition_iter(self) -> TransitionIter<I>;
}

impl<R: Read> IntoTransitionIter<R> for R {
    fn into_transition_iter(self) -> TransitionIter<R> {
        TransitionIter {
            inner: self,
            x: 0,
            y: 0,
        }
    }
}

pub struct TransitionIter<R> {
    inner: R,
    x: usize,
    y: usize,
}
impl<R: Read> Iterator for TransitionIter<R> {
    type Item = (usize, usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == self.inner.height() {
            return None;
        }
        let curr = (self.x, self.y);
        let cell = self.inner.read_cell(curr.0, curr.1).ok()?;
        let neighbors = self.inner.read_neighbors(curr.0, curr.1).ok()?;
        let new_value = is_alive(cell, &neighbors);

        if self.x + 1 == self.inner.width() {
            self.x = 0;
            self.y += 1;
        } else {
            self.x += 1;
        }
        Some((curr.0, curr.1, new_value))
    }
}
