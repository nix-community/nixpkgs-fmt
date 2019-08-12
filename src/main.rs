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
    srcs: Vec<Src>,
    operation: Operation,
}

#[derive(Debug)]
enum Operation {
    Fmt { in_place: bool },
    Parse,
}

#[derive(Debug)]
enum Src {
    Stdin,
    File(PathBuf),
}

fn parse_args() -> Result<Args> {
    let matches = App::new("nixpkgs-fmt")
        .version("0.1")
        .about("Format Nix code")
        .arg(Arg::with_name("srcs").value_name("FILE").multiple(true).help("File to reformat"))
        .arg(
            Arg::with_name("in-place")
                .long("--in-place")
                .short("-i")
                .conflicts_with("parse")
                .help("Overwrite FILE in place"),
        )
        .arg(
            Arg::with_name("parse")
                .long("parse")
                .conflicts_with("in-place")
                .help("Show syntax tree instead of reformatting"),
        )
        .get_matches_safe()?;

    let in_place = matches.is_present("in-place");
    let srcs = match matches.values_of("srcs") {
        None => vec![Src::Stdin], // default to reading from stdin
        Some(srcs) => srcs.map(|src| Src::File(PathBuf::from(src))).collect(),
    };
    let operation = if matches.is_present("parse") {
        Operation::Parse
    } else {
        Operation::Fmt { in_place: in_place }
    };

    Ok(Args { operation, srcs })
}

fn read_input(src: &Src) -> Result<String> {
    match &src {
        Src::Stdin => {
            let mut buf = String::new();
            stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
        Src::File(path) => {
            let buf = fs::read_to_string(path)?;
            Ok(buf)
        }
    }
}

fn try_main(args: Args) -> Result<()> {
    match args.operation {
        Operation::Fmt { in_place } => {
            for src in args.srcs {
                let input = read_input(&src)?;
                let output = nixpkgs_fmt::reformat_string(&input);

                // only output if it has changed
                if in_place && input != output {
                    match src {
                        Src::File(path) => fs::write(path, &output)?,
                        Src::Stdin => print!("{}", output),
                    }
                } else {
                    print!("{}", output)
                }
            }
        }
        Operation::Parse => {
            for src in args.srcs {
                let input = read_input(&src)?;
                let ast = rnix::parse(&input);
                let mut buf = String::new();
                for error in ast.root_errors() {
                    writeln!(buf, "error: {}", error).unwrap();
                }
                writeln!(buf, "{}", ast.root().dump()).unwrap();
                print!("{}", buf)
            }
        }
    };

    Ok(())
}
