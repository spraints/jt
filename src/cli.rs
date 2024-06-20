use clap::Parser;

pub fn parse_args() -> JournalTimeCli {
    JournalTimeCli::parse()
}

#[derive(Parser, Debug)]
#[command(name = "jt")]
#[command(bin_name = "jt")]
pub enum JournalTimeCli {
    Today,
    View,
    Sync,
    Recent,
    Config,
    Find,
}
