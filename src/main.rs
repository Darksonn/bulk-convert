use std::io::{self, BufRead, Lines, StdinLock, Write};
use std::io::{Result as IoResult, Error as IoError, ErrorKind};
use std::process::{Command, Stdio, Child};
use std::path::PathBuf;
use std::fs;
use std::collections::VecDeque;

#[cfg(windows)]
fn ensure_imagemagick() -> IoResult<()> {
    use std::fs::File;
    use std::env;
    let mut convertexe = env::current_dir().unwrap();
    convertexe.push("ffmpeg.exe");
    if !convertexe.exists() {
        println!("ffmpeg.exe not found, creating...");
        let convert = include_bytes!("../imagemagick/ffmpeg.exe");
        let mut file = File::create(convertexe)?;
        file.write(convert)?;
    }
    Ok(())
}

fn get_line(lines: &mut Lines<StdinLock>) -> IoResult<String> {
    io::stdout().flush()?;
    match lines.next() {
        Some(line) => Ok(line?),
        None => return Err(IoError::new(ErrorKind::InvalidData, "Unable to get line.")),
    }
}
fn get_yes_no(
    lines: &mut Lines<StdinLock>,
    question: &str,
    default: bool
) -> IoResult<bool> {
    loop {
        if default {
            print!("{} [Yn] ", question);
        } else {
            print!("{} [yN] ", question);
        }
        let line = get_line(lines)?;
        let r = line.trim();
        if r == "" {
            return Ok(default);
        }
        if r == "y" || r == "Y" {
            return Ok(true);
        }
        if r == "n" || r == "N" {
            return Ok(false);
        }
    }
}
fn split_from<'a>(from_str: &'a str) -> Vec<&'a str> {
    from_str.split(|c: char| !c.is_alphabetic()).filter(|s| s.len() > 0).collect()
}

fn find_files(
    file: PathBuf,
    from: &[&str],
    save: &mut Vec<PathBuf>
) -> IoResult<()> {
    let metadata = fs::metadata(&file)?;
    if metadata.is_dir() {
        for file in fs::read_dir(file)? {
            find_files(file?.path(), from, save)?;
        }
    } else {
        if let Some(ext) = file.extension() {
            if let Some(ext) = ext.to_str() {
                if from.contains(&ext) {
                    save.push(file.to_path_buf());
                }
            }
        }
    }
    Ok(())
}

fn convert_files(
    files: &[PathBuf],
    save_location: Option<PathBuf>,
    to: &str,
) -> IoResult<()> {
    let mut children: VecDeque<(&PathBuf, Child)> = VecDeque::with_capacity(16);

    for file in files {
        let mut new_name = file.file_stem().unwrap().to_os_string();
        new_name.push(".");
        new_name.push(to);
        let res_file = match &save_location {
            Some(dir) => {
                let mut res = dir.clone();
                res.push(new_name);
                res
            },
            None => {
                let mut parent = file.parent().unwrap().to_path_buf();
                parent.push(new_name);
                parent
            },
        };
        let path = if cfg!(windows) {
            ".\\ffmpeg.exe"
        } else {
            "ffmpeg"
        };
        let child = Command::new(path)
            .arg("-hide_banner")
            .arg("-i")
            .arg(file.as_os_str())
            .arg(res_file.as_os_str())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if children.len() == 16 {
            let (wait_for_file, mut wait_for_child) = children.pop_back().unwrap();
            let exit = wait_for_child.wait()?;
            if exit.success() {
                println!("{} converted.", wait_for_file.file_name().unwrap().to_str().unwrap());
            } else {
                println!("Conversion of {} failed.", wait_for_file.to_str().unwrap());
            }
        }
        children.push_front((file, child));
    }
    for (wait_for_file, mut wait_for_child) in children {
        let exit = wait_for_child.wait()?;
        if exit.success() {
            println!("{} converted.", wait_for_file.file_name().unwrap().to_str().unwrap());
        } else {
            println!("Conversion of {} failed.", wait_for_file.to_str().unwrap());
        }
    }

    Ok(())
}

fn real_main() -> Result<(), Box<::std::error::Error>> {

    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut lines = stdin.lines();

    println!("Bulk converter v0.1.0");
    println!("Which file types would you like to convert from?");
    println!("Examples:");
    println!("  png");
    println!("  jpg,png");
    println!("");
    print!("Convert from: ");

    let from_str = get_line(&mut lines)?;
    let from = split_from(&from_str);
    if from.len() == 0 {
        println!("No file types specified.");
        return Ok(());
    }

    println!("\nWhich file type would you like to convert to?");
    print!("Convert to: ");
    let to_str = get_line(&mut lines)?;
    let to = to_str.trim();
    if to.len() == 0 {
        println!("No to selected.");
        return Ok(());
    }

    println!("\nPlease pick the folder with the files you wish to convert.");
    print!("Input folder: ");
    let mut folders = vec![get_line(&mut lines)?];
    loop {
        println!("\nYou have picked the following folders:");
        for folder in &folders {
            println!("  {}", folder);
        }
        if !(get_yes_no(&mut lines, "Would you like to pick another folder?", false)?) {
            break;
        }
        print!("Input folder: ");
        folders.push(get_line(&mut lines)?);
    }

    let mut files = Vec::with_capacity(16);
    for folder in folders {
        find_files(folder.into(), &from, &mut files)?;
    }
    println!("\nThe following files will be converted:");
    for file in &files {
        println!("{}", file.file_name().unwrap().to_str().unwrap());
    }

    println!();
    let separate_folder = get_yes_no(
        &mut lines,
        "Should convertion results be saved in a separate folder?",
        false
    )?;
    let save_location = if separate_folder {
        let dir = get_line(&mut lines)?;
        Some(dir.into())
    } else {
        None
    };

    #[cfg(windows)]
    ensure_imagemagick()?;

    convert_files(&files, save_location, to)?;

    Ok(())
}

fn main() {
    match real_main() {
        Ok(()) => {},
        Err(err) => {
            println!("\nError!\n{}", err);
        },
    }
    println!("Press enter to exit.");
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let mut lines = stdin.lines();
    let _ = lines.next();
}
