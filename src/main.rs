mod errors;
mod template;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use convert_case::Case;
use convert_case::Casing;
use errors::*;

mod icon;

mod helper;
use helper::*;

use std::env;
use std::error;
use std::fs;
use std::fs::create_dir_all;
use std::io::ErrorKind;

use clap::Parser;
use std::fs::OpenOptions;
use std::os::unix::prelude::MetadataExt;

use std::io::Write;

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

    /// Don't add misspellings of the name as keywords
    #[clap(long, short = 'm', value_parser)]
    skip_misspell: bool,

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
    let bin_file_path = Utf8Path::new(&args.bin_file).canonicalize_utf8()?;

    let metadata = fs::metadata(bin_file_path.clone())?;

    if metadata.is_file() == false {
        return Err(Box::new(NotAFileError));
    }

    let executable = 0o0111;
    if metadata.mode() & executable == 0 {
        return Err(Box::new(NotExecutableError));
    }

    println!(
        "selected executable file:\n    {}",
        bin_file_path
    );

    return Ok(bin_file_path);
}

fn process() -> Result<()> {
    let args = Args::parse();

    let bin_file_path = select_exe_file(&args)?; 

    let home_dir_str = env::var("HOME")?;
    let home_dir_path = Utf8Path::new(&home_dir_str);
    let local_apps_path = home_dir_path.join(Utf8Path::new(REL_LOCAL_APPLICATIONS_PATH));
    let local_icons_path = home_dir_path.join(Utf8Path::new(REL_LOCAL_ICONS_PATH));


    let bin_file_name = bin_file_path.file_name().ok_or(NotAFileError)?;
    let title_case_bin_file_name = bin_file_name.to_case(Case::Title);
    let desk_file_stem: Utf8PathBuf = bin_file_name.into();

    let desk_file_path = local_apps_path
        .join(&desk_file_stem)
        .with_extension(DESK_EXT);

    println!("target .desktop file:\n    {}", desk_file_path);

    let desk_file_opt = OpenOptions::new()
        .write(true)
        .create_conditional(args.overwrite)
        .open(desk_file_path.clone());

    let mut desk_file = match desk_file_opt {
        Ok(df) => df,
        Err(err) => {
            if err.kind() == ErrorKind::AlreadyExists {
                return Err(Box::new(CustomAlreadyExistsError));
            }
            return Err(Box::new(err));
        }
    };

    let icon_path = local_icons_path
        .join(&desk_file_stem)
        .with_extension(ICON_EXT);
    println!("generating icon:\n    {}", icon_path);
    create_dir_all(local_icons_path)?;
    icon::draw_and_save_icon(bin_file_name, &icon_path);

    let mut buffer: String = String::with_capacity(800);

    buffer.push_str(&template::HEADER);

    buffer.pushln2("Name=", &title_case_bin_file_name);
    buffer.pushln2("Exec=", bin_file_path.as_str());
    buffer.pushln2("Icon=", &icon_path.as_str());

    // todo: this is flipped lol
    if args.skip_misspell {
        let keywords = misspellings(bin_file_name).join(",");
        buffer.pushln2("Keywords=", &keywords);
    } else {
        buffer.pushln("# Keywords=");
    }

    if args.terminal {
        buffer.pushln("Terminal=true");
    } else {
        buffer.pushln("# Terminal=");
    }

    buffer.push_str(&template::EMPTY_LINES);

    desk_file.write_all(buffer.as_bytes())?;
    desk_file.flush()?;

    if args.edit {
        println!("Opening target file with default editor...");
        edit::edit_file(desk_file_path)?;
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

fn misspellings(text: &str) -> Vec<String> {
    let mut results = Vec::<String>::new();

    // inverted letters
    let n = std::cmp::min(text.len() - 1, 4);
    for i in 0..n {
        results.push(invert_letters(text, i, i + 1));
    }

    // missing letters
    if n >= 3 {
        results.push(text[0..2].to_string() + &text[3..]);
    }
    if n >= 2 {
        results.push(text[0..1].to_string() + &text[2..]);
    }
    results.push(text[1..].to_string());

    return results;
}

fn invert_letters(text: &str, i1: usize, i2: usize) -> String {
    let characs: Vec<char> = text.chars().collect();

    let mut char_vec_1 = Vec::<char>::new();

    for (i, c) in characs.iter().enumerate() {
        let nc = match i {
            i if i == i1 => characs[i2],
            i if i == i2 => characs[i1],
            _ => *c,
        };
        char_vec_1.push(nc);
    }
    let result: String = char_vec_1.into_iter().collect();
    return result;
}
