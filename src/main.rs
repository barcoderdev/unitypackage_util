//----------------------------------------

mod commands;
mod package;

//----------------------------------------

use clap::{Parser, Subcommand};
use std::path::PathBuf;

//----------------------------------------

// https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html
// https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html
// https://serde.rs

//----------------------------------------

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    arg_required_else_help = true,
    subcommand_required = true
)]
struct Cli {
    /// Unity Package (Tar, TarGz, or Folder)
    package: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

//----------------------------------------

#[derive(Subcommand)]
enum Commands {
    // Test,
    /// Find source of dump crashes
    Debug,
    /// Show package info
    Info,
    /// Display path from guid/pathname file
    Name {
        guid: String,
    },
    /// Dump package contents
    Dump {
        /// Pretty Print JSON
        #[arg(short, long)]
        pretty: bool,
    },
    /// List package contents
    List {
        /// Hide GUIDs
        #[arg(short, long)]
        no_guid: bool,

        /// Pretty Print JSON
        #[arg(short, long)]
        pretty: bool,

        /// Directory Filter
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// Extract package file
    Extract {
        #[arg(required = true)]
        guid: Option<String>,

        /// Extract to file
        #[arg(short, long)]
        output_file: Option<PathBuf>,

        /// Extract /asset.meta file instead of /asset
        #[arg(short, long)]
        meta: bool,

        /// Process yaml to json
        #[arg(short, long)]
        json: bool,

        /// Pretty Print JSON
        #[arg(short, long)]
        pretty: bool,

        /// Convert FBX to GLTF
        #[arg(short, long)]
        fbx2gltf: bool,

        /// Base64 encode output
        #[arg(short, long)]
        base64: bool,
    },
    /// Calculate xxhash 64 of string
    XxHash {
        #[arg(required = true)]
        text: String,
    }
}

//----------------------------------------

fn main() {
    let cli = Cli::parse();

    let package_path = cli.package.to_str().unwrap();

    match &cli.command {
        &Some(Commands::Info) => {
            println!(
                "Package: {}",
                package::Package::new(package_path).unwrap()
            );
        }
        &Some(Commands::Dump { pretty }) => {
            commands::package_contents_dump(package_path, pretty, false);
        }
        &Some(Commands::Name { ref guid }) => {
            commands::package_contents_name(package_path, guid);
        }
        Some(Commands::List {
            no_guid,
            pretty,
            dir,
        }) => {
            commands::package_contents_list(package_path, dir, !*no_guid, *pretty);
        }
        &Some(Commands::Extract {
            ref guid,
            ref output_file,
            meta,
            json,
            pretty,
            fbx2gltf,
            base64,
        }) => {
            commands::package_file_extract(
                package_path,
                guid.as_ref().unwrap(),
                output_file,
                meta,
                json,
                pretty,
                fbx2gltf,
                base64,
            );
        }
        // &Some(Commands::Test) => {
        //     let package = package::Package::new(package_path);
        //     // println!("{:?}", package);

        //     if let Ok(package) = package {
        //         let result = package.open();
        //         for file in result.unwrap().entries() {
        //             let size = file.as_ref().unwrap().size().unwrap();
        //             let path = file.as_ref().unwrap().path().unwrap();
        //             println!("{} {}", size, path.display());
        //         }
        //     }
        // }
        &Some(Commands::Debug) => {
            commands::package_contents_dump(package_path, false, true);
        }
        &Some(Commands::XxHash { ref text }) => {
            commands::xx_hash(text);
        }
        &None => (),
    }
}

//----------------------------------------
