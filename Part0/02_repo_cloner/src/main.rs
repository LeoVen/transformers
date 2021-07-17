use std::fs;
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::thread;
use std::process::Command;

const OUT_DATA: &'static str = "../data/";

fn clone_repos(tag: usize, repos: Vec<String>) -> io::Result<(u32, u32)> {
    let mut error = 0u32;
    let mut success = 0u32;
    for (i, repo) in repos.into_iter().enumerate() {
        if i > 1 {
            break;
        }

        let repo_folder = format!("{}{}{:0>6}", OUT_DATA, tag, i);
        println!("Cloning \"{}\" into \"{}\"", repo, repo_folder);
        let out = Command::new("git")
            .args(&["clone", &repo, &repo_folder])
            .output()?;
        if out.status.success() {
            clean_data(repo_folder)?;
            success += 1;
        } else {
            error += 1;
        }
    }

    Ok((success, error))
}

fn clean_data(path: String) -> io::Result<()> {
    println!("Cleaning {}", &path);

    let path_buf = PathBuf::from(&path);
    let mut paths = vec![path_buf];

    while let Some(path) = paths.pop() {
        let meta = fs::symlink_metadata(&path)?;
        let ft = meta.file_type();

        if ft.is_dir() {
            let entries = fs::read_dir(&path)?;
            for entry in entries {
                paths.push(entry?.path());
            }
        } else if ft.is_file() {
            if let Some(name) = path.file_name() {
                if let Some(name) = name.to_str() {
                    if !name.ends_with(".java") {
                        let _ = fs::remove_file(path)?;
                    }
                }
            }
        }
    }

    println!("Cleaning done on {}", path);

    Ok(())
}

fn main() {
    if let Ok(mut file) = fs::File::open("../repos.txt") {
        let mut data = String::with_capacity(10000000);
        if let Ok(_) = file.read_to_string(&mut data) {
            let rows = data.split('\n').map(|row| {
                let mut result = row.to_string();
                let _ = result.pop(); // Remove \n
                result
            }).collect::<Vec<_>>();
            let chunks: Vec<Vec<String>> = rows.chunks(rows.len() / 8)
                                               .map(|chunk| chunk.to_vec())
                                               .collect();
            let chunks = chunks;
            let mut handles = vec![];

            for (tag, chunk) in chunks.into_iter().enumerate() {
                handles.push(thread::spawn(move || {
                    clone_repos(tag, chunk)
                }))
            }

            let mut error = 0;
            let mut success = 0;
            for handle in handles {
                if let Ok(result) = handle.join() {
                    match result {
                        Ok(result) => {
                            success += result.0;
                            error += result.1;
                        }
                        Err(e) => eprintln!("{}", e)
                    }
                } else {
                    eprintln!("Thread exited with error");
                }
            }

            println!("Success: {}\nError: {}", success, error);
        }
    } else {
        eprintln!("Error opening file");
    }
}
