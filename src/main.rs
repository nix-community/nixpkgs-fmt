use std::{
    fs,
    io::{stdin, Read},
    path::PathBuf,
};

use clap::{App, Arg};

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
    dst: Dst,
}

#[derive(Debug)]
enum Src {
    Stdin,
    File(PathBuf),
}

#[derive(Debug)]
enum Dst {
    Stdout,
    File(PathBuf),
}

fn parse_args() -> Result<Args> {
    let matches = App::new("nix-fmt")
        .version("0.1")
        .about("Format Nix code")
        .arg(Arg::with_name("src").value_name("FILE").help("File to reformat"))
        .arg(
            Arg::with_name("in-place")
                .long("--in-place")
                .short("-i")
                .requires("src")
                .conflicts_with("dst")
                .help("Overwrite FILE in place"),
        )
        .arg(
            Arg::with_name("dst")
                .long("output")
                .short("o")
                .takes_value(true)
                .value_name("file")
                .help("Place the output into <file>"),
        )
        .get_matches_safe()?;

    let src_path = matches.value_of("src").map(PathBuf::from);
    let src = src_path.clone().map_or(Src::Stdin, Src::File);
    let dst = if matches.is_present("in-place") {
        Dst::File(src_path.unwrap())
    } else {
        matches
            .value_of("dst")
            .map(PathBuf::from)
            .map_or(Dst::Stdout, Dst::File)
    };

    Ok(Args { src, dst })
}

fn try_main(args: Args) -> Result<()> {
    let input = match &args.src {
        Src::Stdin => {
            let mut buf = String::new();
            stdin().read_to_string(&mut buf)?;
            buf
        }
        Src::File(path) => fs::read_to_string(path)?,
    };

    let res = nix_fmt::reformat_string(&input);

    match &args.dst {
        Dst::Stdout => print!("{}", res),
        //TODO: use atomic replace instead of plain write
        Dst::File(path) => fs::write(path, &res)?,
    }

    Ok(())
}
