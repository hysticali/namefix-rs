use std::path::Path;
use std::fs;
use std::error::Error;
use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Directory or file to process
    path: String,

    /// Show what would be done without making changes
    #[arg(short, long, action = ArgAction::SetTrue)]
    dry_run: bool,

    /// Path to log file
    #[arg(long)]
    log_file: Option<String>,
}

fn clean_filename(filename: &str) -> String {
    // Remove control characters from the filename
    filename
        .split(|c: char| c.is_control())
        .collect::<Vec<&str>>()
        .join("")
}

fn process_directory(path: &Path, dry_run: bool) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }

    let metadata = path.metadata()?;
    if metadata.permissions().readonly() {
        return Err(format!("Path is readonly, cannot modify: {}", path.display()).into());
    }

    if path.is_dir() {
        let entries: Vec<_> = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                process_directory(&path, dry_run)?;
            } else {
                process_file(&path, dry_run)?;
            }
        }
        process_file(path, dry_run)?;
    } else {
        process_file(path, dry_run)?;
    }

    Ok(())
}

fn process_file(path: &Path, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let old_name = path.file_name()
        .ok_or_else(|| format!("Invalid filename for path: {}", path.display()))?
        .to_string_lossy();
    let new_name = clean_filename(&old_name);

    if old_name != new_name {
        let new_path = path.with_file_name(new_name);
        if dry_run {
            println!("Would rename: {} -> {}", path.display(), new_path.display());
        } else {
            fs::rename(&path, &new_path)?;
            println!("Renamed: {} -> {}", path.display(), new_path.display());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Initialize logging if log_file is specified
    if let Some(log_file) = args.log_file {
        use std::fs::File;
        use env_logger::{Builder, Target};

        let file = File::create(log_file)?;
        Builder::new()
            .target(Target::Pipe(Box::new(file)))
            .init();
    } else {
        env_logger::init();
    }

    let path = Path::new(&args.path);
    process_directory(path, args.dry_run)?;

    Ok(())
}