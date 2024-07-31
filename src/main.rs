#![allow(
    dead_code,
//     clippy::upper_case_acronyms,
//     unused_variables,
//     clippy::unwrap_used
)]

use std::{cell::RefCell, process::ExitCode};

use anyhow::Result;

use clap::Parser;
use format::{runtime::RuntimeDoc, Doc as _};

pub mod format;

use crate::format::prototype::PrototypeDoc;

#[derive(Parser, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(value_parser)]
    pub stage: Docs,

    /// Base version of the docs to use
    #[clap(value_parser)]
    pub source: String,

    /// Target version of the docs to compare against
    /// If not specified, the latest version is used
    #[clap(value_parser, default_value = "latest")]
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

    if let Err(e) = cli.stage.compare(&cli.source, &cli.target) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    };

    ExitCode::SUCCESS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Docs {
    Prototype,
    Runtime,
}

impl clap::ValueEnum for Docs {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Prototype, Self::Runtime]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Prototype => Some(clap::builder::PossibleValue::new("prototype")),
            Self::Runtime => Some(clap::builder::PossibleValue::new("runtime")),
        }
    }
}

impl std::fmt::Display for Docs {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Prototype => write!(f, "prototype"),
            Self::Runtime => write!(f, "runtime"),
        }
    }
}

impl Docs {
    fn get(self, version: &str) -> Result<Box<[u8]>> {
        let res = reqwest::blocking::get(format!(
            "https://lua-api.factorio.com/{version}/{self}-api.json"
        ))?
        .bytes()?;

        Ok((*res).into())
    }

    pub fn compare(self, source: &str, target: &str) -> Result<()> {
        let source = self.get(source)?;
        let target = self.get(target)?;

        let (d, s, t): (
            Box<dyn format::Info>,
            Box<dyn format::Info>,
            Box<dyn format::Info>,
        ) = match self {
            Self::Prototype => {
                let source: PrototypeDoc = match serde_json::from_slice(&source) {
                    Ok(s) => s,
                    Err(e) => {
                        anyhow::bail!("Failed to deserialize source: {e}");
                    }
                };
                let target: PrototypeDoc = match serde_json::from_slice(&target) {
                    Ok(t) => t,
                    Err(e) => {
                        anyhow::bail!("Failed to deserialize target: {e}");
                    }
                };

                let diff = source.diff(&target);

                match serde_json::to_string_pretty(&diff) {
                    Ok(d) => println!("{d}"),
                    Err(e) => {
                        anyhow::bail!("Failed to serialize diff: {e}");
                    }
                }

                (Box::new(diff), Box::new(source), Box::new(target))
            }
            Self::Runtime => {
                let source: RuntimeDoc = match serde_json::from_slice(&source) {
                    Ok(s) => s,
                    Err(e) => {
                        anyhow::bail!(
                            "Failed to deserialize source: {e}\n{}",
                            std::str::from_utf8(&source).unwrap()
                        );
                    }
                };
                let target: RuntimeDoc = match serde_json::from_slice(&target) {
                    Ok(s) => s,
                    Err(e) => {
                        anyhow::bail!("Failed to deserialize target: {e}");
                    }
                };

                let diff = source.diff(&target);

                match serde_json::to_string_pretty(&diff) {
                    Ok(d) => println!("{d}"),
                    Err(e) => {
                        anyhow::bail!("Failed to serialize diff: {e}");
                    }
                }

                (Box::new(diff), Box::new(source), Box::new(target))
            }
        };

        s.print_info();
        eprintln!();
        t.print_info();
        eprintln!();
        d.print_info();

        Ok(())
    }
}
