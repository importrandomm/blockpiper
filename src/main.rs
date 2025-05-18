mod compressor;
mod grammar;
mod ctw;
mod arithmetic;

use eframe::{egui, App};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::compressor::compressor::{compress_file, decompress_file, serialize_grammar};

struct BlockPiperApp {
    input_path: String,
    status: Arc<Mutex<String>>,
    compressing: bool,
    progress: Arc<Mutex<f32>>,
    decompress_input: String,
    decompress_output: String,
    decompress_status: Arc<Mutex<String>>,
    decompressing: bool,
}

impl Default for BlockPiperApp {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            status: Arc::new(Mutex::new(String::new())),
            compressing: false,
            progress: Arc::new(Mutex::new(0.0)),
            decompress_input: String::new(),
            decompress_output: String::new(),
            decompress_status: Arc::new(Mutex::new(String::new())),
            decompressing: false,
        }
    }
}

// Custom compress_file_with_progress function
fn compress_file_with_progress(input: String, output: String, progress: Arc<Mutex<f32>>, status: Arc<Mutex<String>>) {
    use std::fs::File;
    use std::io::{Read, BufReader};
    let block_size = 256 * 1024;
    let mut input_file = match File::open(&input) {
        Ok(f) => f,
        Err(e) => {
            let mut status_lock = status.lock().unwrap();
            *status_lock = format!("Compression failed: {}", e);
            return;
        }
    };
    let mut reader = BufReader::new(&mut input_file);
    let mut blocks = Vec::new();
    let mut buffer = vec![0u8; block_size];
    let mut total_bytes = 0;
    loop {
        let bytes_read = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                let mut status_lock = status.lock().unwrap();
                *status_lock = format!("Compression failed: {}", e);
                return;
            }
        };
        if bytes_read == 0 { break; }
        total_bytes += bytes_read;
        blocks.push(buffer[..bytes_read].to_vec());
    }
    let num_blocks = blocks.len();
    let mut completed = 0;
    let mut compressed_blocks = Vec::new();
    for block_data in blocks {
        match compress_file_block(&block_data) {
            Ok(b) => compressed_blocks.push(b),
            Err(e) => {
                let mut status_lock = status.lock().unwrap();
                *status_lock = format!("Compression failed: {}", e);
                return;
            }
        }
        completed += 1;
        let mut progress_lock = progress.lock().unwrap();
        *progress_lock = completed as f32 / num_blocks as f32;
    }
    // Write output
    match std::fs::File::create(&output) {
        Ok(mut f) => {
            for block in compressed_blocks {
                use std::io::Write;
                let _ = f.write_all(&block);
            }
            let mut status_lock = status.lock().unwrap();
            *status_lock = format!("Compression complete! Output: {}", output);
        }
        Err(e) => {
            let mut status_lock = status.lock().unwrap();
            *status_lock = format!("Compression failed: {}", e);
        }
    }
    let mut progress_lock = progress.lock().unwrap();
    *progress_lock = 1.0;
}

// Helper to compress a single block (reuse your existing logic)
fn compress_file_block(block_data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use crate::grammar::Grammar;
    use crate::ctw::Ctw;
    use crate::arithmetic::ArithmeticEncoder;
    let mut grammar = Grammar::new();
    grammar.infer_grammar(block_data);
    let symbol_stream = serialize_grammar(&grammar);
    let mut ctw = Ctw::new();
    let mut encoder = ArithmeticEncoder::new();
    for &symbol in symbol_stream.iter() {
        let (cum, total) = ctw.get_cumulative(symbol);
        encoder.encode_symbol(symbol, (cum, total));
        ctw.process_symbol(symbol);
    }
    Ok(encoder.finish())
}

impl App for BlockPiperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BlockPiper File Compressor");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Input file:");
                ui.text_edit_singleline(&mut self.input_path);
                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.input_path = path.display().to_string();
                    }
                }
            });

            // Progress bar for compression
            let progress = *self.progress.lock().unwrap();
            if self.compressing {
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            }

            if ui.button("Compress").clicked() && !self.compressing {
                let input = self.input_path.clone();
                let output = format!("{}.bpc", input);
                let status = self.status.clone();
                let progress = self.progress.clone();
                self.compressing = true;
                *self.progress.lock().unwrap() = 0.0;
                thread::spawn(move || {
                    compress_file_with_progress(input, output, progress, status);
                });
            }

            let status_msg = self.status.lock().unwrap().clone();
            if !status_msg.is_empty() {
                ui.label(status_msg);
            }

            ui.separator();
            ui.heading("Decompressor");

            ui.horizontal(|ui| {
                ui.label("Compressed file:");
                ui.text_edit_singleline(&mut self.decompress_input);
                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.decompress_input = path.display().to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Output file:");
                ui.text_edit_singleline(&mut self.decompress_output);
                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        self.decompress_output = path.display().to_string();
                    }
                }
            });

            if ui.button("Decompress").clicked() && !self.decompressing {
                let input = self.decompress_input.clone();
                let output = self.decompress_output.clone();
                let status = self.decompress_status.clone();
                self.decompressing = true;
                thread::spawn(move || {
                    let result = decompress_file(&input, &output);
                    let mut status_lock = status.lock().unwrap();
                    if let Err(e) = result {
                        *status_lock = format!("Decompression failed: {}", e);
                    } else {
                        *status_lock = "Decompression complete!".to_string();
                    }
                });
            }

            let decompress_status_msg = self.decompress_status.lock().unwrap().clone();
            if !decompress_status_msg.is_empty() {
                ui.label(decompress_status_msg);
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "BlockPiper Compressor",
        options,
        Box::new(|_cc| Box::new(BlockPiperApp::default())),
    );
}
