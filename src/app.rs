use egui::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ColumnConfig {
    id: String,
    caption: String,
    unit: String,

    edit_caption: bool,
    edit_unit: bool,
}

enum Action {
    Remove,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RowData {
    name: String,

    col_data: HashMap<String, f64>,
    children: Vec<RowData>,

    // UI State
    expanded: bool,
    edit_name: bool,
}

impl RowData {
    fn update(&mut self, column_configs: &Vec<ColumnConfig>) {
        // Update all children
        for child in self.children.iter_mut() {
            child.update(column_configs);
        }

        // Update the actual values
        for col_cfg in column_configs.iter() {
            let col_id = &col_cfg.id;

            if !self.children.is_empty() {
                let mut sum = 0.0;
                for child in self.children.iter() {
                    sum += child.col_data.get(col_id).unwrap_or(&0.0);
                }
                self.col_data.insert(col_id.clone(), sum);
            }
        }
    }

    fn render(
        &mut self,
        ui: &mut Ui,
        column_configs: &Vec<ColumnConfig>,
        indent_level: i32,
    ) -> Option<Action> {
        let mut action = None;

        ui.horizontal(|ui| {
            ui.add_space(10.0 * indent_level as f32);
            ui.expand_button(&mut self.expanded);
            if self.edit_name {
                if ui.text_edit_singleline(&mut self.name).lost_focus() {
                    if !self.name.is_empty() {
                        self.edit_name = false;
                    }
                }
            } else {
                if ui.label(self.name.clone() + ":").double_clicked() {
                    self.edit_name = true;
                }
            }
        });

        let leaf_node = self.children.is_empty();

        for col_cfg in column_configs.iter() {
            let col_id = &col_cfg.id;
            let value = *self.col_data.get(col_id).unwrap_or(&0.0);
            let unit = col_cfg.unit.clone();

            ui.add_space(10.0 * indent_level as f32);
            if leaf_node {
                ui.add(
                    egui::DragValue::new(self.col_data.get_mut(col_id).unwrap())
                        .speed(1.0)
                        .suffix(format!(" {unit}")),
                );
            } else {
                ui.label(format!("{value} {unit}"));
            }
        }
        if ui.button("🗑").clicked() {
            action = Some(Action::Remove);
        }
        ui.end_row();

        // Optionally add the children
        if self.expanded {
            let mut remove_idx = None;
            for (i, child) in self.children.iter_mut().enumerate() {
                match child.render(ui, column_configs, indent_level + 1) {
                    Some(Action::Remove) => remove_idx = Some(i),
                    None => (),
                }
            }

            if let Some(i) = remove_idx {
                self.children.remove(i);
            }

            // Button to add a new element at the same level
            ui.horizontal(|ui| {
                ui.add_space(10.0 * (indent_level + 1) as f32);
                if ui.button("+").clicked() {
                    let mut new_col_data = HashMap::new();
                    for col_cfg in column_configs.iter() {
                        if self.children.is_empty() {
                            new_col_data.insert(
                                col_cfg.id.clone(),
                                *self.col_data.get(&col_cfg.id).unwrap_or(&0.0),
                            );
                        } else {
                            new_col_data.insert(col_cfg.id.clone(), 0.0);
                        }
                    }
                    self.children.push(RowData {
                        name: "".to_owned(),
                        col_data: new_col_data,
                        children: vec![],
                        expanded: false,
                        edit_name: true,
                    });
                }
            });
            ui.end_row();
        }

        return action;
    }
}

// ----------------------------------------------------------------------------

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TreeTablesApp {
    title_text: String,
    column_configs: Vec<ColumnConfig>,
    root_row: RowData,

    #[serde(skip)]
    edit_title_text: bool,
}

impl Default for TreeTablesApp {
    fn default() -> Self {
        Self {
            title_text: "Tree Tables".to_owned(),
            edit_title_text: false,

            column_configs: vec![
                ColumnConfig {
                    id: "cost".to_owned(),
                    caption: "Materialkosten".to_owned(),
                    unit: "€".to_owned(),
                    edit_caption: false,
                    edit_unit: false,
                },
                ColumnConfig {
                    id: "hours".to_owned(),
                    caption: "Arbeitszeit".to_owned(),
                    unit: "h".to_owned(),
                    edit_caption: false,
                    edit_unit: false,
                },
            ],

            root_row: RowData {
                name: "∑".to_owned(),
                col_data: HashMap::from([("hours".to_owned(), 5.0), ("cost".to_owned(), 10.0)]),
                children: vec![
                    RowData {
                        name: "A".to_owned(),
                        col_data: HashMap::from([
                            ("hours".to_owned(), 1.0),
                            ("cost".to_owned(), 8.0),
                        ]),
                        children: vec![
                            RowData {
                                name: "a".to_owned(),
                                col_data: HashMap::from([
                                    ("hours".to_owned(), 1.0),
                                    ("cost".to_owned(), 8.0),
                                ]),
                                children: vec![],
                                expanded: false,
                                edit_name: false,
                            },
                            RowData {
                                name: "b".to_owned(),
                                col_data: HashMap::from([
                                    ("hours".to_owned(), 1.0),
                                    ("cost".to_owned(), 8.0),
                                ]),
                                children: vec![],
                                expanded: false,
                                edit_name: false,
                            },
                            RowData {
                                name: "c".to_owned(),
                                col_data: HashMap::from([
                                    ("hours".to_owned(), 1.0),
                                    ("cost".to_owned(), 8.0),
                                ]),
                                children: vec![],
                                expanded: false,
                                edit_name: false,
                            },
                        ],
                        expanded: false,
                        edit_name: false,
                    },
                    RowData {
                        name: "B".to_owned(),
                        col_data: HashMap::from([
                            ("hours".to_owned(), 1.0),
                            ("cost".to_owned(), 8.0),
                        ]),
                        children: vec![],
                        expanded: false,
                        edit_name: false,
                    },
                ],
                expanded: false,
                edit_name: false,
            },
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
    fn expand_button(&mut self, expanded: &mut bool) -> Response;
}

impl ExpandButton for Ui {
    fn expand_button(&mut self, expanded: &mut bool) -> Response {
        let symbol = if *expanded { "⮩" } else { "➡" };

        let mut response = self.add_enabled(true, egui::SelectableLabel::new(*expanded, symbol));
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

        self.root_row.update(&self.column_configs);

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
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        let file_data = std::fs::read_to_string(path.display().to_string())
                            .expect("Should have been able to read the file");

                        let json_state: TreeTablesApp = serde_json::from_str(file_data.as_str())
                            .expect("JSON data is corrupted.");

                        self.root_row = json_state.root_row;
                        self.column_configs = json_state.column_configs;
                        self.title_text = json_state.title_text;
                    }
                }

                if ui.button("Save as").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        let file_path = path.display().to_string();

                        dbg!(&file_path);

                        if let Ok(mut file) = File::create(file_path) {
                            let res = file.write(serde_json::to_string(&self).unwrap().as_bytes());
                            dbg!(res);
                        }
                    }
                }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                if self.edit_title_text == false {
                    if ui.heading(self.title_text.clone()).double_clicked() {
                        self.edit_title_text = true;
                    }
                } else {
                    let resp = ui.text_edit_singleline(&mut self.title_text);
                    if resp.lost_focus() || resp.clicked_elsewhere() {
                        self.edit_title_text = false;
                    }
                }

                egui::Grid::new("table").show(ui, |ui| {
                    ui.label("");

                    // HEADLINE
                    for cfg in self.column_configs.iter_mut() {
                        let caption = cfg.caption.clone();
                        let unit = cfg.unit.clone();
                        ui.horizontal(|ui| {
                            if !cfg.edit_caption {
                                if ui.label(caption).double_clicked() {
                                    cfg.edit_caption = true;
                                }
                            } else {
                                if ui.text_edit_singleline(&mut cfg.caption).lost_focus() {
                                    if !cfg.caption.is_empty() {
                                        cfg.edit_caption = false;
                                    }
                                }
                            }
                            if !cfg.edit_unit {
                                if ui.label(format!("({unit})")).double_clicked() {
                                    cfg.edit_unit = true;
                                }
                            } else {
                                if ui.text_edit_singleline(&mut cfg.unit).lost_focus() {
                                    cfg.edit_unit = false;
                                }
                            }
                        });
                    }
                    // ui.horizontal(|ui| {
                    //     ui.add_space(20.0);
                    //     if ui.button("+").clicked() {
                    //         // TODO: Add new column!
                    //     }
                    // });
                    ui.end_row();

                    self.root_row.render(ui, &self.column_configs, 0);
                });

                ui.separator();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    egui::warn_if_debug_build(ui);
                    ui.label(
                        RichText::new(format!("tree-tables v{VERSION}"))
                            .text_style(TextStyle::Small),
                    );
                    ui.separator();
                });
            });
        });
    }
}
