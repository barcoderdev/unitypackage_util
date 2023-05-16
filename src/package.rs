//----------------------------------------

use std::borrow::BorrowMut;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use tar::Archive;
use walkdir::WalkDir;

//----------------------------------------

pub struct Package {
    r#type: PackageType,
}

//----------------------------------------

enum PackageType {
    Folder(PathBuf),
    Tar(PathBuf),
    TarGz(PathBuf),
}

//----------------------------------------

enum PackageFileSystemHandle {
    Folder(String, Option<WalkDir>),
    Tar(Archive<File>),
    TarGz(Archive<GzDecoder<File>>),
}

//----------------------------------------

pub enum PackageEntries<'a> {
    Folder(
        String,
        Option<Box<dyn Iterator<Item = Result<walkdir::DirEntry, walkdir::Error>>>>,
    ),
    Tar(Result<tar::Entries<'a, File>, std::io::Error>),
    TarGz(Result<tar::Entries<'a, GzDecoder<File>>, std::io::Error>),
}

//----------------------------------------

pub enum PackageEntry<'a> {
    Folder(String, walkdir::DirEntry),
    Tar(tar::Entry<'a, File>),
    TarGz(tar::Entry<'a, GzDecoder<File>>),
}

//----------------------------------------

pub struct PackageHandle {
    handle: PackageFileSystemHandle,
}

//----------------------------------------

impl core::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.r#type {
            PackageType::Folder(path) => write!(f, "Folder: {}", path.display()),
            PackageType::Tar(path) => write!(f, "Tar: {}", path.display()),
            PackageType::TarGz(path) => write!(f, "TarGz: {}", path.display()),
        }
    }
}

//----------------------------------------

impl Package {
    pub fn new(path: &str) -> Result<Package, String> {
        let path = PathBuf::from(path);

        if path.is_dir() {
            Ok(Package {
                // canonicalize to fix any path issues
                r#type: PackageType::Folder(path.canonicalize().unwrap()),
            })
        } else if path.is_file() {
            let kind = infer::get_from_path(&path).unwrap().unwrap();
            match kind.mime_type() {
                "application/x-tar" => Ok(Package {
                    r#type: PackageType::Tar(path),
                }),
                "application/gzip" => Ok(Package {
                    r#type: PackageType::TarGz(path),
                }),
                _ => Err("File is not a tar or tar.gz".into()),
            }
        } else {
            Err("Path is not a file or directory".into())
        }
    }

    pub fn open(self) -> Result<PackageHandle, String> {
        match self.r#type {
            PackageType::Folder(path) => {
                let dir = WalkDir::new(&path);
                Ok(PackageHandle {
                    handle: PackageFileSystemHandle::Folder(
                        path.to_str().unwrap().to_string(),
                        Some(dir),
                    ),
                })
            }
            PackageType::Tar(path) => {
                let file = File::open(&path).unwrap();
                let archive = Archive::new(file);
                Ok(PackageHandle {
                    handle: PackageFileSystemHandle::Tar(archive),
                })
            }
            PackageType::TarGz(path) => {
                let file = File::open(&path).unwrap();
                let file = GzDecoder::new(file);

                let archive = Archive::new(file);
                Ok(PackageHandle {
                    handle: PackageFileSystemHandle::TarGz(archive),
                })
            }
        }
    }
}

//----------------------------------------

impl PackageHandle {
    pub fn entries(&mut self) -> PackageEntries {
        match self.handle.borrow_mut() {
            PackageFileSystemHandle::Folder(path, entries) => PackageEntries::Folder(
                path.clone(),
                Some(Box::new(entries.take().unwrap().into_iter())),
            ),
            PackageFileSystemHandle::Tar(archive) => PackageEntries::Tar(archive.entries()),
            PackageFileSystemHandle::TarGz(archive) => PackageEntries::TarGz(archive.entries()),
        }
    }
}

//----------------------------------------

impl<'a> Iterator for PackageEntries<'a> {
    type Item = std::io::Result<PackageEntry<'a>>;

    fn next(&mut self) -> Option<std::io::Result<PackageEntry<'a>>> {
        match self.borrow_mut() {
            PackageEntries::Folder(path, entries) => {
                let next = entries.as_mut().unwrap().next();
                match next {
                    Some(Ok(entry)) => Some(Ok(PackageEntry::Folder(path.clone(), entry))),
                    Some(Err(err)) => Some(Err(err.into())),
                    None => None,
                }
            }
            PackageEntries::Tar(entries) => match entries.as_mut().unwrap().next() {
                Some(Ok(entry)) => Some(Ok(PackageEntry::Tar(entry))),
                Some(Err(err)) => Some(Err(err)),
                None => None,
            },
            PackageEntries::TarGz(entries) => match entries.as_mut().unwrap().next() {
                Some(Ok(entry)) => Some(Ok(PackageEntry::TarGz(entry))),
                Some(Err(err)) => Some(Err(err)),
                None => None,
            },
        }
    }
}

impl<'a> PackageEntry<'a> {
    pub fn size(&self) -> Option<usize> {
        match self {
            PackageEntry::Folder(_path, entry) => Some(entry.metadata().unwrap().len() as usize),
            PackageEntry::Tar(entry) => Some(entry.header().size().unwrap() as usize),
            PackageEntry::TarGz(entry) => Some(entry.header().size().unwrap() as usize),
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        match self {
            PackageEntry::Folder(root_path, entry) => {
                let path = entry.path().to_str().unwrap();
                let replace = format!("{}/", root_path);
                let path = path.replace(&replace, "");
                // println!("path: {}", path);
                Some(PathBuf::from(path))
            }
            PackageEntry::Tar(entry) => Some(PathBuf::from(entry.path().unwrap())),
            PackageEntry::TarGz(entry) => Some(PathBuf::from(entry.path().unwrap())),
        }
    }

    pub fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            PackageEntry::Folder(_path, entry) => {
                let path = entry.path().to_str().unwrap();
                let result = fs::read_to_string(path);
                *buf = result.unwrap();
                Ok(buf.len())
            }
            PackageEntry::Tar(entry) => entry.read_to_string(buf),
            PackageEntry::TarGz(entry) => entry.read_to_string(buf),
        }
    }

    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        match self {
            PackageEntry::Folder(_path, entry) => {
                let path = entry.path().to_str().unwrap();
                let result = fs::read(path);
                *buf = result.unwrap();
                Ok(buf.len())
            }
            PackageEntry::Tar(entry) => entry.read_to_end(buf),
            PackageEntry::TarGz(entry) => entry.read_to_end(buf),
        }
    }

    pub fn guid(&self) -> String {
        match self {
            PackageEntry::Folder(root_path, entry) => {
                let replace = format!("{}/", root_path);
                let path = entry.path().to_str().unwrap();
                let path = path.replace(&replace, "");
                let path = path.split("/").collect::<Vec<&str>>();
                let path = path.first().unwrap();
                path.to_string()
            }
            PackageEntry::Tar(entry) => {
                let path = entry.path().unwrap();
                let path = path.to_str().unwrap();
                let path = path.split("/").collect::<Vec<&str>>();
                let path = path.first().unwrap();
                path.to_string()
            }
            PackageEntry::TarGz(entry) => {
                let path = entry.path().unwrap();
                let path = path.to_str().unwrap();
                let path = path.split("/").collect::<Vec<&str>>();
                let path = path.first().unwrap();
                path.to_string()
            }
        }
    }
}

//----------------------------------------
