//! Optimize for dev use
//! ```cargo
//! [profile.dev]
//! opt-level = 3
//! ```

use std::fs;
use std::fs::File;
use std::io;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

const DATA_DIR: &str = "./data";
const OUTPUT_FILE: &str = "./data.txt";

fn copy_data(output: &mut BufWriter<File>, start_path: &str) -> io::Result<()> {
    let path_buf = PathBuf::from(&start_path);
    let mut paths = vec![path_buf];
    while let Some(path) = paths.pop() {
        let ft = fs::symlink_metadata(&path)?.file_type();
        if ft.is_dir() {
            let entries = fs::read_dir(&path)?;
            for entry in entries {
                paths.push(entry?.path());
            }
        } else if ft.is_file() {
            if let Some(name) = path.file_name() {
                if let Some(name) = name.to_str() {
                    if name.ends_with(".py") {
                        if let Ok(mut file) = File::open(path) {
                            let mut buffer = vec![];
                            file.read_to_end(&mut buffer)?;
                            output.write(&buffer)?;
                            output.write(b"\n\n")?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let out_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(OUTPUT_FILE)
        .expect("Failed to open or create output file");
    match copy_data(&mut BufWriter::new(out_file), &DATA_DIR) {
        Ok(_) => {},
        Err(e) => eprint!("{}", e),
    }
}
