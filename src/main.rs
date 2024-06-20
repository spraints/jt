use crate::journal::Journal;

mod cli;
mod errs;
mod journal;

fn main() {
    let args = cli::parse_args();
    match args {
        cli::JournalTimeCli::Today => edit_today(),
        args => todo(args),
    }
    .unwrap();
}

fn todo(args: cli::JournalTimeCli) -> errs::Result<()> {
    println!("todo! {args:?}");
    Ok(())
}

fn edit_today() -> errs::Result<()> {
    let journal = Journal::new()?;
    let mut this_week = journal.current_week()?;
    this_week.prepare_today()?;

    let editor = std::env::var("EDITOR")?;
    std::process::Command::new(editor)
        .arg(this_week.path())
        .status()?;

    this_week.commit()
}
