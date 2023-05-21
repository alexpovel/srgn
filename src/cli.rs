use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(super) struct Args {
    /// Modules to use.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(super) enum Module {
    /// The German language module.
    #[cfg(feature = "de")]
    German,
    /// The symbols module.
    #[cfg(feature = "symbols")]
    Symbols,
}
