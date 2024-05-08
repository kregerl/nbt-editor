#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use eframe::egui::{self, Ui};
use egui_dock::{DockArea, DockState, TabViewer};
use log::info;
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
        Box::new(|_cc| Box::new(NBTEditor::new("playerdata.dat").unwrap())),
    )
}

struct Tabs {
    buffers: BTreeMap<String, NBTMap>,
}

impl Tabs {
    pub fn new(title: &str, contents: NBTMap) -> Self {
        let mut map = BTreeMap::new();
        map.insert(title.to_owned(), contents);
        Self { buffers: map }
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
            let counter = 0;
            let x = self.buffers.get(tab).unwrap();
            for (key, value) in &x.content {
                match value {
                    nbt::tag::NBTValue::ByteArray(_) => {
                        NBTEditor::create_nbt_entry(key, value, ui, counter)
                    }
                    nbt::tag::NBTValue::List(_) => {
                        NBTEditor::create_nbt_entry(key, value, ui, counter)
                    }
                    nbt::tag::NBTValue::Compound(_) => {
                        NBTEditor::create_nbt_entry(key, value, ui, counter)
                    }
                    nbt::tag::NBTValue::IntArray(_) => {
                        NBTEditor::create_nbt_entry(key, value, ui, counter)
                    }
                    nbt::tag::NBTValue::LongArray(_) => {
                        NBTEditor::create_nbt_entry(key, value, ui, counter)
                    }
                    _ => NBTEditor::create_nbt_entry(key, value, ui, counter),
                };
            }
        });
    }
}

struct NBTEditor {
    tabs: Tabs,
    state: DockState<String>,
}

const GZIP_SIGNATURE: [u8; 2] = [0x1f, 0x8b];
const ZLIB_SIGNATURES: [[u8; 2]; 4] = [[0x78, 0x01], [0x78, 0x5e], [0x78, 0x9c], [0x78, 0xda]];
impl NBTEditor {
    pub fn new(file_path: &str) -> nbt::Result<Self> {
        Ok(Self {
            tabs: Tabs::new("Testing", Self::read_nbt_file(file_path)?),
            state: DockState::new(vec!["Testing".to_owned()]),
        })
    }

    fn read_nbt_file(file_path: &str) -> nbt::Result<NBTMap> {
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
        info!("NBTMap: {:#?}", nbt_map);
        Ok(nbt_map)
    }

    fn create_nbt_entry(name: &str, tag: &NBTValue, ui: &mut Ui, counter: usize) {
        let label = if !name.is_empty() {
            format!("{}: ", name)
        } else {
            "".into()
        };

        match tag {
            NBTValue::Byte(n) => {
                ui.push_id(counter, |ui| {
                    ui.label(format!("{}{}", label, n));
                });
            }
            NBTValue::Short(n) => {
                ui.label(format!("{}{}", label, n));
            }
            NBTValue::Int(n) => {
                ui.label(format!("{}{}", label, n));
            }
            NBTValue::Long(n) => {
                ui.label(format!("{}{}", label, n));
            }
            NBTValue::Float(n) => {
                ui.label(format!("{}{}", label, n));
            }
            NBTValue::Double(n) => {
                ui.label(format!("{}{}", label, n));
            }
            NBTValue::String(n) => {
                ui.label(n);
            }
            NBTValue::List(list) => {
                let len = list.len();
                Self::push_collapsing(&label, std::iter::zip(vec![""; len], list), counter, ui);
            }
            NBTValue::Compound(map) => {
                let label = if label.is_empty() {
                    format!("{} entries", map.len())
                } else {
                    label
                };
                Self::push_collapsing(
                    &label,
                    map.iter().map(|(name, tag)| (name.as_str(), tag)),
                    counter,
                    ui,
                );
            }
            NBTValue::ByteArray(byte_array) => {
                Self::push_array(byte_array, ui, counter);
            }
            NBTValue::IntArray(int_array) => {
                Self::push_array(int_array, ui, counter);
            }
            NBTValue::LongArray(long_array) => {
                Self::push_array(long_array, ui, counter);
            }
        }
    }

    fn push_array<I: ToString>(array: &[I], ui: &mut Ui, mut counter: usize) {
        counter += 1;
        ui.push_id(counter, |ui| {
            for long in array {
                counter += 1;
                ui.push_id(counter, |ui| {
                    ui.label(long.to_string());
                });
            }
        });
    }

    fn push_collapsing<'a, I>(label: &str, elements: I, mut counter: usize, ui: &mut Ui)
    where
        I: Iterator<Item = (&'a str, &'a NBTValue)>,
    {
        counter += 1;
        ui.push_id(counter, |ui| {
            ui.collapsing(label, |ui| {
                for (key, value) in elements {
                    counter += 1;
                    ui.push_id(counter, |ui| {
                        Self::create_nbt_entry(&key, value, ui, counter);
                    });
                }
            });
        });
    }
}

impl eframe::App for NBTEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
        DockArea::new(&mut self.state)
            .draggable_tabs(false)
            .show(ctx, &mut self.tabs);
    }
}
