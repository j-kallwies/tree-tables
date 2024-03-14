use egui::*;
use egui_keybind::{Bind, Shortcut};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;
use uuid::Uuid;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const VALID_FILE_EXTENSIONS: [&'static str; 3] = ["tt", "json", "ttree"];

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
pub enum ColumnType {
    Number,
    Text,
    Formula,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ColumnConfig {
    id: String,
    caption: String,
    unit: String,
    col_type: ColumnType,
}

enum Action {
    Modified,
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

            if self.col_data.get(col_id).is_none() {
                self.col_data.insert(col_id.clone(), 0.0);
            }

            ui.add_space(10.0 * indent_level as f32);
            if leaf_node {
                if ui
                    .add(
                        egui::DragValue::new(self.col_data.get_mut(col_id).unwrap())
                            .speed(1.0)
                            .suffix(format!(" {unit}")),
                    )
                    .changed()
                {
                    action = Some(Action::Modified);
                }
            } else {
                ui.label(format!("{value} {unit}"));
            }
        }

        // Remove row button
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
                    Some(Action::Modified) => action = Some(Action::Modified),
                    None => (),
                }
            }

            if let Some(i) = remove_idx {
                self.children.remove(i);

                // Removing a children, means that something changed!
                action = Some(Action::Modified);
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

                    action = Some(Action::Modified);
                }
            });
            ui.end_row();
        }

        return action;
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct TreeTable {
    title_text: String,
    column_configs: Vec<ColumnConfig>,
    root_row: RowData,
}

impl TreeTable {
    fn save_to_file(&self, file_path: &str) {
        if let Ok(mut file) = File::create(file_path) {
            let _res = file.write(serde_json::to_string(&self).unwrap().as_bytes());
        }
    }
}

// ----------------------------------------------------------------------------

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TreeTablesApp {
    #[serde(skip)]
    tree_table: TreeTable,

    #[serde(skip)]
    filename: String,

    #[serde(skip)]
    file_modified: bool,

    #[serde(skip)]
    edit_title_text: bool,

    #[serde(skip)] // TODO: Implement serialization
    save_shortcut: Shortcut,

    #[serde(skip)]
    edit_column_idx: Option<usize>,

    #[serde(skip)]
    close_requested: bool,
}

impl Default for ColumnConfig {
    fn default() -> ColumnConfig {
        ColumnConfig {
            id: Uuid::new_v4().to_string(),
            caption: "".to_owned(),
            unit: "€".to_owned(),
            col_type: ColumnType::Number,
        }
    }
}

impl Default for TreeTablesApp {
    fn default() -> Self {
        Self {
            filename: "unnamed.tt".to_owned(),
            file_modified: true,
            tree_table: TreeTable {
                title_text: "Tree Tables".to_owned(),
                column_configs: vec![
                    ColumnConfig {
                        id: "2387c84a-2c68-405e-a342-d94a1dde6408".to_owned(),
                        caption: "Materialkosten".to_owned(),
                        unit: "€".to_owned(),
                        col_type: ColumnType::Number,
                    },
                    ColumnConfig {
                        id: "94869fe6-c736-4c88-be7f-8084679d78fc".to_owned(),
                        caption: "Arbeitszeit".to_owned(),
                        unit: "h".to_owned(),
                        col_type: ColumnType::Number,
                    },
                ],

                root_row: RowData {
                    name: "∑".to_owned(),
                    col_data: HashMap::from([]),
                    children: vec![RowData {
                        name: "A".to_owned(),
                        col_data: HashMap::from([]),
                        children: vec![],
                        expanded: false,
                        edit_name: false,
                    }],
                    expanded: false,
                    edit_name: false,
                },
            },
            edit_title_text: false,
            save_shortcut: Shortcut::new(
                Some(egui::KeyboardShortcut::new(
                    egui::Modifiers::COMMAND,
                    egui::Key::S,
                )),
                None,
            ),
            edit_column_idx: None,
            close_requested: false,
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

        // Show a confirmation dialog when the close event is detected
        if ctx.input(|i| i.viewport().close_requested()) {
            egui::CentralPanel::default().show(ctx, |_ui| {
                if self.file_modified {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                self.close_requested = true;
            });
        }

        self.tree_table
            .root_row
            .update(&self.tree_table.column_configs);

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
            // ui.label("A simple keybind:");
            // let response = ui.add(Keybind::new(&mut self.save_shortcut, "example_keybind"));
            // if response.changed() {
            //     println!("Save shortcut changed!");
            // }

            if self.close_requested {
                egui::Window::new("Unsaved changes").show(ctx, |ui| {
                    ui.label(
                        "You still have unsaved changes. Do you want to save them before you quit?",
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Yes, save!").clicked() {
                            self.tree_table.save_to_file(self.filename.as_str());
                            self.file_modified = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("No, revert all changes and quit!").clicked() {
                            self.file_modified = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            };

            // let keybind_text = self.save_shortcut.format(&egui::ModifierNames::NAMES, true);
            if ctx.input_mut(|i| self.save_shortcut.pressed(i)) {
                self.tree_table.save_to_file(self.filename.as_str());
                self.file_modified = false;
            }

            ui.label(
                egui::RichText::new(format!(
                    "{}{}",
                    self.filename,
                    if self.file_modified {
                        "*".to_owned()
                    } else {
                        "".to_owned()
                    }
                ))
                .monospace(),
            );

            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Tree-Tables", &VALID_FILE_EXTENSIONS)
                        .pick_file()
                    {
                        let file_data = std::fs::read_to_string(path.display().to_string())
                            .expect("Should have been able to read the file");

                        let json_state: TreeTable = serde_json::from_str(file_data.as_str())
                            .expect("JSON data is corrupted.");

                        self.tree_table = json_state;
                        self.filename = path.display().to_string();
                        self.file_modified = false;
                    }
                }

                if ui.button("Save").clicked() {
                    self.tree_table.save_to_file(self.filename.as_str());
                    self.file_modified = false;
                }

                if ui.button("Save as").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Tree-Tables", &VALID_FILE_EXTENSIONS)
                        .save_file()
                    {
                        // Ensure the ".tt" extension
                        let mut path = path;
                        path.set_extension("tt");

                        self.filename = path.display().to_string();
                        self.tree_table.save_to_file(self.filename.as_str());
                        self.file_modified = false;
                    }
                }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                if self.edit_title_text == false {
                    if ui
                        .heading(self.tree_table.title_text.clone())
                        .double_clicked()
                    {
                        self.edit_title_text = true;
                    }
                } else {
                    let resp = ui.text_edit_singleline(&mut self.tree_table.title_text);
                    if resp.lost_focus() || resp.clicked_elsewhere() {
                        self.edit_title_text = false;
                    }
                }

                egui::Grid::new("table").show(ui, |ui| {
                    ui.label("");

                    // HEADLINE
                    for (col_idx, cfg) in self.tree_table.column_configs.iter().enumerate() {
                        let caption = cfg.caption.clone();
                        let unit = cfg.unit.clone();
                        ui.horizontal(|ui| {
                            if ui.label(format!("{caption} ({unit})")).double_clicked() {
                                self.edit_column_idx = Some(col_idx);
                            }
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        if ui.button("+").clicked() {
                            self.edit_column_idx = Some(self.tree_table.column_configs.len());

                            self.tree_table.column_configs.push(ColumnConfig::default());
                        }
                    });
                    ui.end_row();

                    match self
                        .tree_table
                        .root_row
                        .render(ui, &self.tree_table.column_configs, 0)
                    {
                        Some(Action::Modified) => {
                            self.file_modified = true;
                        }
                        Some(Action::Remove) => {}
                        None => {}
                    }
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

        if self.edit_column_idx.is_some() {
            egui::Window::new("Edit column").show(ctx, |ui| {
                egui::Grid::new("edit_column_table").show(ui, |ui| {
                    // ui.label("ID");
                    // ui.add_sized(
                    //     [140.0, 20.0],
                    //     egui::TextEdit::singleline(
                    //         &mut self
                    //             .tree_table
                    //             .column_configs
                    //             .get_mut(self.edit_column_idx.unwrap())
                    //             .unwrap()
                    //             .id,
                    //     ),
                    // );
                    // ui.end_row();

                    ui.label("Title");
                    ui.add_sized(
                        [140.0, 20.0],
                        egui::TextEdit::singleline(
                            &mut self
                                .tree_table
                                .column_configs
                                .get_mut(self.edit_column_idx.unwrap())
                                .unwrap()
                                .caption,
                        ),
                    );
                    ui.end_row();

                    ui.label("Unit");
                    ui.text_edit_singleline(
                        &mut self
                            .tree_table
                            .column_configs
                            .get_mut(self.edit_column_idx.unwrap())
                            .unwrap()
                            .unit,
                    );
                    ui.end_row();
                });

                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        self.edit_column_idx = None;
                    }
                    ui.add_space(200.0);
                    if ui
                        .button(RichText::new("🗑").color(egui::Color32::RED))
                        .clicked()
                    {
                        self.tree_table
                            .column_configs
                            .remove(self.edit_column_idx.unwrap());

                        // dbg!(&self.tree_table.column_configs);

                        self.edit_column_idx = None;
                    }
                });
            });
        }
    }
}
