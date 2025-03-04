/*
 *  logger.rs - log codebase given info from command line
 *  Copyright (C) 2024  Sebastian Pineda (spineda.wpi.alum@gmail.com)
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *  You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/> */

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader, ErrorKind},
    num::NonZero,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
};

use crate::filetype::{destructure_filetype, stringify_filetype, FileType};

pub struct Logger {
    data: Mutex<VecDeque<PathBuf>>,
    finish_flag: RwLock<bool>,
    line_count: Mutex<usize>,
    keyword_table: Mutex<HashMap<Arc<str>, usize>>,
    filetype_table: Mutex<HashMap<Arc<str>, usize>>,
    data_condition: Condvar,
    data_condition_lock: Mutex<()>,
    root_directory: PathBuf,
    verbose: bool,
}

impl<'a> Logger {
    const CORE_NUM_ERROR: &'a str = "ERROR: Could not properly deduce number of cpu cores!";
    const CPP_FILE_EXTENSIONS: [&'a str; 3] = ["cpp", "cxx", "cc"];
    const KEY_COMMENTS: [&'a str; 4] = ["TODO", "HACK", "BUG", "FIXME"];

    pub fn new(directory: PathBuf, verbose_printing: bool) -> Self {
        let mut comment_table: HashMap<Arc<str>, usize> = HashMap::new();
        for comment in Self::KEY_COMMENTS {
            comment_table.insert(comment.into(), 0);
        }

        Self {
            data: Mutex::new(VecDeque::new()),
            finish_flag: RwLock::new(false),
            line_count: Mutex::new(0),
            keyword_table: Mutex::new(comment_table),
            filetype_table: Mutex::new(HashMap::new()),
            data_condition: Condvar::new(),
            data_condition_lock: Mutex::new(()),
            root_directory: directory,
            verbose: verbose_printing,
        }
    }

    fn print_result(&self) {
        println!("-----------------------------------");
        println!(
            "{: <20} | {: <10}\n",
            "Lines processed",
            self.line_count.lock().unwrap()
        );

        println!("-----------------------------------");
        println!("{: <20} | {: <15}", "Key Comment", "Frequency");
        println!("-----------------------------------");
        for (key, frequency) in self.keyword_table.lock().unwrap().iter() {
            println!("{: <20} | {: <15}", key, frequency);
        }

        println!("\n-----------------------------------");
        println!("{: <20} | {: <15}", "File Type", "Frequency");
        println!("-----------------------------------");
        for (key, frequency) in self.filetype_table.lock().unwrap().iter() {
            println!("{: <20} | {: <15}", key, frequency);
        }
    }

    fn increment_keyword(&self, keyword: &str) {
        if let Some(value) = self.keyword_table.lock().unwrap().get_mut(keyword) {
            *value += 1;
            return;
        }

        self.keyword_table.lock().unwrap().insert(keyword.into(), 1);
    }

    fn increment_filetype_frequency(&self, filetype: &FileType) {
        let name = stringify_filetype!(filetype);

        let mut hashmap_guard = self.filetype_table.lock().unwrap();

        if let Some(value) = hashmap_guard.get_mut(name) {
            *value += 1;
        } else {
            hashmap_guard.insert(name.into(), 1);
        }
    }

    fn classify_file(file: &Path) -> Option<FileType> {
        return match file.extension() {
            Some(extension) => match extension.to_str() {
                Some("c") => Some(FileType::C {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("h") => Some(FileType::CHeader {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some(ext) if Self::CPP_FILE_EXTENSIONS.contains(&ext) => Some(FileType::Cpp {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("hpp") => Some(FileType::CppHeader {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("cs") => Some(FileType::CSharp {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("java") => Some(FileType::Java {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("py") => Some(FileType::Python {
                    inline_comment_format: Some("#"),
                    multiline_comment_start_format: None,
                    multiline_comment_end_format: None,
                }),
                Some("go") => Some(FileType::Go {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("zig") => Some(FileType::Zig {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: None,
                    multiline_comment_end_format: None,
                }),
                Some("rs") => Some(FileType::Rust {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("js") => Some(FileType::Javascript {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("ts") => Some(FileType::Typescript {
                    inline_comment_format: Some("//"),
                    multiline_comment_start_format: Some("/*"),
                    multiline_comment_end_format: Some("*/"),
                }),
                Some("json") => Some(FileType::Json {
                    inline_comment_format: None,
                    multiline_comment_start_format: None,
                    multiline_comment_end_format: None,
                }),
                Some("vb") => Some(FileType::VB {
                    inline_comment_format: Some("'"),
                    multiline_comment_start_format: None,
                    multiline_comment_end_format: None,
                }),
                _ => None,
            },
            None => match file.file_name()?.to_str() {
                Some("Makefile") => Some(FileType::Makefile {
                    inline_comment_format: Some("#"),
                    multiline_comment_start_format: None,
                    multiline_comment_end_format: None,
                }),
                None => None,
                _ => None,
            },
        };
    }

    fn process_line(
        &self,
        line: &str,
        filetype: &FileType,
        file_path: &Path,
        in_multiline_comment: &mut bool,
    ) {
        if line.is_empty() {
            return;
        }

        let (inline_comment_format, multiline_comment_start_format, multiline_comment_end_format) =
            destructure_filetype!(filetype);

        let multiline_start_position: Option<usize> = match multiline_comment_start_format {
            None => None,
            Some(comment_pattern) => line.find(comment_pattern),
        };

        let multiline_end_position: Option<usize> = match multiline_comment_end_format {
            None => None,
            Some(comment_pattern) => line.rfind(comment_pattern),
        };

        let comment_position: Option<usize> = match inline_comment_format {
            None => None,
            Some(comment_pattern) => line.find(comment_pattern),
        };

        // TODO(SEP): There should be 1 of these
        /* HACK(SEP): even in multiline comments
        /* */ FIXME(SEP): This should be caught even with moronic comment style
         */ // BUG(SEP): Even when the comments are weird as hell

        let comment_portion: &str = match (
            multiline_start_position,
            multiline_end_position,
            comment_position,
            *in_multiline_comment,
        ) {
            (None, None, None, false) => return, // Should be impossible
            (Some(_), Some(_), None, true) => line,
            (_, None, _, true) => line,
            (Some(_), Some(_), Some(_), true) => line,

            (Some(multi_left), None, None, false) => {
                *in_multiline_comment = true;
                &line[multi_left..]
            }

            (None, Some(multi_right), None, _) => {
                *in_multiline_comment = false;
                &line[..multi_right]
            }
            (None, Some(multi_right), Some(comment_start), _) => {
                *in_multiline_comment = false;
                match multi_right < comment_start {
                    true => &(line[..multi_right].to_string() + &line[comment_start..]),
                    false => &line[..multi_right],
                }
            }
            (Some(multi_left), None, Some(comment_start), false) => {
                *in_multiline_comment = true;
                match multi_left < comment_start {
                    true => &line[multi_left..],
                    false => &(line[..comment_start].to_string() + &line[multi_left..]),
                }
            }

            (Some(multi_left), Some(multi_right), None, false) => {
                if multi_left < multi_right {
                    &line[multi_left..multi_right]
                } else {
                    line
                }
            }

            (Some(_multi_left), Some(_multi_right), Some(_comment_start), false) => {
                if self.verbose {
                    eprintln!(
                        "WARNING: 
                          This is a complex comment and parsing it is not yet implemented: {:?}",
                        line
                    );
                }
                line
            }

            (None, None, Some(comment_start), false) => &line[comment_start..],
        };

        for keyword in Self::KEY_COMMENTS {
            if comment_portion.contains(keyword) {
                {
                    *self.line_count.lock().unwrap() += 1;
                }

                self.increment_keyword(keyword);

                if self.verbose {
                    println!(
                        "{} Found!\nFile: {:?}\nLine: {}\n",
                        keyword, file_path, line
                    );
                }
            }
        }
    }

    fn parse_file(&self, file_path: &Path) {
        // println!("Parsing File: {:?}", file);

        let file_type = match Self::classify_file(file_path) {
            Some(t) => t,
            None => return,
        };

        self.increment_filetype_frequency(&file_type);

        if let (None, None, None) = destructure_filetype!(file_type) {
            return; // Count file but don't waste time parsing it's lines
        }

        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(_) => return,
        };

        let file_reader: BufReader<File> = BufReader::new(file);
        let mut in_multiline_comment: bool = false;

        for line in file_reader.lines() {
            self.process_line(
                match &line {
                    Ok(good_line) => good_line,
                    Err(_) => "",
                },
                &file_type,
                file_path,
                &mut in_multiline_comment,
            );

            {
                *self.line_count.lock().unwrap() += 1;
            }
        }
    }

    unsafe fn unsafe_waiting_room(handle: LogWrapper) {
        let LogWrapper(inner) = handle;
        (*inner).waiting_room();
    }

    fn waiting_room(&self) {
        loop {
            {
                let guard = self.data_condition_lock.lock().unwrap();
                let _unused = self.data_condition.wait(guard).unwrap();
            }

            if let Some(found_file) = self.data.lock().unwrap().pop_front() {
                self.parse_file(&found_file);
            } else if *self.finish_flag.read().unwrap() {
                self.data_condition.notify_all();
                return;
            }
        }
    }

    fn populate_queue(&self, root: &Path) -> Result<(), std::io::Error> {
        if root.is_dir() {
            for entry in root.read_dir()? {
                let entry = entry?;
                if entry.path().is_dir() {
                    self.populate_queue(&entry.path())?;
                } else {
                    self.data.lock().unwrap().push_back(entry.path());
                    self.data_condition.notify_one();
                }
            }
        } else {
            self.data.lock().unwrap().push_back(root.to_path_buf());
            self.data_condition.notify_one();
        }

        Ok(())
    }

    pub fn log(&mut self) -> Result<(), std::io::Error> {
        let worker_count = NonZero::new(num_cpus::get());
        let worker_count = match worker_count {
            Some(number) => number,
            None => {
                eprintln!("{}", Self::CORE_NUM_ERROR);
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    Self::CORE_NUM_ERROR,
                ));
            }
        };

        println!(
            "Number of CPUs supported for Trace's file I/O: {}\n",
            worker_count
        );

        let mut workers: Vec<thread::JoinHandle<()>> = Vec::with_capacity(worker_count.get());

        // keep one instance of object with internal locks
        unsafe {
            for _ in 0..worker_count.get() {
                let handle: LogWrapper = LogWrapper(self as *mut Logger);
                workers.push(thread::spawn(|| {
                    Self::unsafe_waiting_room(handle);
                }));
            }
        }

        self.populate_queue(&self.root_directory)?;

        loop {
            let size = match self.data.try_lock() {
                Ok(s) => s.len(),
                _ => continue,
            };

            if size == 0 {
                break;
            } else {
                self.data_condition.notify_all();
            }
        }

        {
            *self.finish_flag.write().unwrap() = true;
            self.data_condition.notify_all();
        }

        for worker in workers {
            let _ = worker.join();
        }

        self.print_result();

        Ok(())
    }
}

struct LogWrapper(*mut Logger);

unsafe impl Send for LogWrapper {}
