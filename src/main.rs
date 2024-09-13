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

    /// Run the executable in an interactive terminal
    #[clap(short, long, value_parser)]
    terminal: bool,

    /// Don't add misspellings of the name as keywords
    #[clap(long, short = 'm', value_parser)]
    skip_misspell: bool,

    /// Use an existing icon instead of generating one
    #[clap(long, short, value_parser)]
    icon: Option<String>,

    /// After generating the .desktop file, open it with the default system editor
    #[clap(short, long, value_parser)]
    edit: bool,

    /// If it already exists, open the matching .desktop file with the default system editor, and do nothing else
    #[clap(long, short = 'E', value_parser)]
    edit_only: bool,
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

    if bin_file_metadata.mode() & 0o0111 == 0 {
        return Err(Box::new(NotExecutableError));
    }

    let bin_file_name = bin_file_path.file_name().ok_or(NotAFileError)?;

    println!(
        "Selected executable file:       {}",
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
        "Target .desktop file:           {}",
        desk_file_path.display()
    );

    if args.edit_only == true {
        match edit::edit_file(desk_file_path) {
            x => {
                println!("{:?}", x);
            }
        }
        return Ok(());
    }

    create_dir_all(local_apps_path)?;
    let mut desk_file = match OpenOptions::new()
        .write(true)
        .create_conditional(args.overwrite)
        .open(desk_file_path.clone())
    {
        Ok(df) => df,
        Err(err) => match err.kind() {
            ErrorKind::AlreadyExists => return Err(Box::new(CustomAlreadyExistsError)),
            _ => return Err(Box::new(err)),
        },
    };

    let bin_file_path_unicode = bin_file_path.to_str().ok_or(NonUnicodePathError)?;
    let bin_file_name_unicode = bin_file_name.to_str().ok_or(NonUnicodeNameError)?;

    let title_case_bin_file_name = bin_file_name_unicode.to_case(Case::Title);

    // Generate icon
    let icon_text;
    // Feels weird that I have to declare these 2 here and bind the matched values to them
    let icon_arg;
    let icon_path;
    match args.icon {
        Some(icon_arg_temp) => {
            icon_arg = icon_arg_temp;
            match is_path_and_exists(&icon_arg) {
                Some(icon_path_value) => {
                    icon_path = icon_path_value;
                    println!("Using icon path:  {}", icon_path.display());
                    icon_text = icon_path.to_str().ok_or(NonUnicodeNameError)?
                }
                None => {
                    // Check if the given argument can work as an icon name?
                    // https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html#icon_lookup
                    println!("Using icon name:  {}", icon_arg);
                    icon_text = &icon_arg
                }
            }
        }
        None => {
            let local_icons_path = home_dir_path.join(Path::new(REL_LOCAL_ICONS_PATH));
            icon_path = local_icons_path
                .join(&desk_file_stem)
                .with_extension(ICON_EXT);
            println!("Generating icon:  {}", icon_path.display());
            create_dir_all(local_icons_path)?;
            icon::draw_and_save_icon(bin_file_name_unicode, &icon_path);
            icon_text = icon_path.to_str().ok_or(NonUnicodeNameError)?
        }
    };

    // let icon_text = icon_path.to_str().ok_or(NonUnicodeNameError)?;

    let desk_text = DESK_TEMPLATE;

    // We only read the template, not a real .desktop file.
    // Assumptions made on the template format:
    //    - The first 4 lines are comments, headers or whitespace
    //    - All other lines are in the commented value-less form "# Keyname="
    let mut lines = desk_text.lines();

    let mut new_desk_text: String = "".to_string();
    // Copy first 3 lines
    new_desk_text.manypush(&[lines.next().unwrap(), "\n"]);
    new_desk_text.manypush(&[lines.next().unwrap(), "\n"]);
    new_desk_text.manypush(&[lines.next().unwrap(), "\n"]);
    new_desk_text.manypush(&[lines.next().unwrap(), "\n"]);

    for line in lines {
        let mut tokens = line.split([' ', '=']);
        // The line below does two things at once:
        // - For commented lines, skip the 1st token which is #
        // - For non-commented lines (hardcoded to a default value, like Type=Application), skip the 1st token, which is the key,
        //       so that the value won't match any known key.
        //    Will break if I decide to hardcode some very stupid defaults like Type=PrefersNonDefaultGPU where the value for some key is also the name of a key.
        tokens.next();
        let key = tokens.next().unwrap();
        match key {
            "Name" => {
                new_desk_text.manypush(&["Name=", &title_case_bin_file_name, "\n"]);
            }
            "Exec" => {
                match args.terminal {
                    true => new_desk_text.manypush(&[
                        "Exec=bash -c '",
                        &bin_file_path_unicode,
                        ";$SHELL'",
                        "\n",
                    ]),
                    false => new_desk_text.manypush(&["Exec=", &bin_file_path_unicode, "\n"]),
                };
            }
            "Terminal" => match args.terminal {
                true => new_desk_text += "Terminal=true\n",
                false => new_desk_text += "# Terminal=\n",
            },
            "Keywords" => match args.skip_misspell {
                true => {
                    let keywords_text = misspellings(bin_file_name_unicode).join(",");
                    new_desk_text.manypush(&["Keywords=", &keywords_text, "\n"]);
                }
                false => new_desk_text += "# Keywords=\n",
            },
            "Icon" => {
                new_desk_text.manypush(&["Icon=", &icon_text, "\n"]);
            }
            // Copy all other lines
            _ => {
                new_desk_text.manypush(&[line, "\n"]);
            }
        };
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
