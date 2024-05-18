#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use eframe::egui::{self, Id, Ui};
use egui_dock::{DockArea, DockState, TabViewer};
use log::{debug, info};
use nbt::tag::{NBTMap, NBTValue};

fn main() -> Result<(), eframe::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "NBT Editor",
        options,
        // Box::new(|_cc| Box::new(NBTEditor::default())),
        Box::new(|_cc| Box::new(NBTEditor::new(Path::new("playerdata.dat")).unwrap())),
    )
}

#[derive(Default)]
struct Tabs {
    is_editor_window_open: HashMap<Id, bool>,
    buffers: BTreeMap<String, NBTMap>,
}

impl Tabs {
    pub fn new(title: &str, contents: NBTMap) -> Self {
        let mut map = BTreeMap::new();
        map.insert(title.to_owned(), contents);
        Self {
            is_editor_window_open: HashMap::new(),
            buffers: map,
        }
    }

    fn push_nbt_map(&mut self, ui: &mut Ui, tab: &mut <Tabs as TabViewer>::Tab) {
        let map = self.buffers.get_mut(tab).unwrap();
        for (key, value) in &mut map.content {
            Self::push_nbt_value(key, value, ui, &mut self.is_editor_window_open);
        }
    }

    fn push_nbt_value(
        name: &str,
        tag: &mut NBTValue,
        ui: &mut Ui,
        is_editor_window_open: &mut HashMap<Id, bool>,
    ) {
        let label = if !name.is_empty() {
            format!("{}: ", name)
        } else {
            "".into()
        };

        match tag {
            NBTValue::Byte(n) => {
                ui.label(format!("[B] {}{}", label, n));
            }
            NBTValue::Short(n) => {
                ui.label(format!("[S] {}{}", label, n));
            }
            NBTValue::Int(n) => {
                ui.label(format!("[I] {}{}", label, n));
            }
            NBTValue::Long(n) => {
                ui.label(format!("[L] {}{}", label, n));
            }
            NBTValue::Float(n) => {
                let id = ui.next_auto_id();
                let text = format!("[F] {}{}", label, n);

                ui.push_id(id, |ui| {
                    if ui.selectable_label(false, &text).double_clicked() {
                        debug!("Double clicked");
                        is_editor_window_open.insert(id, true);
                    }
                });

                if let Some(open_editor) = is_editor_window_open.get_mut(&id) {
                    let mut contents = String::new();
                    egui::Window::new(&text)
                        .open(open_editor)
                        .show(ui.ctx(), |ui| {
                            ui.label(text);
                            ui.text_edit_singleline(&mut contents);
                        });
                }
            }
            NBTValue::Double(n) => {
                ui.label(format!("[D] {}{}", label, n));
            }
            NBTValue::String(n) => {
                ui.label(n.clone());
            }
            NBTValue::List(list) => {
                let len = list.len();
                Self::push_collapsing(&label, std::iter::zip(vec![""; len], list), ui);
            }
            NBTValue::Compound(map) => {
                let label = if label.is_empty() {
                    format!("{} entries", map.len())
                } else {
                    label
                };
                Self::push_collapsing(
                    &label,
                    map.iter_mut().map(|(name, tag)| (name.as_str(), tag)),
                    ui,
                );
            }
            NBTValue::ByteArray(byte_array) => {
                Self::push_array(byte_array, ui);
            }
            NBTValue::IntArray(int_array) => {
                Self::push_array(int_array, ui);
            }
            NBTValue::LongArray(long_array) => {
                Self::push_array(long_array, ui);
            }
        }
    }

    fn push_array<I: ToString>(array: &[I], ui: &mut Ui) {
        ui.push_id(ui.next_auto_id(), |ui| {
            for long in array {
                ui.push_id(ui.next_auto_id(), |ui| {
                    ui.label(long.to_string());
                });
            }
        });
    }

    fn push_collapsing<'a, I>(label: &str, elements: I, ui: &mut Ui)
    where
        I: Iterator<Item = (&'a str, &'a mut NBTValue)>,
    {
        ui.push_id(ui.next_auto_id(), |ui| {
            ui.collapsing(label, |ui| {
                for (key, value) in elements {
                    ui.push_id(ui.next_auto_id(), |ui| {
                        Self::push_nbt_value(&key, value, ui, &mut HashMap::new());
                    });
                }
            });
        });
    }
}

impl TabViewer for Tabs {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        egui::WidgetText::from(&*tab)
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.heading("NBT Editor");
        egui::ScrollArea::vertical().show(ui, |ui| {
            // let mut map = self.buffers.get_mut(tab).unwrap();
            self.push_nbt_map(ui, tab);
        });
    }
}

// TODO: Make the tabs an option so the program can be opened without an initial file
struct NBTEditor {
    tabs: Tabs,
    state: DockState<String>,
}

impl Default for NBTEditor {
    fn default() -> Self {
        Self {
            tabs: Tabs::default(),
            state: DockState::new(vec![]),
        }
    }
}

const GZIP_SIGNATURE: [u8; 2] = [0x1f, 0x8b];
const ZLIB_SIGNATURES: [[u8; 2]; 4] = [[0x78, 0x01], [0x78, 0x5e], [0x78, 0x9c], [0x78, 0xda]];
impl NBTEditor {
    pub fn new(file_path: &Path) -> nbt::Result<Self> {
        let file_name = match file_path.file_name() {
            Some(s) => s.to_str().unwrap(),
            None => "Untitled",
        };
        Ok(Self {
            tabs: Tabs::new(file_name, Self::read_nbt_file(file_path)?),
            state: DockState::new(vec![file_name.to_owned()]),
        })
    }

    fn add_tab(&mut self, title: &str, contents: NBTMap) {
        self.tabs.buffers.insert(title.to_owned(), contents);
        let mut tabs = self
            .state
            .main_surface()
            .tabs()
            .map(|name| name.to_owned())
            .collect::<Vec<String>>();
        tabs.push(title.into());
        self.state = DockState::new(tabs);
        if let Some(tab_location) = self.state.find_tab(&title.to_owned()) {
            self.state.set_active_tab(tab_location);
        } else {
            self.state.push_to_focused_leaf(title.to_owned());
        }
    }

    /// Reads an NBT file and decompresses it with the correct method
    /// (gzip, zlib) before returning it as a `NBTMap`
    fn read_nbt_file(file_path: &Path) -> nbt::Result<NBTMap> {
        let mut file = File::open(file_path)?;
        let mut buffer = [0u8; 2];
        let _ = file.read_exact(&mut buffer)?;
        let _ = file.seek(SeekFrom::Start(0))?;
        let nbt_map = if buffer == GZIP_SIGNATURE {
            NBTMap::from_gzip_reader(&mut file)?
        } else if ZLIB_SIGNATURES.iter().any(|signature| *signature == buffer) {
            NBTMap::from_zlib_reader(&mut file)?
        } else {
            NBTMap::from_reader(&mut file)?
        };
        debug!("NBTMap: {:#?}", nbt_map);
        Ok(nbt_map)
    }

    fn update_central_panel(&mut self, ctx: &egui::Context) {
        // Tabbed central panel for editing and viewing nbt files
        egui::CentralPanel::default().show(ctx, |ui| {
            for title in self.tabs.buffers.keys() {
                let tab_location = self.state.find_tab(title);
                if ui.selectable_label(tab_location.is_some(), title).clicked() {
                    if let Some(tab_location) = tab_location {
                        self.state.set_active_tab(tab_location);
                    } else {
                        self.state.push_to_focused_leaf(title.clone());
                    }
                }
            }
        });
    }

    fn update_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        info!("New");
                    }

                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let title = path.file_name().unwrap().to_str().unwrap();
                            let nbt = Self::read_nbt_file(&path).unwrap();
                            self.add_tab(title, nbt);
                            // let x = Some(path.display().to_string());
                            info!("Got file path: {:#?}", path);
                            info!("Tabs: {:#?}", self.tabs.buffers);
                            ui.close_menu();
                        }
                    }
                });
            });
        });
    }
}

impl eframe::App for NBTEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_menu_bar(ctx);

        // self.update_side_panel(ctx);

        if self.tabs.buffers.is_empty() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Central panel");
            });
        } else {
            self.update_central_panel(ctx);
            DockArea::new(&mut self.state)
                .draggable_tabs(false)
                .show(ctx, &mut self.tabs);
        }
    }
}
