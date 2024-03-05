use egui::*;
use std::collections::HashMap;
use std::vec::Vec;

pub struct ColumnConfig {
    caption: String,
    unit: String,
}

pub struct RowData {
    col_data: HashMap<String, f64>,
    // sum_col_data: HashMap<String, f64>,
    children: Vec<RowData>,
}

// ----------------------------------------------------------------------------

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TreeTablesApp {
    #[serde(skip)]
    column_config: HashMap<String, ColumnConfig>,

    #[serde(skip)]
    root_row: RowData,

    // Example stuff
    dummy_bool: bool,
    dummy_usize: usize,
    checklist: [bool; 3],
    num_columns: usize,
    expanded: [bool; 7],
    price: [f32; 5],
    hours: [f32; 5],
}

impl Default for TreeTablesApp {
    fn default() -> Self {
        Self {
            column_config: HashMap::from([
                (
                    "A".to_owned(),
                    ColumnConfig {
                        caption: "foo".to_owned(),
                        unit: " h".to_owned(),
                    },
                ),
                (
                    "B".to_owned(),
                    ColumnConfig {
                        caption: "bar".to_owned(),
                        unit: " €".to_owned(),
                    },
                ),
            ]),

            root_row: RowData {
                col_data: HashMap::from([]),
                children: vec![],
            },

            // Example stuff:
            expanded: [false, false, false, false, false, false, false],
            dummy_bool: false,
            dummy_usize: 10,
            checklist: std::array::from_fn(|i| i == 0),
            num_columns: 3,
            price: [0.0, 42.0, 0.0, 0.0, 12.0],
            hours: [0.0, 42.0, 0.0, 0.0, 12.0],
        }
    }
}

impl TreeTablesApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

trait ExpandButton {
    fn expand_button(&mut self, expanded: &mut bool, enabled: bool) -> Response;
}

impl ExpandButton for Ui {
    fn expand_button(&mut self, expanded: &mut bool, enabled: bool) -> Response {
        let symbol = if *expanded { "⮩" } else { "➡" };
        // self.toggle_value(expanded, symbol)

        let mut response = self.add_enabled(enabled, egui::SelectableLabel::new(*expanded, symbol));
        if response.clicked() {
            *expanded = !*expanded;
            response.mark_changed();
        }
        response
    }
}

impl eframe::App for TreeTablesApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                ui.heading("Tree Tables");

                egui::Grid::new("some_unique_id").show(ui, |ui| {
                    ui.label("");
                    ui.label("Materialkosten (€)");
                    ui.label("Arbeitszeit (Std.)");
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.expand_button(&mut self.expanded[0], true);
                        ui.label("∑:");
                    });
                    let price_sum = self.price[0]
                        + self.price[1]
                        + self.price[2]
                        + self.price[3]
                        + self.price[4];
                    let hours_sum = self.hours[0]
                        + self.hours[1]
                        + self.hours[2]
                        + self.hours[3]
                        + self.hours[4];
                    ui.label(format!("{price_sum} €"));
                    ui.label(format!("{hours_sum} Std."));
                    ui.end_row();

                    if self.expanded[0] {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.expand_button(&mut self.expanded[1], true);
                            ui.label("Foo:");
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            let sum = self.price[0] + self.price[1] + self.price[2];
                            ui.label(format!("{sum} €"));
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            let sum = self.hours[0] + self.hours[1] + self.hours[2];
                            ui.label(format!("{sum} Std."));
                        });
                        ui.end_row();

                        if self.expanded[1] {
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.expand_button(&mut self.expanded[2], false);
                                ui.label("C:");
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.add(
                                    egui::DragValue::new(&mut self.price[0])
                                        .speed(1.0)
                                        .suffix(" €"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.add(
                                    egui::DragValue::new(&mut self.hours[0])
                                        .speed(1.0)
                                        .suffix(" Std."),
                                );
                            });
                            ui.end_row();

                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.expand_button(&mut self.expanded[3], false);
                                ui.label("Rust:");
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.add(
                                    egui::DragValue::new(&mut self.price[1])
                                        .speed(1.0)
                                        .suffix(" €"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.add(
                                    egui::DragValue::new(&mut self.hours[1])
                                        .speed(1.0)
                                        .suffix(" Std."),
                                );
                            });
                            ui.end_row();

                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.expand_button(&mut self.expanded[4], false);
                                ui.label("C++:");
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.add(
                                    egui::DragValue::new(&mut self.price[2])
                                        .speed(1.0)
                                        .suffix(" €"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.add(
                                    egui::DragValue::new(&mut self.hours[2])
                                        .speed(1.0)
                                        .suffix(" Std."),
                                );
                            });
                            ui.end_row();
                        }

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.expand_button(&mut self.expanded[5], false);
                            ui.label("Java:");
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.add(
                                egui::DragValue::new(&mut self.price[3])
                                    .speed(1.0)
                                    .suffix(" €"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.add(
                                egui::DragValue::new(&mut self.hours[3])
                                    .speed(1.0)
                                    .suffix(" Std."),
                            );
                        });
                        ui.end_row();

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.expand_button(&mut self.expanded[6], false);
                            ui.label("JavaScript:");
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.add(
                                egui::DragValue::new(&mut self.price[4])
                                    .speed(1.0)
                                    .suffix(" €"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.add(
                                egui::DragValue::new(&mut self.hours[4])
                                    .speed(1.0)
                                    .suffix(" Std."),
                            );
                        });
                        ui.end_row();
                    }
                });

                ui.separator();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }
}
