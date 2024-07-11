use clap::{Args, Parser, Subcommand};

pub fn parse_args() -> JournalTimeCli {
    JournalTimeCli::parse()
}

#[derive(Parser, Debug)]
#[command(name = "jt")]
#[command(bin_name = "jt")]
pub enum JournalTimeCli {
    /// Edit today's journal entry.
    Today,

    /// Subcommands for books.
    Book(BookArgs),

    /// Hack to push my journal to github so I have a backup of it.
    ///
    /// Eventually, this will be replaced with 'pt config' and an automatic push at the end of 'pt
    /// today'.
    JustPush,

    /// Hack to fetch my journal from github to a new machine.
    ///
    /// Eventually, this will be replaced with 'pt config' and an automatic push at the end of 'pt
    /// today'.
    JustFetch,

    /// Hack to show my journal with 'gh repo view'.
    JustView,

    /// Show the path to the journal repo.
    Path,

    View,
    Sync,
    Recent,
    Config,
    Find,
}

#[derive(Args, Debug)]
pub struct BookArgs {
    #[clap(subcommand)]
    pub cmd: BookCmd,
}

#[derive(Subcommand, Debug)]
pub enum BookCmd {
    /// List books that I'm making notes about.
    List,

    /// Add a new book.
    Add,

    /// Edit notes for a book.
    Notes,
}
