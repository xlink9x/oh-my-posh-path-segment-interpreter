use anyhow::{Context, Result};
use clap::{arg, command, Parser};
use homedir::my_home;
use once_cell::sync::Lazy;
use std::{
    ffi::OsString,
    path::{Path, PathBuf, MAIN_SEPARATOR},
};

static ANCHOR_FILES: Lazy<Vec<OsString>> = Lazy::new(|| {
    vec![
        ".git", // Git root directory
        ".bzr",
        ".citc",
        ".hg",
        ".node-version",          // Node.js package
        ".python-version",        // Python package
        ".go-version",            // Go package
        ".ruby-version",          // Ruby package
        ".lua-version",           // Lua package
        ".java-version",          // Java package
        ".perl-version",          // Perl package
        ".php-version",           // PHP package
        ".tool-versions",         // Rust package
        ".shorten_folder_marker", // Git submodule
        ".svn",                   // Subversion
        "CVS",
        "Cargo.toml",        // Rust package
        "composer.json",     // PHP package
        "go.mod",            // Go module
        "package.json",      // JavaScript package
        "package-lock.json", // JavaScript package
        "yarn.lock",
        "stack.yaml",
        "requirements.txt",
        "go.work",
        "__main__.py",
        "init.lua",
    ]
    .iter()
    .map(|e| e.into())
    .collect()
});

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    stop_early: bool,

    #[arg(short, long, default_value_t = 0.8)]
    factor: f32,
}

fn main() {
    if let Ok(res) = interpret() {
        println!("{res}");
    } else {
        println!(":::::")
    }
}

fn interpret() -> Result<String> {
    let args = Args::parse();

    let mut result = String::from("");

    if let Some((term_width, _)) = term_size::dimensions() {
        // the real available width is 80% of the terminal width
        let width = (term_width as f32 * args.factor) as usize;
        let path = std::env::current_dir().context("parsing current dir")?;
        let user_home = my_home()?.context("")?;

        // start trimming
        let path = trim(&path, width, path.starts_with(&user_home), args.stop_early)?;
        let path = if let Ok(p) = path.strip_prefix(&user_home) {
            PathBuf::from(p)
        } else {
            path
        };

        let mut p_temp = PathBuf::from("~");
        p_temp.extend(&path);

        result = p_temp
            .to_str()
            .context("convert PathBuf to str")?
            .to_string();

        // make the last component bold
        if let Some(last) = result.split(MAIN_SEPARATOR).last() {
            result = result.replace(last, &format!("<b>{last}</b>"));
        }
    }

    Ok(result)
}

fn trim(path: &Path, width: usize, in_users_home: bool, stop_early: bool) -> Result<PathBuf> {
    // if we want to stop early and the path is already short enough, return
    if stop_early
        && path
            .to_str()
            .context("convert Path to str")?
            .chars()
            .count()
            <= width
    {
        return Ok(path.to_path_buf());
    }

    let components = path.iter().collect::<Vec<_>>();

    let mut out_components = vec![];

    let mut start_idx = 0;
    // if in home_dir, do not trim the /home/user part
    if in_users_home {
        start_idx += 3;
    }

    // iterate over all compoments, but not the last one, which should never be trimmed
    for i in start_idx..components.len() - 1 {
        let p = components[..=i].iter().collect::<PathBuf>();
        if p.read_dir()?
            .any(|entry| ANCHOR_FILES.contains(&entry.unwrap().file_name()))
        {
            out_components.push(PathBuf::from(components[i]));
        } else {
            let trimmed = &components[i]
                .to_str()
                .context("trim the path component to one char")?;
            let first_char = &trimmed[..1];

            // if hidden directory, use first two chars
            if first_char
                .chars()
                .next()
                .context("getting first char of trimmed component")?
                == '.'
            {
                out_components.push(PathBuf::from(&trimmed[0..2]));
            } else {
                out_components.push(PathBuf::from(first_char));
            }
        }

        // when the path is short enough already, stop early
        if stop_early {
            let mut temp = out_components.iter().collect::<PathBuf>();
            let remaining = components[i + 1..].iter().collect::<PathBuf>();
            temp.push(remaining);
            if temp
                .to_str()
                .context("convert PathBuf to str")?
                .chars()
                .count()
                <= width
            {
                return Ok(temp);
            }
        }
    }

    // push last component, because it should never be trimmed.
    // only do this, when we are not in the homedir itself
    if !out_components.is_empty() {
        out_components.push(
            components
                .last()
                .context("parsing the last element of path")?
                .into(),
        );
    }

    Ok(out_components.iter().collect::<PathBuf>())
}
