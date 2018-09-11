use nfd::Response;
use std::io::{self, BufRead, Lines, StdinLock, Write};
use std::io::{Result as IoResult, Error as IoError, ErrorKind};

extern crate nfd;

fn get_line(lines: &mut Lines<StdinLock>) -> IoResult<String> {
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
        io::stdout().flush()?;
        let line = get_line(lines)?;
        let r = line.trim();
        if r == "" {
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

fn get_folders() -> Result<Vec<String>, Box<::std::error::Error>> {
    let result = nfd::open_pick_folder(None)?;
    match result {
        Response::Okay(file_path) => Ok(vec![file_path]),
        Response::OkayMultiple(files) => Ok(files),
        Response::Cancel => Ok(Vec::new()),
    }
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
    io::stdout().flush()?;

    let from_str = get_line(&mut lines)?;
    let from = split_from(&from_str);
    if from.len() == 0 {
        println!("No file types specified.");
        return Ok(());
    }

    println!("\nWhich file type would you like to convert to?");
    print!("Convert to: ");
    io::stdout().flush()?;
    let to_str = get_line(&mut lines)?;
    let to = to_str.trim();

    println!("\nPlease pick the folder with the files you wish to convert.");
    let mut folders = get_folders()?;
    while folders.len() == 0 {
        println!("You must pick at least one folder.");
        folders = get_folders()?;
    }
    loop {
        println!("\nYou have picked the following folders:");
        for folder in &folders {
            println!("  {}", folder);
        }
        if !(get_yes_no(&mut lines, "Would you like to pick more folders?", false)?) {
            break;
        }
        let more = get_folders()?;
        folders.extend(more);
    }
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
