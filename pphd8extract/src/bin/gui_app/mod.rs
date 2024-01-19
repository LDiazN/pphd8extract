// All of this code is used to implement the GUI for the gui
// executable file


// Third party imports
use eframe::{self, egui::{self, InputState, RichText, accesskit::Vec2}, epaint::{Color32, Stroke}};

// Rust imports
use std::{fmt::Display, fs::File, default, ops::Mul};

// Local imports
use ::pphd8extract::pphd8parser;

#[derive(Debug)]
pub struct App
{
    pphd8_files : Vec<egui::DroppedFile>,
    are_files_hovering : bool,
    state : AppState
}

#[derive(Debug)]
enum AppState
{
    /// First state, waiting for the user to drop the files
    WaitingForFiles,
    /// Second state, actually processing valid files
    ProcessingFiles,
    /// The process finished, displaying all possible errors and relevant information
    Finished
}

impl eframe::App for App
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input(|i| self.process_input(i));

            ui.heading("PPHD8 Extract");

            

            match self.state
            {
                AppState::WaitingForFiles => self.update_waiting_files(ctx, _frame, ui),
                AppState::ProcessingFiles => self.update_processing_files(ctx, _frame, ui),
                AppState::Finished => self.update_finished(ctx, _frame, ui)
            }
            
        });
        
    }
}

impl App 
{
    fn process_input(&mut self, input_state : &InputState)
    {

        self.are_files_hovering = !input_state.raw.hovered_files.is_empty();

        if self.are_files_hovering {
            self.pphd8_files = input_state.raw.dropped_files.clone();
        }
    }

    fn update_waiting_files(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui)
    {
        // File dropping area
        egui::Frame::central_panel(&egui::Style::default())
            .fill(egui::Color32::DARK_GRAY)
            .rounding(5.0)
            .stroke(Stroke::new(2.0, Color32::GRAY))
            .outer_margin(5.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| 
                {
                    let text = RichText::new("Drop PPHD8 files here")
                        .color(Color32::BLACK)
                        .strong()
                        .size(20.0);

                    ui.label(text);
                });

                let mut space = ui.available_size();
                space.y = space.y * 0.1;
                ui.allocate_space(space)
            });
        
        // Scrollable area with all selected files
        ui.label(
            RichText::new("Selected files")
                .size(18.0)
        );
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
                        if self.pphd8_files.is_empty() 
                        {
                            Self::get_scrollable_area_label("No files selected", ui);
                        }
                        ui.allocate_space(ui.available_size());
                });
            });
        
        // Extraction button
        ui.allocate_ui(ui.available_size() * egui::vec2(1.0, 0.25), |ui|
        {
            ui.horizontal_centered(|ui| {
                ui.add_enabled(self.extraction_ready(), egui::Button::new(
                        RichText::new("Extract").size(18.0)
                    )
                    .min_size(
                        ui.available_size()
                    )
                );
            });
        });

        if self.pphd8_files.is_empty() {
            let msg = "Arrastra los archivos que quieras editar aqui";
            if self.are_files_hovering {
                ui.colored_label(Color32::BLUE, msg);
            } else {
                ui.label(msg);
            }
        } else {
            ui.label("Archivos seleccionados: ");
            for file in self.pphd8_files.iter() {
                ui.label(
                    file.path
                        .as_ref()
                        .map(|pathbuf| pathbuf.as_path().display().to_string())
                        .unwrap_or("No pude recuperar el nombre de este archivo :(".to_owned()),
                );
            }
        }
    }

    fn update_processing_files(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui)
    {

    }

    fn update_finished(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, ui : &mut egui::Ui)
    {

    }

    fn check_file(file: egui::DroppedFile) -> Result<(), FileErrors>
    {
        Ok(())
    }

    fn get_scrollable_area_label<T : Display>(text : T, ui : &mut egui::Ui) -> egui::Response
    {
        ui.label(
            RichText::new(format!("{text}"))
            .size(14.0)
            .color(Color32::BLACK)
        )
    }

    fn extraction_ready(&self) -> bool 
    {
        !self.pphd8_files.is_empty() && !self.are_files_hovering
    }
    
}

/// Invalid file errors
enum FileErrors 
{
    NotAValidPPHD8
}

impl Default for App
{
    fn default() -> Self {
        return Self { 
            pphd8_files: vec![], 
            are_files_hovering: false,
            state: AppState::WaitingForFiles
        }
    }
}

impl Display for FileErrors
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {
            FileErrors::NotAValidPPHD8 => write!(f, "")
        }
        
    }
}