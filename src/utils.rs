/// Adjusts the coordinate by the provided values, only returning Some if
/// adjustments in both directions succeed.
pub(crate) fn adjust_coordinate(
    coordinate: (usize, usize),
    dx: isize,
    dy: isize,
) -> Option<(usize, usize)> {
    match (
        coordinate.0.checked_add_signed(dx),
        coordinate.1.checked_add_signed(dy),
    ) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

/// Returns an array of all 8 neighbor coordinates for the provided coordinate. If any
/// neighbor coordinate would be invalid (i.e. a negative value), None is returned instead.
pub(crate) fn neighbor_coordinates(x: usize, y: usize) -> [Option<(usize, usize)>; 8] {
    [
        adjust_coordinate((x, y), -1, -1),
        adjust_coordinate((x, y), 0, -1),
        adjust_coordinate((x, y), 1, -1),
        adjust_coordinate((x, y), 1, 0),
        adjust_coordinate((x, y), 1, 1),
        adjust_coordinate((x, y), 0, 1),
        adjust_coordinate((x, y), -1, 1),
        adjust_coordinate((x, y), -1, 0),
    ]
}

/// Game of Life aliveness check, returns whether or not the provided cell survives, dies, or comes
/// to life.
/// * If `current == true` and neighbors includes 2 or 3 `Some(true)`, the cell survives
/// * If `current == true` but neighbors doesn't meet that criteria, the cell dies
/// * If `current != true` and neighbors includes 3 `Some(true)`, the cell comes to life
/// * If none of those criteria are met, the cell remains dead
pub(crate) fn is_alive(current: bool, neighbors: &[Option<bool>]) -> bool {
    let alive_neighbors = neighbors.iter().flatten().filter(|n| **n).count();
    if current && (2..=3).contains(&alive_neighbors) {
        true
    } else {
        !current && alive_neighbors == 3
    }
}
