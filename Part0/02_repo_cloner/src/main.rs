use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

const OUT_DATA: &str = "../data/";
const NUM_THREADS: usize = 2;

fn clone_repos(
    tag: usize,
    repos: Vec<(usize, String)>,
    log: Arc<Mutex<File>>,
) -> io::Result<(u32, u32)> {
    let mut error = 0u32;
    let mut success = 0u32;
    for (id, repo) in repos.into_iter() {
        let repo_folder = format!("{}{:0>6}", OUT_DATA, id);
        println!("[{}] Cloning \"{}\" into \"{}\"", tag, repo, repo_folder);
        let git_clone = Command::new("git")
            .args(&["clone", "--depth", "1", &repo, &repo_folder])
            .output()?;
        let rm_git = Command::new("rm")
            .args(&["-rf", &format!("{}/.git", repo_folder)])
            .output()?;
        if git_clone.status.success() && rm_git.status.success() {
            clean_data(tag, repo_folder)?;
            // End of processing
            let mut log = log.lock().expect("Failed to lock Mutex for Log file");
            if log.write(format!("{}\n", id).as_bytes()).is_err() {
                eprintln!("[{}] Failed to write log of {}", tag, id);
                error += 1;
            } else {
                println!("[{}] Done {}", tag, repo);
                success += 1;
            }
        } else {
            if !git_clone.stderr.is_empty() {
                eprintln!(
                    "[{}] :\n{}",
                    tag,
                    String::from_utf8(git_clone.stderr)
                        .unwrap_or(format!("[{}] Failed to parse Stderr of git_clone", tag))
                );
            }
            if !rm_git.stderr.is_empty() {
                eprintln!(
                    "[{}] :\n{}",
                    tag,
                    String::from_utf8(rm_git.stderr)
                        .unwrap_or(format!("[{}] Failed to parse Stderr of rm_git", tag))
                );
            }
            error += 1;
        }
    }

    Ok((success, error))
}

fn clean_data(tag: usize, path: String) -> io::Result<()> {
    println!("[{}] Cleaning {}", tag, &path);

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
                    if !name.ends_with(".py") {
                        if let Err(e) = fs::remove_file(&path) {
                            println!("[{}] Failed to delete {:?}", tag, path);
                            return Err(e);
                        }
                    }
                }
            }
        }
    }

    println!("[{}] Cleaning done on {}", tag, path);

    Ok(())
}

fn main() {
    if let Ok(mut file) = fs::File::open("../repos.txt") {
        let mut data = String::with_capacity(10000000);
        if file.read_to_string(&mut data).is_ok() {
            // Read log
            let mut log_file = OpenOptions::new()
                .read(true)
                .append(true)
                .write(false)
                .open("../log.txt")
                .expect("Failed to open Log file");
            let mut keys = String::new();
            log_file
                .read_to_string(&mut keys)
                .expect("Failed to read from Log file");
            let mut ids: HashSet<usize> = HashSet::new();
            if let Some(_) = keys.pop() {
                ids = keys
                .split('\n')
                .map(|split| str::parse(split).expect("Failed to parse number from log file"))
                .collect();
            }
            // Create and filter the ids already at the log file
            let rows: Vec<(usize, String)> = data
                .split('\n')
                .filter(|split| !split.is_empty())
                .map(|row| {
                    let row = &row[..row.len() - 1]; // Remove \n
                    let mut iter = row.split(',');
                    (
                        str::parse::<usize>(iter.next().expect("Missing Row Id"))
                            .expect("Failed to parse Row Id"),
                        iter.next().expect("Missing Row URL").to_string(),
                    )
                })
                .filter(|(id, _)| !ids.contains(id))
                .collect::<Vec<_>>();
            let chunks: Vec<Vec<(usize, String)>> = rows
                .chunks(rows.len() / NUM_THREADS)
                .map(|chunk| chunk.to_vec())
                .collect();
            let chunks = chunks;
            let mut handles = vec![];
            let log = Arc::new(Mutex::new(log_file));

            println!("Skipping {} repos already cloned", ids.len());

            for (i, chunk) in chunks.into_iter().enumerate() {
                let log = log.clone();
                handles.push(thread::spawn(move || clone_repos(i, chunk, log)))
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
                        Err(e) => eprintln!("{}", e),
                    }
                } else {
                    eprintln!("Thread exited with error");
                }
            }

            println!("Success: {}\nError: {}", success, error);
        }
    } else {
        eprintln!("Error opening repos file");
    }
}
