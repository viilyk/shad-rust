#![forbid(unsafe_code)]

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

use rayon::prelude::*;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    pub path: PathBuf,
    pub line: String,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct Error {
    pub path: PathBuf,
    pub error: io::Error,
}

pub enum Event {
    Match(Match),
    Error(Error),
}

pub fn run<P: AsRef<Path>>(path: P, pattern: &str) -> Vec<Event> {
    if path.as_ref().is_file() {
        BufReader::new(File::open(path.as_ref()).unwrap())
            .lines()
            .enumerate()
            .filter(|(_, x)| x.is_err() || x.as_ref().unwrap().contains(pattern))
            .map(|(i, line)| match line {
                Ok(l) => Event::Match(Match {
                    path: PathBuf::from(path.as_ref()),
                    line: l,
                    line_number: i + 1,
                }),
                Err(error) => Event::Error(Error {
                    path: PathBuf::from(path.as_ref()),
                    error,
                }),
            })
            .collect()
    } else {
        match path.as_ref().read_dir() {
            Ok(p) => p
                .map(|e| e.unwrap().path())
                .collect::<Vec<PathBuf>>()
                .par_iter()
                .map(|p| run(p, pattern))
                .flatten()
                .collect(),
            Err(e) => vec![Event::Error(Error {
                path: path.as_ref().to_path_buf(),
                error: e,
            })],
        }
    }
}
