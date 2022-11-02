use std::collections::BTreeMap;

use egui::{Color32, Context, Pos2, Rect, Rounding, Ui, Vec2};
use rand::{rngs::StdRng, Rng};

use crate::geometry::GridPosition;

const DARK_BROWN: Color32 = Color32::from_rgb(0x36, 0x24, 0x19);

#[derive(Clone)]
pub struct Forest {
    grid_width: usize,
    grid_height: usize,
    suceptibility: f64,
    burn_duration: usize,
    rng: StdRng,
    // NB: Rust's HashMap is nondeterministic (as a DoS mitigation). We MUST use an ordered map
    // to get determinstic behavior, even with a seeded RNG. Otherwise our RNG will be generating
    // the same numbers, but we'll be visiting trees in a different order.
    trees: BTreeMap<GridPosition, TreeState>,
    tick: usize,
    burning: isize,
    changeset: Vec<(GridPosition, TreeState)>,
}
impl Forest {
    pub fn new(
        grid_width: usize,
        grid_height: usize,
        suceptibility: f64,
        burn_duration: usize,
        tree_density: f64,
        mut rng: StdRng,
    ) -> Self {
        let mut trees = BTreeMap::default();
        for x in 0..grid_width {
            for y in 0..grid_height {
                if rng.gen_bool(tree_density) {
                    let grid_pos = GridPosition::new(x, y);
                    trees.insert(grid_pos, Default::default());
                }
            }
        }
        trees.insert(
            GridPosition::new(rng.gen_range(0..grid_width), rng.gen_range(0..grid_height)),
            TreeState::Catching,
        );
        // Preallocate a buffer for our changesets between ticks, to avoid allocations during the
        // most intensive parts of our simulation to help keep the animation smooth.
        let changeset = Vec::with_capacity((trees.len() / 10).max(1000));
        Self {
            grid_width,
            grid_height,
            suceptibility,
            burn_duration,
            rng,
            trees,
            tick: 0,
            burning: 1,
            changeset,
        }
    }
    pub fn steady_state(&self) -> bool {
        self.burning <= 0
    }
    fn grid_params(&self, available: Vec2) -> (f32, Rect) {
        let grid_step =
            (available.x / self.grid_width as f32).min(available.y / self.grid_height as f32);
        let horiz_slack = available.x - (grid_step * self.grid_width as f32);
        let vert_slack = available.y - (grid_step * self.grid_height as f32);
        let min = Pos2::new(horiz_slack / 2., vert_slack / 2.);
        let max = Pos2::new(
            min.x + (grid_step * self.grid_width as f32),
            min.y + (grid_step * self.grid_height as f32),
        );

        (grid_step, Rect { min, max })
    }
    pub fn draw(&mut self, ctx: &Context, ui: &Ui) {
        let (grid_step, grid_rect) = self.grid_params(ui.available_size());
        let painter = ui.painter();

        // Background
        painter.rect_filled(grid_rect, Rounding::default(), DARK_BROWN);

        // Trees
        let x_offset = grid_rect.min.x;
        let y_offset = grid_rect.min.y;
        for (grid_pos, state) in self.trees.iter() {
            let x = grid_pos.x as f32;
            let y = grid_pos.y as f32;

            let tree = Rect {
                min: Pos2::new(grid_step * x + x_offset, grid_step * y + y_offset),
                max: Pos2::new(
                    grid_step * (x + 1.) + x_offset,
                    grid_step * (y + 1.) + y_offset,
                ),
            };
            painter.rect_filled(tree, Rounding::default(), state.color());
        }

    }
    pub fn tick(&mut self) {
        for (grid_pos, state) in self.trees.iter() {
            match state {
                TreeState::Uncaught => {
                    let mut probablity_of_remaining_uncaught = 1.;
                    for neighbor in grid_pos.neighbors() {
                        if let Some(neighbor_state) = self.trees.get(&neighbor) && matches!(neighbor_state, TreeState::Burning(_)) {
                            probablity_of_remaining_uncaught *= 1. - self.suceptibility;
                        }
                    }
                    if !self.rng.gen_bool(probablity_of_remaining_uncaught) {
                        self.changeset.push((*grid_pos, TreeState::Catching));
                    }
                }
                TreeState::Catching => {
                    self.changeset.push((*grid_pos, TreeState::Burning(self.tick + self.burn_duration)))
                }
                TreeState::Burning(until) => {
                    if self.tick >= *until {
                        self.changeset.push((*grid_pos, TreeState::Burnt))
                    }
                }
                TreeState::Burnt => (),
            }
        }
        for (grid_pos, state) in self.changeset.drain(..) {
            self.trees.insert(grid_pos, state);
            match state {
                TreeState::Catching => self.burning += 1,
                TreeState::Burnt => self.burning -= 1,
                _ => (),
            }
        }

        self.tick += 1;
    }
}

#[derive(Copy, Clone, Debug, Default)]
enum TreeState {
    #[default]
    Uncaught,
    Catching,
    Burning(usize),
    Burnt,
}
impl TreeState {
    pub fn color(&self) -> Color32 {
        match self {
            TreeState::Uncaught => Color32::DARK_GREEN,
            TreeState::Catching => Color32::DARK_RED,
            TreeState::Burning(_) => Color32::RED,
            TreeState::Burnt => Color32::GRAY,
        }
    }
}
