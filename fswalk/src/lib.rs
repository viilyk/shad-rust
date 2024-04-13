#![forbid(unsafe_code)]

use std::{
    fs,
    io::{self, Read},
    path::Path,
};

////////////////////////////////////////////////////////////////////////////////

type Callback<'a> = dyn FnMut(&mut Handle) + 'a;

#[derive(Default)]
pub struct Walker<'a> {
    callbacks: Vec<Box<Callback<'a>>>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut Handle) + 'a,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn walk<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        Self::rec_walk(path.as_ref(), self.callbacks.as_mut_slice())
    }

    fn rec_walk(p: &Path, callbacks: &mut [Box<Callback>]) -> io::Result<()> {
        if callbacks.is_empty() {
            return Ok(());
        }
        for entry in fs::read_dir(p)? {
            let path = entry?.path();
            let mut handle;
            if path.is_dir() {
                handle = Handle::Dir(DirHandle {
                    path: &path,
                    interesting: false,
                })
            } else if path.is_file() {
                handle = Handle::File(FileHandle {
                    path: &path,
                    interesting: false,
                })
            } else {
                continue;
            }
            let mut x = 0;
            for i in 0..callbacks.len() {
                (callbacks[i])(&mut handle);
                if handle.check() {
                    if x < i {
                        callbacks.swap(x, i);
                    }
                    x += 1;
                }
                handle.reset();
            }
            match handle {
                Handle::Dir(dir) => {
                    Self::rec_walk(dir.path(), &mut callbacks[..x])?;
                }
                Handle::File(file) => {
                    let mut buffer: Vec<u8> = Vec::new();
                    fs::File::open(file.path())?.read_to_end(&mut buffer)?;
                    let mut content = Handle::Content {
                        file_path: file.path(),
                        content: &buffer,
                    };
                    for callback in callbacks[..x].iter_mut() {
                        (callback)(&mut content);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum Handle<'a> {
    Dir(DirHandle<'a>),
    File(FileHandle<'a>),
    Content {
        file_path: &'a Path,
        content: &'a [u8],
    },
}

impl Handle<'_> {
    fn check(&mut self) -> bool {
        match self {
            Handle::Dir(dir) => dir.interesting,
            Handle::File(file) => file.interesting,
            _ => false,
        }
    }
    fn reset(&mut self) {
        match self {
            Handle::Dir(dir) => dir.interesting = false,
            Handle::File(file) => file.interesting = false,
            _ => {}
        }
    }
}

pub struct DirHandle<'a> {
    path: &'a Path,
    interesting: bool,
}

impl<'a> DirHandle<'a> {
    pub fn descend(&mut self) {
        self.interesting = true;
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}

pub struct FileHandle<'a> {
    path: &'a Path,
    interesting: bool,
}

impl<'a> FileHandle<'a> {
    pub fn read(&mut self) {
        self.interesting = true;
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
