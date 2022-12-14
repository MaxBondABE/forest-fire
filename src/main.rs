#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod forest;
mod geometry;

use std::ops::RangeInclusive;

use eframe::App;
use egui::{panel::Side, ComboBox, Slider};
use forest::Forest;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoroshiro128PlusPlus;
use sha2::{Digest, Sha256};

const GRID_VALUES: RangeInclusive<usize> = 1..=1000;
const GRID_DEFAULT: usize = 100;
const SUCEPTIBILITY_DEFAULT: usize = 35;
const TREE_DENSITY_DEFAULT: usize = 45;
const PERCENTAGE_VALUES: RangeInclusive<usize> = 0..=100;
const PERLIN_SCALE_VALUES: RangeInclusive<f64> = 0.0..=50.0;

pub struct Simulation {
    grid_width: usize,
    grid_height: usize,
    burn_duration: usize,
    suceptibility_pct: usize,
    placement: TreePlacement,
    uniform_density_pct: usize,
    perlin_scale: f64,
    seed: String,
    forest: Option<Forest>,
    running: bool,
}
impl App for Simulation {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::new(Side::Right, "controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Grid Width");
                ui.add(Slider::new(&mut self.grid_width, GRID_VALUES));
            });
            ui.end_row();
            ui.horizontal(|ui| {
                ui.label("Grid Height");
                ui.add(Slider::new(&mut self.grid_height, GRID_VALUES));
            });
            ui.end_row();
            ui.horizontal(|ui| {
                ui.label("Burn duration (ticks)");
                ui.add(Slider::new(&mut self.burn_duration, 1..=100));
            });
            ui.end_row();
            ui.horizontal(|ui| {
                ui.label("Suceptibility (%)");
                ui.add(Slider::new(&mut self.suceptibility_pct, PERCENTAGE_VALUES));
            });
            ui.end_row();
            ComboBox::from_label("Tree placement")
                .selected_text(self.placement.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.placement,
                        TreePlacement::Uniform,
                        TreePlacement::Uniform.label(),
                    );
                    ui.selectable_value(
                        &mut self.placement,
                        TreePlacement::Perlin,
                        TreePlacement::Perlin.label(),
                    );
                });
            match self.placement {
                TreePlacement::Uniform => {
                    ui.horizontal(|ui| {
                        ui.label("Tree Density (%)");
                        ui.add(Slider::new(
                            &mut self.uniform_density_pct,
                            PERCENTAGE_VALUES,
                        ));
                    });
                }
                TreePlacement::Perlin => {
                    ui.horizontal(|ui| {
                        ui.label("Scale");
                        ui.add(Slider::new(&mut self.perlin_scale, PERLIN_SCALE_VALUES));
                    });
                }
            };
            ui.end_row();
            ui.horizontal(|ui| {
                ui.label("Seed");
                ui.text_edit_singleline(&mut self.seed);
            });
            ui.end_row();
            if ui.button("New seed").clicked() {
                let mut trng = rand::thread_rng();
                self.seed = trng.gen::<u64>().to_string();
            }
            ui.end_row();

            let new_sim_label = match self.forest {
                Some(_) => "Restart simulation",
                None => "Start simulation",
            };
            if ui.button(new_sim_label).clicked() {
                self.running = true;
                let mut hasher = Sha256::new();
                hasher.update(self.seed.clone());
                let hash: [u8; 32] = hasher.finalize().into();
                let seed: [u8; 8] = hash[..8].try_into().unwrap();
                let rng = Xoroshiro128PlusPlus::seed_from_u64(u64::from_le_bytes(seed));
                let suceptibility = self.suceptibility_pct as f64 / 100.;
                match self.placement {
                    TreePlacement::Uniform => {
                        let tree_density = self.uniform_density_pct as f64 / 100.;
                        self.forest = Some(Forest::uniform(
                            self.grid_width,
                            self.grid_height,
                            suceptibility,
                            self.burn_duration,
                            tree_density,
                            rng,
                        ))
                    }
                    TreePlacement::Perlin => {
                        self.forest = Some(Forest::perlin(
                            self.grid_width,
                            self.grid_height,
                            suceptibility,
                            self.burn_duration,
                            self.perlin_scale,
                            rng,
                        ))
                    }
                }
            }

            if !self.running {
                if let Some(forest) = self.forest.as_mut() {
                    if !forest.steady_state() {
                        if ui.button("Continue").clicked() {
                            self.running = true;
                        }
                        if ui.button("Step").clicked() {
                            forest.tick();
                        }
                    }
                }
            } else if let Some(forest) = self.forest.as_ref() && !forest.steady_state() {
                if ui.button("Pause").clicked() {
                    self.running = false;
                }
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(forest) = self.forest.as_mut() {
                forest.draw(ctx, ui);
                if !forest.steady_state() && self.running {
                    forest.tick();
                    ctx.request_repaint();
                }
            }
        });
    }
}
impl Default for Simulation {
    fn default() -> Self {
        Self {
            grid_width: GRID_DEFAULT,
            grid_height: GRID_DEFAULT,
            burn_duration: 5,
            suceptibility_pct: SUCEPTIBILITY_DEFAULT,
            placement: TreePlacement::default(),
            uniform_density_pct: TREE_DENSITY_DEFAULT,
            perlin_scale: 15.,
            seed: Default::default(),
            forest: None,
            running: false,
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
enum TreePlacement {
    #[default]
    Uniform,
    Perlin,
}
impl TreePlacement {
    fn label(&self) -> &'static str {
        match self {
            TreePlacement::Uniform => "Uniform",
            TreePlacement::Perlin => "Perlin noise",
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Forest Fire",
        options,
        Box::new(|_cc| Box::new(Simulation::default())),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() -> anyhow::Result<()> {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    let options = eframe::WebOptions::default();
    eframe::start_web(
        "egui-canvas",
        options,
        Box::new(|_cc| Box::new(Simulation::default())),
    )
    .unwrap();
    Ok(())
}
