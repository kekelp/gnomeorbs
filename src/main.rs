mod errors;
mod template;
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
use std::io::Write;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::path::PathBuf;

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

fn select_exe_file(args: &Args) -> Result<PathBuf> {
    let bin_file_path = Path::new(&args.bin_file).canonicalize()?;

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
        bin_file_path.to_string_lossy()
    );

    return Ok(bin_file_path);
}

fn process() -> Result<()> {
    let args = Args::parse();

    let bin_file_path = select_exe_file(&args)?; 

    let home_dir_str = env::var("HOME")?;
    let home_dir_path = Path::new(&home_dir_str);
    let local_apps_path = home_dir_path.join(Path::new(REL_LOCAL_APPLICATIONS_PATH));

    let bin_file_name = bin_file_path.file_name().ok_or(NotAFileError)?;
    let desk_file_stem: PathBuf = bin_file_name.into();

    let desk_file_path = local_apps_path
        .join(&desk_file_stem)
        .with_extension(DESK_EXT);

    println!("target .desktop file:\n    {}", desk_file_path.to_string_lossy());

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

    let bin_file_path_unicode = bin_file_path.to_str().ok_or(NonUnicodePathError)?;
    let bin_file_name_unicode = bin_file_name.to_str().ok_or(NonUnicodeNameError)?;

    let title_case_bin_file_name = bin_file_name_unicode.to_case(Case::Title);

    let local_icons_path = home_dir_path.join(Path::new(REL_LOCAL_ICONS_PATH));
    let icon_path = local_icons_path
        .join(&desk_file_stem)
        .with_extension(ICON_EXT);
    println!("generating icon:\n    {}", icon_path.to_string_lossy());
    create_dir_all(local_icons_path)?;
    icon::draw_and_save_icon(bin_file_name_unicode, &icon_path);

    let icon_text = icon_path.to_str().ok_or(NonUnicodeNameError)?;

    let mut buffer: Vec<u8> = Vec::with_capacity(800);

    write!(buffer, "{}", template::HEADER)?;

    writeln!(buffer, "Name={}", title_case_bin_file_name)?;
    writeln!(buffer, "Exec={}", bin_file_path_unicode)?;
    
    if args.terminal {
        writeln!(buffer, "Terminal=true")?;
    } else {
        writeln!(buffer, "# Terminal=")?;
    }
    
    if args.skip_misspell {
        let keywords_text = misspellings(bin_file_name_unicode).join(",");
        writeln!(buffer, "Keywords={keywords_text}")?;
    } else {
        writeln!(buffer, "# Keywords=")?;
    }

    writeln!(buffer, "Icon={}", icon_text)?;

    write!(buffer, "{}", template::EMPTY_LINES)?;


    desk_file.write_all(&buffer)?;
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
