# BlockPiper

**BlockPiper** is a high-performance, lossless file compression tool written in Rust. It uses advanced grammar-based modeling (Sequitur), context tree weighting (CTW), and real arithmetic coding for state-of-the-art compression. BlockPiper features a modern GUI for easy file selection, compression, and decompression.

## Features
- Lossless, block-based file compression
- Grammar-based modeling (Sequitur algorithm)
- Adaptive context modeling (CTW)
- Real arithmetic coding (via [constriction](https://github.com/fkiesel/constriction))
- Parallel block processing (via Rayon)
- Modern GUI (egui/eframe)
- Decompression support

## Build Instructions

1. **Install Rust** (if not already):
   https://rustup.rs/

2. **Clone the repository** (or copy the project files):
   ```sh
   git clone <your-repo-url>
   cd blockpiper
   ```

3. **Build and run the GUI:**
   ```sh
   cargo run --release
   ```

   The GUI window will open for file selection and compression/decompression.

## Usage (GUI)
1. **Compress:**
   - Select an input file and an output file.
   - Click **Compress**.
   - Wait for the status message "Compression complete!"

2. **Decompress:**
   - Select a compressed file and an output file.
   - Click **Decompress**.
   - Wait for the status message "Decompression complete!"

## Algorithm Overview
- **Block Architecture:** Files are split into blocks for parallel processing.
- **Grammar-Based Modeling:** Each block is modeled using the Sequitur algorithm, producing a compact grammar.
- **CTW (Context Tree Weighting):** Adaptive context modeling predicts symbol probabilities for each block.
- **Arithmetic Coding:** The symbol stream is entropy-coded using real arithmetic coding for maximum compression.
- **Decompression:** The process is reversed, reconstructing the original file exactly.

## Dependencies
- [Rayon](https://crates.io/crates/rayon) (parallelism)
- [egui](https://crates.io/crates/egui), [eframe](https://crates.io/crates/eframe) (GUI)
- [rfd](https://crates.io/crates/rfd) (file dialogs)
- [constriction](https://crates.io/crates/constriction) (arithmetic coding)

## Credits
- Sequitur algorithm: [Craig Nevill-Manning, Ian H. Witten](https://www.sequitur.info/)
- CTW: [Willems, Shtarkov, Tjalkens, 1995]
- Arithmetic coding: [constriction crate](https://github.com/fkiesel/constriction)
- GUI: [egui/eframe](https://github.com/emilk/egui)

---

**BlockPiper** is open source and extensible. Contributions and feedback are welcome! 