use cli::JournalTimeCli::*;
use errs::CheckStatus;

use crate::journal::Journal;

mod cli;
mod errs;
mod journal;

// todo - don't push to a hard coded URL like this. Instead, use 'pt config' to set the remote on
// the Journal so that 'jt today' can just 'git push'.
//
// In the meantime, if you're not spraints, this is where you should configure where your repo will
// go when you run 'jt just-push' or 'jt just-fetch'.
const REMOTE_REPO_URL: &'static str = "git@github.com:spraints/work-journal.git";

fn main() {
    let args = cli::parse_args();
    match args {
        Today => edit_today(),
        JustPush => tmp_push(),
        JustFetch => tmp_fetch(),
        Path => show_path(),
        args => todo(args),
    }
    .unwrap();
}

fn todo(args: cli::JournalTimeCli) -> errs::Result<()> {
    println!("todo! {args:?}");
    Ok(())
}

fn show_path() -> errs::Result<()> {
    println!("{}", Journal::path()?.display());
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
    println!("push to {REMOTE_REPO_URL}...");
    Journal::new()?
        .git_cmd()
        .arg("push")
        .arg(REMOTE_REPO_URL)
        .arg("refs/heads/main:refs/heads/main")
        .status()?
        .check()?;

    Ok(())
}

fn tmp_fetch() -> errs::Result<()> {
    let j = Journal::new()?;

    println!("fetch to {REMOTE_REPO_URL}...");
    j.git_cmd()
        .arg("fetch")
        .arg(REMOTE_REPO_URL)
        .arg("+refs/heads/*:refs/just-fetch/*")
        .status()?
        .check()?;

    // Collect output because the user doesn't care about this one.
    let rev_parse_output = j.git_cmd().arg("rev-parse").arg("HEAD").output()?;
    if rev_parse_output.status.success() {
        println!("Merging main from remote...");
        j.git_cmd()
            .arg("merge")
            .arg("refs/just-fetch/main")
            .status()?
            .check()?;
    } else {
        println!("Checking out main...");
        j.git_cmd()
            .arg("checkout")
            .arg("-b")
            .arg("main")
            .arg("refs/just-fetch/main")
            .status()?
            .check()?;
    }

    Ok(())
}
