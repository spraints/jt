use cli::JournalTimeCli::*;
use errs::CheckStatus;

use crate::journal::Journal;

mod cli;
mod errs;
mod journal;

fn main() {
    let args = cli::parse_args();
    match args {
        Today => edit_today(),
        JustPush => tmp_push(),
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

fn tmp_push() -> errs::Result<()> {
    // todo - don't push to a hard coded URL like this. Instead, use 'pt config' to set the remote
    // on the Journal so that 'jt today' can just 'git push'.
    let push_url = "git@github.com:spraints/work-journal.git";

    println!("push to {push_url}...");
    Journal::new()?
        .git_cmd()
        .arg("push")
        .arg(push_url)
        .arg("refs/heads/main:refs/heads/main")
        .status()?
        .check()?;

    Ok(())
}
