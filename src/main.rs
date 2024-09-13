mod errors;
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

const DESK_TEMPLATE: &str = include_str!("template.desktop");

fn process() -> Result<()> {
    let args = Args::parse();

    let bin_file_path = Path::new(&args.bin_file).canonicalize()?;

    let bin_file_metadata = fs::metadata(bin_file_path.clone())?;

    if bin_file_metadata.is_file() == false {
        return Err(Box::new(NotAFileError));
    }

    let executable = 0o0111;
    if bin_file_metadata.mode() & executable == 0 {
        return Err(Box::new(NotExecutableError));
    }

    let bin_file_name = bin_file_path.file_name().ok_or(NotAFileError)?;

    println!(
        "Selected executable file:\n     {}",
        bin_file_path.display()
    );

    let home_dir_str = env::var("HOME")?;
    let home_dir_path = Path::new(&home_dir_str);
    let local_apps_path = home_dir_path.join(Path::new(REL_LOCAL_APPLICATIONS_PATH));

    let desk_file_stem: PathBuf = bin_file_name.into();

    let desk_file_path = local_apps_path
        .join(&desk_file_stem)
        .with_extension(DESK_EXT);

    println!(
        "Target .desktop file:\n    {}",
        desk_file_path.display()
    );

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
        },
    };

    let bin_file_path_unicode = bin_file_path.to_str().ok_or(NonUnicodePathError)?;
    let bin_file_name_unicode = bin_file_name.to_str().ok_or(NonUnicodeNameError)?;

    let title_case_bin_file_name = bin_file_name_unicode.to_case(Case::Title);

    let local_icons_path = home_dir_path.join(Path::new(REL_LOCAL_ICONS_PATH));
    let icon_path = local_icons_path
        .join(&desk_file_stem)
        .with_extension(ICON_EXT);
    println!("Generating icon:  {}", icon_path.display());
    create_dir_all(local_icons_path)?;
    icon::draw_and_save_icon(bin_file_name_unicode, &icon_path);

    let icon_text = icon_path.to_str().ok_or(NonUnicodeNameError)?;

    let desk_text = DESK_TEMPLATE;

    // We only read the template, not a real .desktop file.
    // Assumptions made on the template format:
    //    - The first 4 lines are comments, headers or whitespace
    //    - All other lines are in the commented value-less form "# Keyname="
    let mut lines = desk_text.lines();

    let mut new_desk_text: String = "".to_string();
    // Copy first 3 lines
    new_desk_text.push_line(lines.next().unwrap());
    new_desk_text.push_line(lines.next().unwrap());
    new_desk_text.push_line(lines.next().unwrap());
    new_desk_text.push_line(lines.next().unwrap());
    
    for line in lines {
        let mut tokens = line.split([' ', '=']);
        tokens.next(); // Skip the first token (either comment marker or key)
        let key = tokens.next().unwrap(); // Get the second token (key)
        
        match key {
            "Name" => {
                new_desk_text.push_line(&format!("Name={title_case_bin_file_name}"));
            }
            "Exec" => {
                new_desk_text.push_line(&format!("Exec={bin_file_path_unicode}"));
            }
            "Terminal" => {
                if args.terminal {
                    new_desk_text.push_line("Terminal=true");
                } else {
                    new_desk_text.push_line("# Terminal=");
                }
            }
            "Keywords" => {
                if args.skip_misspell {
                    let keywords_text = misspellings(bin_file_name_unicode).join(",");
                    new_desk_text.push_line(&format!("Keywords={keywords_text}"));
                } else {
                    new_desk_text.push_line("# Keywords=");
                }
            }
            "Icon" => {
                new_desk_text.push_line(&format!("Icon={icon_text}"));
            }
            _ => {
                new_desk_text.push_line(line); // Copy all other lines
            }
        }
    }

    write!(desk_file, "{}", new_desk_text)?;

    if args.edit == true {
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
