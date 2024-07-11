use clap::{Args, Parser, Subcommand, ValueEnum};

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
    Add(AddBookArgs),

    /// Edit notes for a book.
    Notes(BookNoteArgs),
}

#[derive(Args, Debug)]
pub struct AddBookArgs {
    /// Author(s) of the book. May be specified multiple times.
    #[arg(short, long)]
    pub author: Vec<String>,

    /// ISBN for the book, if known.
    #[arg(short, long)]
    pub isbn: Option<String>,

    /// How am I reading this?
    #[arg(short, long)]
    pub media: BookMedia,

    /// Explicit slug, otherwise the title will be used to generate a slug.
    #[arg(short, long)]
    pub slug: Option<String>,

    pub title: Vec<String>,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum BookMedia {
    Kindle,
    HardCopy,
}

#[derive(Args, Debug)]
pub struct BookNoteArgs {
    /// A unique identifier for the book, otherwise the most recently edited notes.
    pub slug: Option<String>,
}
