#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, epaint::Color32};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
                    .with_inner_size([320.0, 240.0])
                    .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:

            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    files: Vec<egui::DroppedFile>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            files: vec![]
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut files_hovering = false;
            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty()
                {
                    self.files = i.raw.dropped_files.clone();
                }

                files_hovering = !i.raw.hovered_files.is_empty();
            });
            
            ui.heading("Hola Natascha");
            

            if self.files.is_empty()
            {
                let msg = "Arrastra los archivos que quieras editar aqui";
                if files_hovering
                {
                    ui.colored_label(Color32::BLUE, msg);
                }
                else
                {
                    ui.label(msg);
                }
            }
            else 
            {
                ui.label("Archivos seleccionados: ");
                for file in self.files.iter()
                {
                    ui.label(
                        file.path.as_ref()
                        .and_then(
                            |pathbuf| 
                            Some(pathbuf.as_path().display().to_string())
                        )
                        .unwrap_or("No pude recuperar el nombre de este archivo :(".to_owned())
                    );
                }
            }

        });
    }
}