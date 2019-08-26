#[macro_use]
extern crate clap;

use std::{
    fmt::Write,
    fs,
    io::{stdin, Read},
    path::PathBuf,
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
    Parse,
}

#[derive(Debug)]
enum Src {
    Stdin,
    Paths(Vec<PathBuf>),
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
        .arg(
            Arg::with_name("parse")
                .long("parse")
                .conflicts_with("in-place")
                .help("Show syntax tree instead of reformatting"),
        )
        .get_matches_safe()?;

    let src = match matches.values_of("srcs") {
        None => Src::Stdin, // default to reading from stdin
        Some(srcs) => Src::Paths(srcs.map(PathBuf::from).collect()),
    };
    let operation = if matches.is_present("parse") { Operation::Parse } else { Operation::Fmt };

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
                    let input = fs::read_to_string(path)?;
                    let output = nixpkgs_fmt::reformat_string(&input);
                    if input != output {
                        fs::write(path, &output)?
                    }
                }
            }
        },
        Operation::Parse => {
            let input = read_single_source(&args.src)?;
            let ast = rnix::parse(&input);
            let mut buf = String::new();
            for error in ast.root_errors() {
                writeln!(buf, "error: {}", error).unwrap();
            }
            writeln!(buf, "{}", ast.root().dump()).unwrap();
            print!("{}", buf)
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
