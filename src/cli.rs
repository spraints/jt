use clap::{Parser, Subcommand};

pub fn parse_args() -> JournalTimeCli {
    JournalTimeCli::parse()
}

#[derive(Parser, Debug)]
#[command(name = "jt")]
#[command(bin_name = "jt")]
pub enum JournalTimeCli {
    Today(TodayArgs),
    View(ViewArgs),
    Sync(SyncArgs),

    #[command(subcommand)]
    Slack(AppSubcommand),

    #[command(subcommand)]
    GitHub(AppSubcommand), // todo - add "gh" shorthand.
}

#[derive(Subcommand, Debug)]
pub enum AppSubcommand {
    Recent,
    Config,
}

#[derive(Parser, Debug)]
pub struct TodayArgs {}

#[derive(Parser, Debug)]
pub struct ViewArgs {}

#[derive(Parser, Debug)]
pub struct SyncArgs {}
