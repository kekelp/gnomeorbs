mod errors;
mod template;
mod icon;
mod extra;
use extra::*;
use camino::*;
use convert_case::*;
use errors::*;
use std::*;
use std::io::*;
use std::fs::*;
use std::os::unix::prelude::MetadataExt;
use clap::Parser;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Generate a .desktop file for a given executable file or script.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Target executable file or script
    #[clap(value_parser)]
    bin_file: String,

    /// If a .desktop file with the same name if found, overwrite it
    #[clap(short, long, value_parser)]
    overwrite: bool,

    /// Make the executable run in an interactive terminal
    #[clap(short, long, value_parser)]
    terminal: bool,

    /// After generating the .desktop file, open it with the default system editor
    #[clap(short, long, value_parser)]
    edit: bool,
    // todo: remove this and make sure that --edit without --overwrite works like that
    // If it already exists, open the matching .desktop file with the default system editor, and do nothing else
    // #[clap(long, short = 'E', value_parser)]
    // edit_only: bool,
}

const REL_LOCAL_APPLICATIONS_PATH: &str = ".local/share/applications/";
const REL_LOCAL_ICONS_PATH: &str = ".local/share/icons/hicolor/128x128/apps/";
const DESK_EXT: &str = "desktop";
const ICON_EXT: &str = "png";

fn select_exe_file(args: &Args) -> Result<Utf8PathBuf> {
    let exe_file = Utf8Path::new(&args.bin_file).canonicalize_utf8()?;

    let metadata = fs::metadata(exe_file.clone())?;

    if metadata.is_file() == false {
        return Err(Box::new(NotAFileError));
    }

    let executable = 0o0111;
    if metadata.mode() & executable == 0 {
        return Err(Box::new(NotExecutableError));
    }

    println!(
        "selected executable file:\n    {}",
        exe_file
    );

    return Ok(exe_file);
}

fn process() -> Result<()> {
    let args = Args::parse();

    let exe_file = select_exe_file(&args)?; 

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

    println!("target .desktop file:\n    {}", desktop_file_path);

    let desktop_file_opt = OpenOptions::new()
        .write(true)
        .create_conditional(args.overwrite)
        .open(desktop_file_path.clone());

    let mut desktop_file = match desktop_file_opt {
        Ok(df) => df,
        Err(err) => {
            if err.kind() == ErrorKind::AlreadyExists {
                return Err(Box::new(CustomAlreadyExistsError));
            }
            return Err(Box::new(err));
        }
    };

    let icon_path = local_icons_path
        .join(&desktop_file_stem)
        .with_extension(ICON_EXT);
    println!("generating icon:\n    {}", icon_path);
    create_dir_all(local_icons_path)?;
    icon::draw_and_save_icon(exe_filename, &icon_path);

    let mut buffer = String::with_capacity(800);

    buffer.push_str(&template::HEADER);

    buffer.pushln2("Name=", &title_case_exe_filename);
    buffer.pushln2("Exec=", exe_file.as_str());
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
