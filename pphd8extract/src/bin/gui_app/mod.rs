// All of this code is used to implement the GUI for the gui
// executable file

// Third party imports
use eframe::{
    self,
    egui::{self, InputState, Layout, RichText},
    emath::Align,
    epaint::{vec2, Color32, Stroke},
};
use pphd8extract::pphd8parser::PPHD8FileData;
use scc::Queue;

// Rust imports
use rayon::{
    self,
    iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
};
use std::{
    fmt::Display,
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle},
};

// Local imports
use ::pphd8extract::pphd8parser;

#[derive(Debug)]
pub struct App {
    pphd8_files: Vec<(egui::DroppedFile, Result<(), FileErrors>)>,
    are_files_hovering: bool,
    state: AppState,
    output_dir: Option<PathBuf>,
    processing_work: Option<WorkManager>,
}

#[derive(Debug)]
struct WorkManager {
    processing_thread_handle: Option<JoinHandle<()>>,
    work: Arc<Work>,
    files: Vec<(PathBuf, FileState)>,
    generated_files: Vec<(PathBuf, bool)>,
}

#[derive(Debug)]
enum FileState {
    Pending,
    Success,
    Error(pphd8parser::ParseError),
}

#[derive(Debug)]
struct Work {
    pending_files: FileQueue,
    success_files: FileQueue,
    error_files: ErrorQueue,
    generated_files: FileStatusQueue,
}

type FileStatusQueue = Queue<(PathBuf, bool)>; // (file path, is ok)
type FileQueue = Queue<PathBuf>;
type ErrorQueue = Queue<(PathBuf, pphd8parser::ParseError)>;

#[derive(Debug, PartialEq)]
enum AppState {
    /// First state, waiting for the user to drop the files
    WaitingForFiles,
    /// Second state, actually processing valid files
    ProcessingFiles,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 8.0);

            match self.state {
                AppState::WaitingForFiles => self.update_waiting_files(ctx, _frame, ui),
                AppState::ProcessingFiles => self.update_processing_files(ctx, _frame, ui),
            }

            ui.allocate_ui(ui.available_size(), |ui| {
                ui.with_layout(egui::Layout::bottom_up(Align::Center), |ui| {
                    ui.hyperlink_to("Luis Díaz", "https://github.com/LDiazN");
                    ui.label("Written by ");
                });
            });
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
        {
            let work = self.processing_work.as_mut().unwrap();
            let text = if work.is_work_done() {
                "Processing finished!"
            } else {
                "Processing files..."
            };
            ui.label(RichText::new(text).size(18.0));
            work.check_work();
        }

        // Print pending files
        self.draw_processing_files(ui);

        // Print files generated so far
        self.draw_generated_files(ui);

        // Print exit buttons
        self.draw_exit_buttons(ui);
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
                space.y *= 0.1;
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
            None => "Output directory...".to_string(),
        };

        let btn = egui::Button::new(text).min_size(ui.available_size() * egui::vec2(0.0, 0.2));

        if ui
            .add(btn)
            .on_hover_text("Select directory where resulting files will be saved")
            .clicked()
        {
            self.output_dir = rfd::FileDialog::new().pick_folder();
        }
    }

    fn draw_start_button(&mut self, ui: &mut egui::Ui) {
        ui.allocate_ui(ui.available_size() * egui::vec2(1.0, 0.25), |ui| {
            ui.horizontal_centered(|ui| {
                let btn_clicked = ui
                    .add_enabled(
                        self.extraction_ready(),
                        egui::Button::new(RichText::new("Extract").size(18.0))
                            .min_size(ui.available_size()),
                    )
                    .clicked();

                if btn_clicked {
                    self.start_processing();
                }
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

    fn get_scrollable_area_label_colored<T: Display>(
        text: T,
        ui: &mut egui::Ui,
        color: Color32,
    ) -> egui::Response {
        ui.label(RichText::new(format!("{text}")).size(14.0).color(color))
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

    fn start_processing(&mut self) {
        let files = self
            .pphd8_files
            .iter()
            .map(|(f, _)| f.path.clone().unwrap())
            .collect::<Vec<PathBuf>>();

        self.processing_work = Some(WorkManager::new(&files));

        let output_dir = self.get_output_dir();
        
        if let Some(w) = self.processing_work.as_mut()
        {
            w.start_work(output_dir);
        }

        self.state = AppState::ProcessingFiles;
    }

    fn draw_processing_files(&mut self, ui: &mut egui::Ui) {
        let work = self.processing_work.as_mut().unwrap();

        ui.label(RichText::new("Pending files:").size(16.0));
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
                        for (file, state) in work.files.iter() {
                            ui.allocate_ui(ui.available_size() * vec2(1.0, 0.1), |ui| {
                                ui.columns(2, |ui| {
                                    Self::get_scrollable_area_label(file.display(), &mut ui[0]);

                                    ui[1].with_layout(
                                        egui::Layout::right_to_left(Align::Center),
                                        |ui| match state {
                                            FileState::Pending => ui.label(
                                                RichText::new("Pending").color(Color32::YELLOW),
                                            ),
                                            FileState::Success => ui.label(
                                                RichText::new("Success!").color(Color32::GREEN),
                                            ),
                                            FileState::Error(e) => ui.label(
                                                RichText::new(format!("Error: {e}"))
                                                    .color(Color32::DARK_RED),
                                            ),
                                        },
                                    );
                                });
                            });
                            ui.separator();
                        }

                        ui.allocate_space(ui.available_size() * egui::vec2(1.0, 0.3));
                    });
            });
    }

    fn draw_generated_files(&mut self, ui: &mut egui::Ui) {
        let work = self.processing_work.as_mut().unwrap();

        ui.label(RichText::new("Generated Files").size(18.0));
        egui::Frame::central_panel(&egui::Style::default())
            .fill(Color32::DARK_GRAY)
            .rounding(5.0)
            .outer_margin(5.0)
            .show(ui, |ui| {
                ui.push_id(ui.next_auto_id(), |ui| {
                    ui.allocate_space(ui.available_size() * egui::vec2(1.0, 0.0));
                    egui::ScrollArea::vertical()
                        .max_width(ui.available_width())
                        .max_height(ui.available_height() * 0.7)
                        .show(ui, |ui| {
                            for (file, was_ok) in work.generated_files.iter() {
                                ui.allocate_ui(ui.available_size() * vec2(1.0, 0.1), |ui| {
                                    if *was_ok {
                                        Self::get_scrollable_area_label(
                                            format!("{}", file.display()),
                                            ui,
                                        );
                                    } else {
                                        Self::get_scrollable_area_label_colored(
                                            format!("{}: Could not write file", file.display()),
                                            ui,
                                            Color32::DARK_RED,
                                        );
                                    }
                                });

                                ui.separator();
                            }
                        });
                });
            });
    }

    fn draw_exit_buttons(&mut self, ui: &mut egui::Ui) {
        let work = self.processing_work.as_ref().unwrap();
        let reset_enabled = work.processing_thread_handle.is_some()
            && work
                .processing_thread_handle
                .as_ref()
                .unwrap()
                .is_finished();

        ui.add_enabled_ui(reset_enabled, |ui| {
            if ui
                .button(RichText::new("Finish").size(18.0))
                .on_hover_text("Go back to the file selection screen")
                .clicked()
            {
                self.reset();
            }
        });
    }

    fn reset(&mut self) {
        self.state = AppState::WaitingForFiles;
        self.processing_work = None;
        self.output_dir = None;
        self.pphd8_files = vec![];
    }
}

impl WorkManager {
    fn new(files: &Vec<PathBuf>) -> Self {
        let pending_files = Queue::default();
        for file in files {
            pending_files.push(file.clone());
        }

        WorkManager {
            processing_thread_handle: None,
            work: Arc::new(Work {
                pending_files,
                success_files: Queue::default(),
                error_files: Queue::default(),
                generated_files: Queue::default(),
            }),
            files: files
                .iter()
                .map(|f| (f.clone(), FileState::Pending))
                .collect(),
            generated_files: vec![],
        }
    }

    fn start_work(&mut self, output_dir: PathBuf) {
        let work = self.work.clone();
        self.processing_thread_handle = Some(thread::spawn(move || {
            work.do_work(output_dir);
        }));
    }

    fn check_work(&mut self) {
        // fill errors
        while let Some(v) = self.work.error_files.pop() {
            let (v, err) = &(**v);
            for i in 0..self.files.len() {
                let (p, _) = &self.files[i];
                if p != v {
                    continue;
                }
                self.files[i].1 = FileState::Error(err.clone());
            }
        }

        // Fill successful files
        while let Some(v) = self.work.success_files.pop() {
            let v = &(**v);
            for i in 0..self.files.len() {
                let (p, _) = &self.files[i];
                if p != v {
                    continue;
                }
                self.files[i].1 = FileState::Success;
            }
        }

        // Fill generated files
        while let Some(v) = self.work.generated_files.pop() {
            self.generated_files.push((**v).clone());
        }
    }

    fn is_work_done(&self) -> bool {
        self.files.iter().all(|(_, state)| !matches!(state, FileState::Pending))
    }
}

impl Work {
    fn do_work(&self, output_dir: PathBuf) {
        let mut files_to_process = vec![];
        let output_dir = output_dir.as_path();

        while let Some(file) = self.pending_files.pop() {
            files_to_process.push(file);
        }

        files_to_process
            .par_iter()
            .map(|file| {
                // parse pphd file
                let pphd_filepath = file.as_path();
                let pphd8_file = match PPHD8FileData::parse_from_file(pphd_filepath) {
                    Err(e) => {
                        self.error_files.push(((***file).clone(), e));
                        return;
                    }
                    Ok(file) => file,
                };

                // extract vag files
                let vags = match pphd8_file.get_vag_files() {
                    Err(e) => {
                        self.error_files.push(((***file).clone(), e));
                        return;
                    }
                    Ok(vags) => vags,
                };

                // save vag files
                vags.par_iter()
                    .enumerate()
                    .map(|(i, file)| {
                        let mut filename = pphd_filepath
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                        filename.push_str(format!("extracted_{i}.vag").as_str());
                        let filepath = output_dir.join(filename);
                        // TODO handle this error as well
                        let result = file.write_to_file(filepath.as_path());
                        self.generated_files.push((filepath, result.is_ok()));
                    })
                    .collect::<Vec<()>>();

                self.success_files.push((***file).clone());
            })
            .collect::<Vec<()>>();
    }
}

/// Invalid file errors
#[derive(Debug)]
enum FileErrors {
    NotAValidPPHD8,
}

impl Default for App {
    fn default() -> Self {
        Self {
            pphd8_files: vec![],
            are_files_hovering: false,
            state: AppState::WaitingForFiles,
            output_dir: None,
            processing_work: None,
        }
    }
}

impl Display for FileErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileErrors::NotAValidPPHD8 => write!(f, "Not a valid pphd8 file"),
        }
    }
}
