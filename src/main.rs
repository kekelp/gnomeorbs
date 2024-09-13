mod errors;
mod template;
mod icon;
mod extra;
mod misspellings;
use extra::*;
use misspellings::*;
use camino::*;
use convert_case::*;
use errors::*;
use std::*;
use std::io::*;
use std::fs::*;
use std::os::unix::prelude::MetadataExt;
use clap::Parser;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Generate a .desktop file for a given executable file or script
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Target executable file or script
    #[clap(value_parser)]
    executable_file: String,

    /// Overwrite .desktop files or icons if they are already present
    #[clap(short, long, value_parser)]
    overwrite: bool,

    /// Make the executable run in an interactive terminal
    #[clap(short, long, value_parser)]
    terminal: bool,

    /// After generating the .desktop file, open it with the default system editor
    #[clap(short, long, value_parser)]
    edit: bool,
}

const REL_LOCAL_APPLICATIONS_PATH: &str = ".local/share/applications/";
const REL_LOCAL_ICONS_PATH: &str = ".local/share/icons/hicolor/128x128/apps/";
const DESK_EXT: &str = "desktop";
const ICON_EXT: &str = "png";

fn process() -> Result<()> {
    let args = Args::parse();

    // select executable file
    let exe_file = Utf8Path::new(&args.executable_file).canonicalize_utf8()?;
    let metadata = fs::metadata(exe_file.clone())?;

    if metadata.is_file() == false {
        return Err(Box::new(NotAFileError));
    }

    let executable = 0o0111;
    if metadata.mode() & executable == 0 {
        return Err(Box::new(NotExecutableError));
    }

    println!(
        "Selected executable file:\n    {}",
        exe_file
    );

    // determine target .desktop file
    let home_dir = env::var("HOME")?;
    let home_dir = Utf8Path::new(&home_dir);
    let local_apps_path = home_dir.join(Utf8Path::new(REL_LOCAL_APPLICATIONS_PATH));
    let local_icons_path = home_dir.join(Utf8Path::new(REL_LOCAL_ICONS_PATH));

    let exe_filename = exe_file.file_name().ok_or(NotAFileError)?;
    let title_case_exe_filename = exe_filename.to_case(Case::Title);
    let desktop_file_stem: Utf8PathBuf = exe_filename.into();

    let desktop_file_path = local_apps_path
        .join(&desktop_file_stem)
        .with_extension(DESK_EXT);

    println!("Target .desktop file:\n    {}", desktop_file_path);

    // write .desktop file    
    let should_write = args.overwrite | (Utf8Path::exists(&desktop_file_path) == false);
    if should_write {
        let mut desktop_file = File::create(&desktop_file_path)?;

        let icon_path = local_icons_path
            .join(&desktop_file_stem)
            .with_extension(ICON_EXT);

        let should_write_icon = args.overwrite | (Utf8Path::exists(&icon_path) == false);

        if should_write_icon {
            create_dir_all(local_icons_path)?;
            icon::draw_and_save_icon(exe_filename, &icon_path);
            println!("Generated icon:\n    {}", icon_path);
        } else {
            println!("Icon already exists, not touching it.");
            println!("You can use --overwrite to overwrite both the .desktop file and the icon.");
        }

        let mut buffer = String::with_capacity(800);

        buffer.push_str(&template::HEADER);

        buffer.pushln2("Name=", &title_case_exe_filename);

        if args.terminal {
            let terminal_exec = format!("Exec=bash -c {exe_file}; $SHELL");
            buffer.pushln(&terminal_exec)
        } else {
            buffer.pushln2("Exec=", exe_file.as_str());
        }

        buffer.pushln2("Icon=", &icon_path.as_str());

        let keywords = misspellings(exe_filename).join(",");
        buffer.pushln2("Keywords=", &keywords);

        if args.terminal {
            buffer.pushln("Terminal=true");
        } else {
            buffer.pushln("# Terminal=");
        }

        buffer.push_str(&template::EMPTY_LINES);

        desktop_file.write_all(buffer.as_bytes())?;
        desktop_file.flush()?;
    } else {
        println!("Target .desktop file already exists, not touching it.");
        if args.edit {
            println!("You can use --overwrite to overwrite it.");
        } else {
            println!("You can use --overwrite to overwrite it,");
            println!("or --edit to open the existing one with the system default editor.");
        }
    }

    // open with default editor
    if args.edit {
        println!("Opening target file with default editor...");
        edit::edit_file(desktop_file_path)?;
    }

    return Ok(());
}

fn main() {
    let result = process();
    match result {
        Err(err) => {
            println!("Error: {}", err);
        }
        Ok(()) => {
            println!("Completed.");
        }
    }
}
