use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(super) struct Args {
    /// Modules to use.
    // https://github.com/TeXitoi/structopt/issues/84#issuecomment-1443764459
    #[arg(value_enum, required = true, num_args = 1..)]
    modules: Vec<Module>,
}

impl Args {
    pub fn init() -> Self {
        Self::parse()
    }

    pub fn modules(&self) -> &Vec<Module> {
        &self.modules
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(super) enum Module {
    /// German language module.
    #[cfg(feature = "de")]
    German,
    /// Symbols module.
    #[cfg(feature = "symbols")]
    Symbols,
}
