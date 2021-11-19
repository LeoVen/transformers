//! Optimize for dev use
//! ```cargo
//! [profile.dev]
//! opt-level = 3
//! ```

use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

const DATA_DIR: &str = "./data";
const OUTPUT_FILE: &str = "./data.txt";
const DATA_SPLIT: &str = "\n\n";
const NEWLINE_CHAR: &str = "<N>";
const DATA_LEN_MAX: usize = 1024;
const DATA_LEN_MIN: usize = 300;

fn copy_data(output: &mut BufWriter<File>, start_path: &str) {
    let path_buf = PathBuf::from(&start_path);
    let mut paths = vec![path_buf];
    let mut total_files = 0;
    while let Some(path) = paths.pop() {
        if let Ok(md) = fs::symlink_metadata(&path) {
            let ft = md.file_type();
            if ft.is_dir() {
                if let Ok(entries) = fs::read_dir(&path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            paths.push(entry.path());
                        }
                    }
                }
            } else if ft.is_file() {
                if let Some(name) = path.file_name() {
                    if let Some(name) = name.to_str() {
                        if name.ends_with(".py") {
                            if let Ok(mut file) = File::open(path) {
                                let mut buffer = String::new();
                                if let Ok(_) = file.read_to_string(&mut buffer) {
                                    total_files += 1;
                                    if total_files % 1000 == 0 {
                                        println!("Total of {} files read", total_files);
                                    }
                                    let buffer = transform_data(buffer);
                                    if buffer.len() > 4 {
                                        let _ = output.write(buffer.as_bytes());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn transform_data(data: String) -> String {
    let data = data.replace("\r", "");
    let mut result = String::new();
    for cap in data.split(DATA_SPLIT) {
        if cap.len() < 4 {
            continue;
        }
        let row_result = &cap.replace("\n", NEWLINE_CHAR);
        if row_result.len() < DATA_LEN_MAX && row_result.len() > DATA_LEN_MIN {
            result += row_result;
            result += "\n";
        }
    }
    result
}

fn main() {
    let out_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(OUTPUT_FILE)
        .expect("Failed to open or create output file");
    copy_data(&mut BufWriter::new(out_file), &DATA_DIR);
}
