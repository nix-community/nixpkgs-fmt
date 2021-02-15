use std::{
    fmt::Write,
    fs,
    io::{stdin, Read},
    path::{Path, PathBuf},
    thread,
};

use clap::{App, Arg};
use crossbeam_channel::{unbounded, Receiver, Sender};
use rnix::types::TypedNode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type FormatResult = (PathBuf, FormatStatus);
enum FormatStatus {
    Change,
    NoChange,
}

fn main() {
    if let Err(err) = parse_args().and_then(try_main) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

#[derive(Debug)]
struct Args {
    src: Src,
    operation: Operation,
}

#[derive(Debug)]
enum Operation {
    Fmt { write_changes: bool, fail_on_changes: bool },
    Explain,
    Parse { output_format: OutputFormat },
}

#[derive(Debug)]
enum Src {
    Stdin,
    Paths(Vec<PathBuf>),
}

#[derive(Debug)]
enum OutputFormat {
    Default,
    Json,
}

fn parse_args() -> Result<Args> {
    let matches = App::new("nixpkgs-fmt")
        .version(clap::crate_version!())
        .about("Format Nix code")
        .arg(
            Arg::with_name("srcs")
                .value_name("FILE")
                .multiple(true)
                .conflicts_with("explain")
                .help("File to reformat in place. If no file is passed, read from stdin."),
        )
        .arg(
            Arg::with_name("parse")
                .long("parse")
                .conflicts_with("explain")
                .conflicts_with("check")
                .help("Show syntax tree instead of reformatting"),
        )
        .arg(
            Arg::with_name("output-format")
                .long("output-format")
                .requires("parse")
                .possible_values(&["json"])
                .help("Output syntax tree in JSON format"),
        )
        .arg(
            Arg::with_name("explain")
                .long("explain")
                .conflicts_with("parse")
                .conflicts_with("check")
                .help("Show which rules are violated"),
        )
        .arg(
            Arg::with_name("check")
                .long("check")
                .conflicts_with("parse")
                .conflicts_with("explain")
                .help("Only test if the formatter would produce differences"),
        )
        .get_matches_safe()?;

    let src = match matches.values_of("srcs") {
        None => Src::Stdin, // default to reading from stdin
        Some(srcs) => Src::Paths(srcs.map(PathBuf::from).collect()),
    };
    let operation = if matches.is_present("parse") {
        let output_format = match matches.value_of("output-format") {
            Some("json") => OutputFormat::Json,
            _ => OutputFormat::Default,
        };
        Operation::Parse { output_format }
    } else if matches.is_present("explain") {
        Operation::Explain
    } else if matches.is_present("check") {
        Operation::Fmt { write_changes: false, fail_on_changes: true }
    } else {
        Operation::Fmt { write_changes: true, fail_on_changes: false }
    };

    Ok(Args { operation, src })
}

fn try_main(args: Args) -> Result<()> {
    match args.operation {
        Operation::Fmt { write_changes, fail_on_changes } => match &args.src {
            Src::Stdin => {
                let input = read_stdin_to_string()?;
                let output = nixpkgs_fmt::reformat_string(&input);
                let has_changes = input != output;
                if write_changes {
                    print!("{}", output);
                }
                if fail_on_changes && has_changes {
                    return Err("error: fail on changes".into());
                }
            }
            Src::Paths(paths) => {
                let (sender, receiver): (Sender<FormatResult>, Receiver<FormatResult>) =
                    unbounded();

                // Reducer, collect all the paths and statuses that have been seen
                let reducer = thread::spawn(move || {
                    let mut files_count = 0;
                    let mut files_changed = 0;
                    for (file_path, status) in receiver {
                        files_count += 1;
                        if let FormatStatus::Change = status {
                            files_changed += 1;
                            println!("{}", file_path.display());
                        }
                    }
                    (files_count, files_changed)
                });

                // Start formatting
                for path in paths {
                    if path.is_dir() {
                        reformat_dir_in_place(path, write_changes, &sender)?;
                    } else {
                        let status = reformat_file(path, write_changes)?;
                        // unwrap justification: the channel only fails if it's closed on either
                        // end. The drop() happens below.
                        sender.send((path.clone(), status)).unwrap()
                    }
                }

                // Time to collect the results
                drop(sender);
                // unwrap justification: the reducer code has no exceptions
                let (files_count, files_changed) = reducer.join().unwrap();

                let text = if write_changes {
                    "have been reformatted"
                } else {
                    "would have been reformatted"
                };
                eprintln!("{} / {} {}", files_changed, files_count, text);
                if fail_on_changes && files_changed > 0 {
                    return Err("error: fail on changes".into());
                }
            }
        },
        Operation::Parse { output_format } => {
            let input = read_single_source(&args.src)?;
            let ast = rnix::parse(&input);
            let res = match output_format {
                OutputFormat::Default => {
                    let mut buf = String::new();
                    for error in ast.errors() {
                        writeln!(buf, "error: {}", error).unwrap();
                    }
                    writeln!(buf, "{}", ast.root().dump()).unwrap();
                    buf
                }
                OutputFormat::Json => serde_json::to_string_pretty(&ast.node())?,
            };
            print!("{}", res)
        }
        Operation::Explain => {
            let input = read_stdin_to_string()?;
            let output = nixpkgs_fmt::explain(&input);
            print!("{}", output);
        }
    };

    Ok(())
}

fn read_stdin_to_string() -> Result<String> {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn read_single_source(src: &Src) -> Result<String> {
    let res = match src {
        Src::Stdin => read_stdin_to_string()?,
        Src::Paths(paths) => {
            if paths.len() != 1 {
                return Err("exactly one path required".into());
            }
            fs::read_to_string(&paths[0])?
        }
    };
    Ok(res)
}

fn reformat_dir_in_place(
    dir: &Path,
    write_changes: bool,
    sender: &Sender<FormatResult>,
) -> Result<()> {
    let nix_file_types = {
        let mut builder = ignore::types::TypesBuilder::new();
        builder.add_defaults();
        // unwrap justification: this would be a bug in the code, logic error
        builder.add("nix", "*.nix").unwrap();
        builder.select("nix");
        // unwrap justification: this would be a bug in the code, logic error
        builder.build().unwrap()
    };

    ignore::WalkBuilder::new(dir).types(nix_file_types).threads(8).build_parallel().run(
        move || {
            let s = sender.clone();
            Box::new(move |entry| {
                match reformat_dir_entry(entry, write_changes, &s) {
                    Err(err) => eprintln!("error: {}", err),
                    Ok(()) => {}
                }
                ignore::WalkState::Continue
            })
        },
    );
    Ok(())
}

fn reformat_dir_entry(
    entry: std::result::Result<ignore::DirEntry, ignore::Error>,
    write_changes: bool,
    sender: &Sender<FormatResult>,
) -> Result<()> {
    let path = entry?.into_path();
    if !path.is_file() {
        return Ok(());
    }
    let status = reformat_file(&path, write_changes)?;
    sender.send((path, status))?;
    Ok(())
}

fn reformat_file(file: &Path, write_changes: bool) -> Result<FormatStatus> {
    let input = fs::read_to_string(file)?;
    let output = nixpkgs_fmt::reformat_string(&input);
    if input != output {
        if write_changes {
            fs::write(file, &output)?;
        }
        return Ok(FormatStatus::Change);
    }
    Ok(FormatStatus::NoChange)
}
