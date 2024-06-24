use clap::Parser;

pub fn parse_args() -> JournalTimeCli {
    JournalTimeCli::parse()
}

#[derive(Parser, Debug)]
#[command(name = "jt")]
#[command(bin_name = "jt")]
pub enum JournalTimeCli {
    /// Edit today's journal entry.
    Today,

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

    /// Show the path to the journal repo.
    Path,

    View,
    Sync,
    Recent,
    Config,
    Find,
}
