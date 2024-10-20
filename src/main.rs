use std::{cell::RefCell, path::Path, process::ExitCode};

use anyhow::Result;

use clap::{crate_authors, crate_description, Parser};
use format::{runtime::RuntimeDoc, Doc as _};

pub mod format;

use crate::format::prototype::PrototypeDoc;

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Clone)]
#[clap(author = crate_authors!(), version, about = crate_description!())]
pub struct Cli {
    /// Stage of the docs to use.
    ///
    /// Prototype stage supports format versions 4 and 5.
    /// Runtime stage supports format version 5 only.
    #[clap(value_parser, verbatim_doc_comment)]
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

    /// Read source and target from local files
    #[clap(short, long, action)]
    pub local: bool,
}

thread_local! {static CLI: RefCell<Cli> = RefCell::new(Cli::parse());}
thread_local! {static SRC_INF: RefCell<format::Common> = RefCell::default();}
thread_local! {static TRGT_INF: RefCell<format::Common> = RefCell::default();}

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

    fn get_local(self, path: &Path) -> Result<Box<[u8]>> {
        let res = std::fs::read(path.join(format!("doc-html/{self}-api.json")))?;

        Ok(res.into())
    }

    #[allow(clippy::too_many_lines)]
    pub fn compare(self, source: &str, target: &str) -> Result<()> {
        let (source, target) = if CLI.with_borrow(|c| c.local) {
            (
                self.get_local(Path::new(&source))?,
                self.get_local(Path::new(&target))?,
            )
        } else {
            (self.get(source)?, self.get(target)?)
        };

        let source_info = match serde_json::from_slice::<format::Common>(&source) {
            Ok(s) => s,
            Err(e) => {
                anyhow::bail!("Failed to get common info header from source: {e}");
            }
        };

        SRC_INF.replace(source_info.clone());

        let target_info = match serde_json::from_slice::<format::Common>(&target) {
            Ok(s) => s,
            Err(e) => {
                anyhow::bail!("Failed to get common info header from target: {e}");
            }
        };

        TRGT_INF.replace(target_info.clone());

        let (d, s, t): (
            Box<dyn format::Info>,
            Box<dyn format::Info>,
            Box<dyn format::Info>,
        ) = match self {
            Self::Prototype => {
                if source_info.api_version < 4 {
                    anyhow::bail!(
                        "Source api format is too old! Only api version 4, 5 and 6 are supported"
                    );
                }

                if target_info.api_version < 4 {
                    anyhow::bail!(
                        "Target api format is too old! Only api version 4, 5 and 6 are supported"
                    );
                }

                if source_info.api_version > 6 {
                    anyhow::bail!(
                        "Source api format is too new! Only api version 4, 5 and 6 are supported"
                    );
                }

                if target_info.api_version > 6 {
                    anyhow::bail!(
                        "Target api format is too new! Only api version 4, 5 and 6 are supported"
                    );
                }

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
                if source_info.api_version < 5 {
                    anyhow::bail!(
                        "Source api format is too old! Only api version 5 and 6 are supported"
                    );
                }

                if target_info.api_version < 5 {
                    anyhow::bail!(
                        "Target api format is too old! Only api version 5 and 6 are supported"
                    );
                }

                if source_info.api_version > 6 {
                    anyhow::bail!(
                        "Source api format is too new! Only api version 5 and 6 are supported"
                    );
                }

                if target_info.api_version > 6 {
                    anyhow::bail!(
                        "Target api format is too new! Only api version 5 and 6 are supported"
                    );
                }

                if source_info.api_version > target_info.api_version {
                    anyhow::bail!("Source api format is newer than target api format");
                }

                let source: RuntimeDoc = match serde_json::from_slice(&source) {
                    Ok(s) => s,
                    Err(e) => {
                        anyhow::bail!(
                            "Failed to deserialize source: {e}\n{}",
                            std::str::from_utf8(&source).unwrap_or("[invalid utf-8]")
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
