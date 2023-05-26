//----------------------------------------

use std::{
    collections::HashMap,
    io::prelude::*,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use unitypackage_util;

use xxhash_rust::xxh64;

use crate::package;

//----------------------------------------

#[derive(Debug, Serialize)]
struct Dump {
    pathname: Option<String>,
    content_type: Option<String>,
    asset: Option<serde_yaml::Value>,
    asset_meta: Option<serde_yaml::Value>,
}

//----------------------------------------

fn yaml_matcher(buf: &[u8]) -> bool {
    let sig = b"%YAML";
    return buf.len() >= sig.len() && buf[0..sig.len()].cmp(sig) == std::cmp::Ordering::Equal;
}

//----------------------------------------

fn fbx_matcher(buf: &[u8]) -> bool {
    let sig = b"Kaydara FBX Binary";
    return buf.len() >= sig.len() && buf[0..sig.len()].cmp(sig) == std::cmp::Ordering::Equal;
}

//----------------------------------------

pub fn deserializer(yaml: &String) -> serde_yaml::Value {
    serde_yaml::Deserializer::from_str(&yaml)
        .map(|doc| <serde_yaml::Value>::deserialize(doc).unwrap())
        .collect()
}

/*
// This allows it to continue but removes the whole sub-document if it fails to deserialize
// unitypackage_godot needs to be update to handle this before implementing

    serde_yaml::Deserializer::from_str(&yaml)
        .map(|doc| {
            let res = <serde_yaml::Value>::deserialize(doc);
            match res {
                Ok(res) => res,
                Err(_e) => {
                    // std::io::stderr()
                    //     .write_all(format!("Deserialize Error: {:?}\n", e).as_bytes())
                    //     .unwrap();
                    serde_yaml::Value::String("".to_owned())
                }
            }
        })
        .collect()
 */
//----------------------------------------

pub fn package_contents_dump(package_file: &str, pretty: bool, debug: bool) {
    let mut data = HashMap::<String, Dump>::new();

    let mut info = infer::Infer::new();
    info.add("text/yaml", "yaml", yaml_matcher);
    info.add("data/fbx", "fbx", fbx_matcher);
    // info.add("text/json", "json", json_matcher);

    for file in package::Package::new(package_file)
        .unwrap()
        .open()
        .unwrap()
        .entries()
    {
        let mut file = file.unwrap();

        let file_path = file.path().unwrap().to_str().unwrap().to_owned();
        let size = file.size().unwrap();

        let guid = file.guid();
        if guid.len() < 32 {
            continue;
        }

        let entry = data.entry(guid.clone()).or_insert_with(|| Dump {
            pathname: None,
            content_type: None,
            asset: None,
            asset_meta: None,
        });

        // println!("checking pathname");
        if file_path.ends_with("/pathname") {
            if debug {
                println!("{}/pathname", guid);
            }

            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();

            entry.pathname = Some(s.split("\n").next().unwrap().to_owned());
        }

        // println!("checking asset.meta");
        if file_path.ends_with("/asset.meta") {
            if debug {
                println!("{}/asset.meta", guid);
            }

            let mut yaml = String::new();
            file.read_to_string(&mut yaml).unwrap();

            let yaml = unitypackage_util::asset_meta_yaml_cleanup(&yaml);
            let deserialized = deserializer(&yaml);

            entry.asset_meta = Some(deserialized);
        }

        // println!("checking asset");
        if file_path.ends_with("/asset") {
            if debug {
                println!("{}/asset", guid);
            }

            let mut buffer = Vec::with_capacity(size);
            file.read_to_end(&mut buffer).unwrap();

            if let Some(content_type) = info.get(&buffer) {
                entry.content_type = Some(content_type.mime_type().to_owned());
            }

            if buffer.starts_with(b"%YAML") {
                let yaml = std::str::from_utf8(&buffer).unwrap();
                let yaml = unitypackage_util::asset_yaml_cleanup(&yaml);
                let deserialized = deserializer(&yaml);

                entry.asset = Some(deserialized);
            }
        }
    }

    if !debug {
        println!("{}", serde_json_to_string(&data, pretty));
    }
}

//----------------------------------------

pub fn package_contents_name(package_file: &str, guid: &str) {
    let looking_for = format!("{}/pathname", guid);

    for file in package::Package::new(package_file)
        .unwrap()
        .open()
        .unwrap()
        .entries()
    {
        let mut file = file.unwrap();

        let file_path = file.path().unwrap().to_str().unwrap().to_owned();
        // let size = file.size().unwrap();

        if file_path == looking_for {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();

            println!("{}", s.split("\n").next().unwrap());
            return;
        }
    }

    std::io::stderr()
        .write_all(format!("Could not find {} in package\n", looking_for).as_bytes())
        .unwrap();
    std::process::exit(exitcode::NOINPUT);
}

//----------------------------------------

pub fn package_contents_list(
    package_file: &str,
    dir: &Option<String>,
    with_guids: bool,
    pretty: bool,
) {
    let mut contents = Vec::new();

    for file in package::Package::new(package_file)
        .unwrap()
        .open()
        .unwrap()
        .entries()
    {
        let mut file = file.unwrap();

        let file_path = file.path().unwrap().to_str().unwrap().to_owned();
        // let size = file.size().unwrap();

        let guid = file.guid();
        if guid.len() < 32 {
            continue;
        }

        if file_path.ends_with("/pathname") {
            let mut pathname = String::new();
            file.read_to_string(&mut pathname).unwrap();

            let mut include = true;

            if let Some(dir) = dir {
                let tmp_file_path = format!("{}", pathname.split("\n").next().unwrap());

                let path = Path::new(&tmp_file_path);
                let parent = path.parent().unwrap().to_str().unwrap();

                //println!("Looking for {} in {} || {}", dir, s, parent);
                include = parent == dir;
            }

            if include {
                contents.push([guid.to_owned(), pathname.split("\n").next().unwrap().to_owned()]);
            }
        }
    }

    contents.sort_by(|a, b| a[1].cmp(&b[1]));

    if with_guids {
        print!("{}", serde_json_to_string(&contents, pretty));
    } else {
        let output = contents
            .iter()
            .map(|item| item[1].to_owned())
            .collect::<Vec<String>>();
        print!("{}", serde_json_to_string(&output, pretty));
    }
}

//----------------------------------------

fn serde_json_to_string<T: Serialize>(json: &T, pretty: bool) -> String {
    let printer = if pretty {
        serde_json::to_string_pretty
    } else {
        serde_json::to_string
    };

    printer(&json).unwrap()
}

//----------------------------------------

fn yaml_to_json<'a, T: Deserialize<'a> + Serialize>(yaml: &'a str, pretty: bool) -> String {
    let output = serde_yaml::Deserializer::from_str(&yaml)
        .map(|doc| T::deserialize(doc).unwrap())
        .map(|doc| serde_json_to_string(&doc, pretty))
        .collect::<Vec<String>>()
        .join(",");

    format!("[{}]]\n", output)
}

//----------------------------------------

pub fn package_file_extract(
    package_file: &str,
    guid: &str,
    _output_file: &Option<PathBuf>,
    meta: bool,
    json: bool,
    pretty: bool,
    fbx2gltf: bool,
    base64: bool,
) {
    let looking_for = format!("{}/{}", guid, if meta { "asset.meta" } else { "asset" });

    for file in package::Package::new(package_file)
        .unwrap()
        .open()
        .unwrap()
        .entries()
    {
        let mut file = file.unwrap();

        let file_path = file.path().unwrap().to_str().unwrap().to_owned();
        let size = file.size().unwrap();

        if file_path == looking_for {
            let mut buffer = Vec::with_capacity(size);
            file.read_to_end(&mut buffer).unwrap();

            if json {
                let yaml = std::str::from_utf8(&buffer).unwrap();
                if meta {
                    let yaml = unitypackage_util::asset_meta_yaml_cleanup(&yaml);
                    print!("{}", yaml_to_json::<serde_json::Value>(&yaml, pretty));
                } else {
                    let yaml = unitypackage_util::asset_yaml_cleanup(&yaml);
                    print!("{}", yaml_to_json::<serde_json::Value>(&yaml, pretty));
                }
            } else {
                if fbx2gltf {
                    convert_fbx2gltf(&mut buffer, base64).unwrap();
                } else if base64 {
                    print!("{}", general_purpose::STANDARD.encode(buffer));
                } else {
                    std::io::stdout().write_all(&buffer).unwrap();
                }
            }

            return;
        }
    }

    std::io::stderr()
        .write_all(format!("Could not find {} in package\n", looking_for).as_bytes())
        .unwrap();
    std::process::exit(exitcode::NOINPUT);
}

//----------------------------------------

pub fn convert_fbx2gltf(buf: &mut Vec<u8>, base64: bool) -> Result<(), std::io::Error> {
    let program = if cfg!(windows) {
        "FBX2glTF.exe"
    } else {
        "./FBX2glTF"
    };

    let process = Command::new(program)
        .arg("-IO")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    process.stdin.unwrap().write_all(buf)?;

    let mut buffer = Vec::new();
    process.stdout.unwrap().read_to_end(&mut buffer)?;

    if base64 {
        print!("{}", general_purpose::STANDARD.encode(buffer));
        return Ok(());
    }

    std::io::stdout().write_all(&buffer)?;
    Ok(())
}

//----------------------------------------

pub fn xx_hash(text: &str) {
    print!("{}", xxh64::xxh64(text.as_bytes(), 0) as i64)
}

//----------------------------------------
