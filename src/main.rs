#[macro_use]
extern crate clap;

use std::{
    fmt::Write,
    fs,
    io::{stdin, Read},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use clap::{App, Arg};
use rnix::types::TypedNode;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
    Fmt,
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
        .version(crate_version!())
        .about("Format Nix code")
        .arg(
            Arg::with_name("srcs")
                .value_name("FILE")
                .multiple(true)
                .help("File to reformat in place. If no file is passed, read from stdin."),
        )
        .arg(Arg::with_name("parse").long("parse").help("Show syntax tree instead of reformatting"))
        .arg(
            Arg::with_name("output-format")
                .long("output-format")
                .requires("parse")
                .possible_values(&["json"])
                .help("Output syntax tree in JSON format"),
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
    } else {
        Operation::Fmt
    };

    Ok(Args { operation, src })
}

fn try_main(args: Args) -> Result<()> {
    match args.operation {
        Operation::Fmt => match &args.src {
            Src::Stdin => {
                let input = read_stdin_to_string()?;
                let output = nixpkgs_fmt::reformat_string(&input);
                print!("{}", output);
            }
            Src::Paths(paths) => {
                for path in paths {
                    if path.is_dir() {
                        reformat_dir_in_place(path)?;
                    } else {
                        reformat_file_in_place(path)?;
                    }
                }
            }
        },
        Operation::Parse { output_format } => {
            let input = read_single_source(&args.src)?;
            let ast = rnix::parse(&input);
            let res = match output_format {
                OutputFormat::Default => {
                    let mut buf = String::new();
                    for error in ast.root_errors() {
                        writeln!(buf, "error: {}", error).unwrap();
                    }
                    writeln!(buf, "{}", ast.root().dump()).unwrap();
                    buf
                }
                OutputFormat::Json => serde_json::to_string_pretty(&ast.node())?,
            };
            print!("{}", res)
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
                Err("exactly one path required")?;
            }
            fs::read_to_string(&paths[0])?
        }
    };
    Ok(res)
}

fn reformat_file_in_place(file: &PathBuf) -> Result<()> {
    let input = fs::read_to_string(file)?;
    let output = nixpkgs_fmt::reformat_string(&input);
    if input != output {
        fs::write(file, &output)?;
    }
    Ok(())
}

fn reformat_dir_in_place(dir: &PathBuf) -> Result<()> {
    let nix_file_type = {
        let mut builder = ignore::types::TypesBuilder::new();
        builder.add_defaults();
        builder.add("nix", "*.nix").unwrap();
        builder.select("nix");
        builder.build().unwrap()
    };
    let has_errors = Arc::new(AtomicBool::new(false));
    ignore::WalkBuilder::new(dir).threads(8).types(nix_file_type).build_parallel().run(|| {
        let has_errors = Arc::clone(&has_errors);
        Box::new(move |entry| match reformat_dir_entry(entry) {
            Ok(()) => ignore::WalkState::Continue,
            Err(err) => {
                has_errors.store(true, Ordering::SeqCst);
                eprintln!("error: {}", err);
                ignore::WalkState::Continue
            }
        })
    });
    if has_errors.load(Ordering::SeqCst) {
        Err("there were errors during directory traversal")?
    }
    Ok(())
}

fn reformat_dir_entry(entry: std::result::Result<ignore::DirEntry, ignore::Error>) -> Result<()> {
    let path = entry?.into_path();
    if !path.is_file() {
        return Ok(());
    }
    reformat_file_in_place(&path)
}
