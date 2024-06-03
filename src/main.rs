#![allow(
    dead_code,
//     clippy::upper_case_acronyms,
//     unused_variables,
//     clippy::unwrap_used
)]

use std::{cell::RefCell, process::ExitCode};

use clap::Parser;

pub mod format;

use crate::format::prototype::PrototypeDoc;

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Base version of the docs to use
    #[clap(short, long, value_parser)]
    pub source: String,

    /// Target version of the docs to compare against
    /// If not specified, the latest version is used
    #[clap(short, long, value_parser, default_value = "latest")]
    pub target: String,

    /// Diff descriptions
    #[clap(short, long, action)]
    pub descriptions: bool,

    /// Diff examples
    #[clap(short, long, action)]
    pub examples: bool,

    /// Full diff (descriptions, examples, ordering, images, lists)
    #[clap(short, long, action)]
    pub full: bool,
}

thread_local! {static CLI: RefCell<Cli> = RefCell::new(Cli::parse());}

fn main() -> ExitCode {
    let cli = CLI.with_borrow(std::clone::Clone::clone);

    let source = match PrototypeDoc::fetch(&cli.source) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Failed to fetch source docs: {e}");
            return ExitCode::FAILURE;
        }
    };

    let target = match PrototypeDoc::fetch(&cli.target) {
        Ok(target) => target,
        Err(e) => {
            eprintln!("Failed to fetch target docs: {e}");
            return ExitCode::FAILURE;
        }
    };

    let diff = source.diff(&target);

    match serde_json::to_string_pretty(&diff) {
        Ok(d) => println!("{d}"),
        Err(e) => {
            eprintln!("Failed to serialize diff: {e}");
            return ExitCode::FAILURE;
        }
    }

    println!();
    proto_info(&source);
    println!();
    proto_info(&target);
    println!();

    println!("=> {} prototypes changed", diff.prototypes.len());
    println!("=> {} types changed", diff.types.len());

    ExitCode::SUCCESS
}

enum Docs {
    Prototypes,
    Runtime,
}

impl std::fmt::Display for Docs {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Prototypes => write!(f, "prototype"),
            Self::Runtime => write!(f, "runtime"),
        }
    }
}

impl Docs {
    fn get(&self, version: &str) -> Result<Box<[u8]>, reqwest::Error> {
        let res = reqwest::blocking::get(format!(
            "https://lua-api.factorio.com/{version}/{self}-api.json"
        ))?
        .bytes()?;

        Ok((*res).into())
    }
}

fn proto_info(proto: &format::prototype::PrototypeDoc) {
    println!(
        "{:?} @ {}: {:?}",
        proto.common.application, proto.common.application_version, proto.common.stage
    );
    println!("  {} prototypes", proto.prototypes.len());
    println!("  {} types", proto.types.len());
}
