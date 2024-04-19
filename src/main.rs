#![allow(
    dead_code,
    clippy::upper_case_acronyms,
    unused_variables,
    clippy::unwrap_used
)]

use std::cell::RefCell;

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
    #[clap(short, long, value_parser)]
    pub target: Option<String>,

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

fn main() {
    let cli = CLI.with_borrow(std::clone::Clone::clone);
    let source = reqwest::blocking::get(format!(
        "https://lua-api.factorio.com/{}/prototype-api.json",
        cli.source
    ))
    .unwrap()
    .bytes()
    .unwrap();

    let target = reqwest::blocking::get(format!(
        "https://lua-api.factorio.com/{}/prototype-api.json",
        cli.target.unwrap_or_else(|| "latest".to_owned())
    ))
    .unwrap()
    .bytes()
    .unwrap();

    let source: PrototypeDoc = serde_json::from_slice(&source).unwrap();
    let target: PrototypeDoc = serde_json::from_slice(&target).unwrap();

    // calculate the diff
    let diff = source.diff(&target);

    // print the diff
    // diff.diff_print(&source, &target, 0, "");

    let d = serde_json::to_string_pretty(&diff).unwrap();
    println!("{d}");

    println!();
    proto_info(&source);
    println!();
    proto_info(&target);
    println!();

    println!("=> {} prototypes changed", diff.prototypes.len());
    println!("=> {} types changed", diff.types.len());
}

fn proto_info(proto: &format::prototype::PrototypeDoc) {
    println!(
        "{:?} @ {}: {:?}",
        proto.common.application, proto.common.application_version, proto.common.stage
    );
    println!("  {} prototypes", proto.prototypes.len());
    println!("  {} types", proto.types.len());
}
