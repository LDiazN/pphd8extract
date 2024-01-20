// All of this code is used to implement the GUI for the gui
// executable file

// Third party imports
use eframe::{
    self,
    egui::{self, InputState, Layout, RichText},
    emath::Align,
    epaint::{vec2, Color32, Stroke},
};

// Rust imports
use std::{fmt::Display, path::PathBuf};

// Local imports
use ::pphd8extract::pphd8parser;

#[derive(Debug)]
pub struct App {
    pphd8_files: Vec<(egui::DroppedFile, Result<(), FileErrors>)>,
    are_files_hovering: bool,
    state: AppState,
    output_dir: Option<PathBuf>,
}

#[derive(Debug, PartialEq)]
enum AppState {
    /// First state, waiting for the user to drop the files
    WaitingForFiles,
    /// Second state, actually processing valid files
    ProcessingFiles,
    /// The process finished, displaying all possible errors and relevant information
    Finished,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PPHD8 Extract");

            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 8.0);

            match self.state {
                AppState::WaitingForFiles => self.update_waiting_files(ctx, _frame, ui),
                AppState::ProcessingFiles => self.update_processing_files(ctx, _frame, ui),
                AppState::Finished => self.update_finished(ctx, _frame, ui),
            }
        });
    }
}

impl App {
    fn update_waiting_files(
        &mut self,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        _ctx.input_mut(|input_state| self.process_input_waiting_files(input_state));

        // File dropping area
        self.draw_file_dropping_area(ui);

        // Scrollable area with all selected files
        self.draw_scrollable_file_list_area(ui);

        // Selection of output dir
        self.draw_output_dir_field(ui);

        ui.add_space(16.0);

        // Extraction button
        self.draw_start_button(ui);
    }

    fn update_processing_files(
        &mut self,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
    }

    fn update_finished(
        &mut self,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
    }

    fn process_input_waiting_files(&mut self, input_state: &mut InputState) {
        self.are_files_hovering = !input_state.raw.hovered_files.is_empty();

        if !self.are_files_hovering && !input_state.raw.dropped_files.is_empty() {
            self.pphd8_files = input_state
                .raw
                .dropped_files
                .iter()
                .map(|file| (file.clone(), Self::check_file(file)))
                .collect();
            input_state.raw.dropped_files.clear();
        }
    }

    fn check_file(file: &egui::DroppedFile) -> Result<(), FileErrors> {
        // I don't know in which cases this will return None (?)
        let path_buf = file.path.as_ref().unwrap();
        let file_path = path_buf.as_path();

        // Check that this file is a valid pphd
        if let Some(extension) = file_path.extension() {
            if extension != "pphd8" {
                return Err(FileErrors::NotAValidPPHD8);
            }
        } else {
            return Err(FileErrors::NotAValidPPHD8);
        }

        Ok(())
    }

    fn draw_file_dropping_area(&self, ui: &mut egui::Ui) {
        let stroke_color = if self.are_files_hovering {
            Color32::BLUE
        } else {
            Color32::GRAY
        };
        let text_color = if self.are_files_hovering {
            Color32::LIGHT_BLUE
        } else {
            Color32::BLACK
        };
        egui::Frame::central_panel(&egui::Style::default())
            .fill(egui::Color32::DARK_GRAY)
            .rounding(5.0)
            .stroke(Stroke::new(2.0, stroke_color))
            .outer_margin(5.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let text = RichText::new("Drop PPHD8 files here")
                        .color(text_color)
                        .strong()
                        .size(20.0);

                    ui.label(text);
                });

                let mut space = ui.available_size();
                space.y = space.y * 0.1;
                ui.allocate_space(space)
            });
    }

    fn draw_scrollable_file_list_area(&self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Selected files").size(18.0));
        egui::Frame::central_panel(&egui::Style::default())
            .fill(Color32::DARK_GRAY)
            .rounding(5.0)
            .outer_margin(5.0)
            .show(ui, |ui| {
                ui.allocate_space(ui.available_size() * egui::vec2(1.0, 0.0));
                egui::ScrollArea::vertical()
                    .max_width(ui.available_width())
                    .max_height(ui.available_height() * 0.5)
                    .show(ui, |ui| {
                        if !self.are_files_selected() {
                            Self::get_scrollable_area_label("No files selected", ui);
                        } else {
                            for (file, status) in self.pphd8_files.iter() {
                                ui.allocate_ui(ui.available_size() * vec2(1.0, 0.1), |ui| {
                                    ui.columns(2, |columns| {
                                        Self::get_scrollable_area_label(
                                            format!("{}", file.path.as_ref().unwrap().display()),
                                            &mut columns[0],
                                        );

                                        columns[1].with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                ui.add_space(12.0);

                                                match status {
                                                    Ok(()) => {
                                                        ui.label(
                                                            RichText::new("Ok")
                                                                .color(Color32::GREEN),
                                                        );
                                                    }
                                                    Err(e) => {
                                                        ui.label(
                                                            RichText::new(format!("Error: {e}"))
                                                                .color(Color32::DARK_RED),
                                                        );
                                                    }
                                                }
                                            },
                                        );
                                    });
                                });

                                ui.separator();
                            }
                        }
                        ui.allocate_space(ui.available_size());
                    });
            });
        let n_files = self.pphd8_files.len();
        let n_errors = self.pphd8_files.iter().filter(|(_, e)| e.is_err()).count();

        if n_files > 0 {
            ui.label(format!("{n_files} files selected."));
        }

        if n_errors > 0 {
            ui.label(
                RichText::new(format! {"There are {n_errors} files with errors"})
                    .color(Color32::RED),
            );
        }
    }

    fn draw_output_dir_field(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Output Directory:").size(18.0));

        let text = match &self.output_dir {
            Some(p) => format!("Output directory: {}", p.display()),
            None => format!("Output directory..."),
        };

        let btn = egui::Button::new(text).min_size(ui.available_size() * egui::vec2(0.0, 0.2));

        if ui.add(btn).clicked() {
            self.output_dir = rfd::FileDialog::new().pick_folder();
        }
    }

    fn draw_start_button(&self, ui: &mut egui::Ui) {
        ui.allocate_ui(ui.available_size() * egui::vec2(1.0, 0.25), |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_enabled(
                    self.extraction_ready(),
                    egui::Button::new(RichText::new("Extract").size(18.0))
                        .min_size(ui.available_size()),
                );
            });
        });

        if !self.extraction_ready() {
            ui.label("Add files and specify output directory to start extraction process");
        } else {
            ui.label(RichText::new("Ready to go!").color(Color32::GREEN));
        }

        if self.files_with_errors() {
            ui.label(RichText::new("Fix errors to start extraction").color(Color32::RED));
        }
    }

    fn get_scrollable_area_label<T: Display>(text: T, ui: &mut egui::Ui) -> egui::Response {
        ui.label(
            RichText::new(format!("{text}"))
                .size(14.0)
                .color(Color32::BLACK),
        )
    }

    fn extraction_ready(&self) -> bool {
        !self.pphd8_files.is_empty()
            && !self.are_files_hovering
            && self.output_dir.is_some()
            && !self.files_with_errors()
    }

    fn files_with_errors(&self) -> bool {
        self.pphd8_files.iter().any(|(_, e)| e.is_err())
    }

    fn get_output_dir(&self) -> PathBuf {
        debug_assert!(self.state == AppState::WaitingForFiles, "You shouldn't be calling this function while waiting for files since output dir might be not yet selected");
        self.output_dir.as_ref().unwrap().clone()
    }

    #[inline(always)]
    fn are_files_selected(&self) -> bool {
        !self.are_files_hovering && !self.pphd8_files.is_empty()
    }
}

/// Invalid file errors
#[derive(Debug)]
enum FileErrors {
    NotAValidPPHD8,
}

impl Default for App {
    fn default() -> Self {
        return Self {
            pphd8_files: vec![],
            are_files_hovering: false,
            state: AppState::WaitingForFiles,
            output_dir: None,
        };
    }
}

impl Display for FileErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileErrors::NotAValidPPHD8 => write!(f, "Not a valid pphd8 file"),
        }
    }
}
