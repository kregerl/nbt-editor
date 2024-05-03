#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
};

use eframe::egui::{self, CollapsingHeader, Ui};
use log::info;
use nbt::tag::{NBTMap, NBTValue};

fn main() -> Result<(), eframe::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Box::new(NBTEditor::new("playerdata.dat").unwrap())),
    )
}

struct NBTEditor {
    data: NBTMap,
    id: u32,
}

const GZIP_SIGNATURE: [u8; 2] = [0x1f, 0x8b];
const ZLIB_SIGNATURES: [[u8; 2]; 4] = [[0x78, 0x01], [0x78, 0x5e], [0x78, 0x9c], [0x78, 0xda]];
impl NBTEditor {
    pub fn new(file_path: &str) -> nbt::Result<Self> {
        Ok(Self {
            data: Self::read_nbt_file(file_path)?,
            id: 0
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

    fn create_nbt_entry(name: &str, tag: &NBTValue, ui: &mut Ui) {
        let label = if !name.is_empty(){
            format!("{}: ", name)
        } else {
            "".into()
        };


        match tag {
            NBTValue::Byte(n) => {
                ui.label(format!("{}{}", label, n));
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
            NBTValue::ByteArray(byte_array) => {
                Self::create_collapsing_header(name, ui, |ui| {
                    for byte in byte_array {
                        ui.label(byte.to_string());
                    }
                });
            }
            NBTValue::String(n) => {
                ui.label(n);
            }
            NBTValue::List(list) => {
                Self::create_collapsing_header(name, ui, |ui| {
                    for element in list {
                        Self::create_nbt_entry("", element, ui);
                    }
                });
            }
            NBTValue::Compound(map) => {
                let label = if label.is_empty() {
                    format!("{} entries", map.len())
                } else {
                    label
                };

                Self::create_collapsing_header(&label, ui, |ui| {
                    for (key, value) in map {
                        Self::create_nbt_entry(key, value, ui);
                    }
                });
            }
            NBTValue::IntArray(int_array) => {
                Self::create_collapsing_header(name, ui, |ui| {
                    for int in int_array {
                        ui.label(int.to_string());
                    }
                });
            }
            NBTValue::LongArray(long_array) => {
                Self::create_collapsing_header(name, ui, |ui| {
                    for long in long_array {
                        ui.label(long.to_string());
                    }
                });
            }
        }
    }

    fn create_collapsing_header<BodyRet>(
        name: &str,
        ui: &mut Ui,
        body_function: impl FnOnce(&mut Ui) -> BodyRet,
    ) {
        // let id = ui.make_persistent_id(rand::random::<u16>());
        // let id = egui::Id::from_u64(rand::thread_rng().gen());
        CollapsingHeader::new(name).show(ui, body_function);

        // egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            // .show_header(ui, |ui| {
                // ui.label(name); // you can put checkboxes or whatever here
            // })
            // .body(body_function);
    }
}

impl eframe::App for NBTEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            for (key, value) in &self.data.content {
                match value {
                    nbt::tag::NBTValue::ByteArray(_) => Self::create_nbt_entry(key, value, ui),
                    nbt::tag::NBTValue::List(_) => Self::create_nbt_entry(key, value, ui),
                    nbt::tag::NBTValue::Compound(_) => Self::create_nbt_entry(key, value, ui),
                    nbt::tag::NBTValue::IntArray(_) => Self::create_nbt_entry(key, value, ui),
                    nbt::tag::NBTValue::LongArray(_) => Self::create_nbt_entry(key, value, ui),
                    _ => {
                        Self::create_nbt_entry(key, value, ui)
                    }
                };
            }
        });
    }
}
