use std::ops::Range;

#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

impl GridPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    pub fn neighbors(&self) -> MooreNeighborhood {
        MooreNeighborhood::new(*self)
    }
}

pub struct MooreNeighborhood {
    pos: GridPosition,
    delta_idx: Range<usize>,
}

impl MooreNeighborhood {
    const DELTAS: [(isize, isize); 8] = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
        (-1, -1),
        (-1, 1),
        (1, 1),
        (1, -1),
    ];
    pub fn new(pos: GridPosition) -> Self {
        let delta_idx = 0..Self::DELTAS.len();
        Self { pos, delta_idx }
    }
}
impl Iterator for MooreNeighborhood {
    type Item = GridPosition;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(idx) = self.delta_idx.next() {
            let delta = Self::DELTAS[idx];
            let pos_x = self.pos.x as isize;
            let pos_y = self.pos.y as isize;
            match (pos_x.checked_add(delta.0), pos_y.checked_add(delta.1)) {
                (Some(x), Some(y)) if x >= 0 && y >= 0 => {
                    dbg!((x, y));
                    return Some(GridPosition::new(x as usize, y as usize))
                }
                _ => (),
            }
        }
        None
    }
}
