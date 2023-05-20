//----------------------------------------

use std::fs::File;

use flate2::read::GzDecoder;
use tar::Archive;

use std::{
    fs,
    io::{self},
    ops::Add,
};

use regex::Regex;

//----------------------------------------

const UNITY_MACRO: &str = "--- !u!";

//----------------------------------------

pub fn asset_yaml_cleanup(yaml: &str) -> String {
    let mut class_id: i32 = 0;
    let mut file_id: i64 = 0;
    let mut extra: Option<String> = None;

    let re = Regex::new(r"fileID: (\d+)").unwrap();

    yaml.lines()
        .into_iter()
        .map(|line| {
            if line.starts_with(UNITY_MACRO) {
                // Split "--- !u!CLASS_ID &FILE_ID EXTRA" into parts
                let mut chunks = line[UNITY_MACRO.len()..]
                    .trim()
                    .split_whitespace()
                    .into_iter();
                class_id = chunks.next().unwrap_or("").parse().unwrap();
                file_id = chunks
                    .next()
                    .unwrap_or("")
                    .replace("&", "")
                    .parse()
                    .unwrap_or(0);
                extra = chunks.next().map(|s| s.to_string());
                // Keep the "---" document separator
                "---\n".to_string()
            } else if line.starts_with("%") || line.len() == 0 {
                // Ignore %YAML, %TAG, etc
                "".to_string()
            } else if !line.starts_with(" ") && !line.starts_with("'") && !line.is_empty() {
                // Replace "Prefab:"(or other) with "type: Prefab" and "content:"
                // And add the class_id, file_id, and extra
                // TODO: Account for multi-line strings in a better way
                format!(
                    "_class_id: {}\n_file_id: \"{}\"\n_extra: {}\ntype: {}\ncontent:\n",
                    class_id,
                    file_id,
                    match extra {
                        Some(ref s) => format!("{}\n", s),
                        None => "".to_string(),
                    },
                    line.replace(":", ""),
                )
            } else {
                // Keep the usual field lines
                let k = re.replace(line, "fileID: \"$1\"");
                format!("{k}\n")
            }
        })
        .collect::<String>()
}

//----------------------------------------

pub fn asset_meta_yaml_cleanup(yaml: &str) -> String {
    let mut file_format_version = 2usize;
    let mut guid = "".to_string();
    let mut folder_asset: Option<bool> = None;
    let mut time_created = "".to_string();
    let mut license_type = "".to_string();

    let re = Regex::new(r"fileID: (\d+)").unwrap();

    yaml.lines()
        .into_iter()
        .map(|line| {
            if line.starts_with("fileFormatVersion:") {
                let mut split = line.split(":");
                split.next().unwrap();
                file_format_version = split.next().unwrap().trim().parse().unwrap();
                "".to_owned()
            } else if line.starts_with("guid:") {
                let mut split = line.split(":");
                split.next().unwrap();
                guid = split.next().unwrap().trim().to_string();
                "".to_owned()
            } else if line.starts_with("timeCreated:") {
                let mut split = line.split(":");
                split.next().unwrap();
                time_created = split.next().unwrap().trim().to_string();
                "".to_owned()
            } else if line.starts_with("licenseType:") {
                let mut split = line.split(":");
                split.next().unwrap();
                license_type = split.next().unwrap().trim().to_string();
                "".to_owned()
            } else if line.starts_with("folderAsset:") {
                folder_asset = if line.ends_with("yes") {
                    Some(true)
                } else {
                    None
                };
                "".to_owned()
            } else if !line.starts_with(" ") {
                // Replace "DefaultImporter:"(or other) with "type: DefaultImporter" and "content:"
                format!("type: {}\ncontent:\n", line.replace(":", ""))
            } else {
                // Keep the usual field lines
                let k = re.replace(line, "fileID: \"$1\"");
                format!("{k}\n")
            }
        })
        .collect::<String>()
        .add(
            format!(
                "fileFormatVersion: {}\nguid: {}\nfolderAsset: {}\n",
                file_format_version,
                guid,
                match folder_asset {
                    Some(ref s) => format!("{}\n", s),
                    None => "".to_string(),
                }
            )
            .as_str(),
        )
}

//----------------------------------------

pub fn readfile(dir: &str, file: &str) -> io::Result<String> {
    fs::read_to_string(format!("{dir}{file}"))
}

//----------------------------------------

pub fn unitypackage_open(filepath: &str) -> Archive<GzDecoder<File>> {
    let file = File::open(filepath).unwrap();
    let file = GzDecoder::new(file);
    Archive::new(file)
}

//----------------------------------------
