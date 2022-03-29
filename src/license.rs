// license.rs copyright 2022 
// balh blah blah

// mog

use ignore::DirEntry;
use mktemp::Temp;
use std::{fs, fs::File, io, io::Write, path::Path};

use crate::config::FileTypeConfig;

const LICENSE_PATH: &str = ".licensesnip";

fn prepend_file(data: &[u8], file_path: &Path) -> io::Result<()> {
    // Create a temporary file
    let tmp = Temp::new_file()?;
    let tmp_path = tmp.to_path_buf();
    // Stop the temp file being automatically deleted when the variable
    // is dropped, by releasing it.
    tmp.release();
    // Open temp file for writing
    let mut tmp = File::create(&tmp_path)?;
    // Open source file for reading
    let mut src = File::open(&file_path)?;
    // Write the data to prepend
    tmp.write_all(&data)?;
    // Copy the rest of the source file
    io::copy(&mut src, &mut tmp)?;
    fs::remove_file(&file_path)?;
    fs::copy(&tmp_path, &file_path)?;
    fs::remove_file(&tmp_path)?;
    Ok(())
}

pub enum ReadLicenseErr {
    FileReadErr,
}

pub struct License {
    pub raw_text: String,
}

impl License {
    pub fn get_formatted<Y: std::fmt::Display + Copy>(
        raw_text: &String,
        file_name: &str,
        year: Y,
    ) -> String {
        raw_text
            .replace("%FILENAME%", file_name)
            .replace("%YEAR%", &year.to_string())
    }

    pub fn get_formatted_lines<Y: std::fmt::Display + Copy>(
        lines: &Vec<&str>,
        file_name: &str,
        year: Y,
    ) -> Vec<String> {
        let mut vec = Vec::<String>::new();
        for str in lines {
            vec.push(License::get_formatted(&str.to_string(), file_name, year))
        }
        vec
    }

    pub fn get_lines(&self) -> Vec<&str> {
        self.raw_text.split("\n").collect::<Vec<&str>>()
    }

    pub fn get_header_text(lines: &Vec<String>, cfg: &FileTypeConfig) -> String {
        let mut text = String::new();
        text.push_str(&cfg.before_block);

        for line in lines {
            if line.trim().is_empty() {
                text.push('\n')
            } else {
                let mut line_text = cfg.before_line.clone();
                line_text.push_str(line);

                text.push_str(&line_text);
                text.push('\n');
            }
        }

        text.push_str(&cfg.after_block);

        text
    }

    pub fn add_to_file(
        ent: &DirEntry,
        header_text: &String,
    ) -> Result<AddToFileResult, AddToFileErr> {
        let path = ent.path();
        let file_text = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Err(AddToFileErr::ReadFileErr),
        };

        let mut f_chars = file_text.bytes();

        let mut should_add = false;

        for h_char in header_text.bytes() {
            let f_char = match f_chars.next() {
                Some(c) => c,
                None => {
                    // Reached the end of file
                    should_add = true;
                    break;
                }
            };
            if f_char != h_char {
                should_add = true;
                break;
            }
        }

        if should_add {
            let mut text_to_add = header_text.clone();
            text_to_add.push('\n');

            // add to top of file
            match prepend_file(text_to_add.as_bytes(), &path) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    return Err(AddToFileErr::WriteFileErr);
                }
            };

            return Ok(AddToFileResult::Added);
        }

        Ok(AddToFileResult::NoChange)
    }
}

pub enum AddToFileResult {
    Added,
    NoChange,
}

#[derive(Debug)]
pub enum AddToFileErr {
    ReadFileErr,
    WriteFileErr,
}

pub fn read_license() -> Result<License, ReadLicenseErr> {
    let read_result = fs::read_to_string(LICENSE_PATH);

    match read_result {
        Ok(str) => return Ok(License { raw_text: str }),
        Err(_) => return Err(ReadLicenseErr::FileReadErr),
    }
}
