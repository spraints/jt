use std::{io::Write, path::PathBuf};

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
    let res = match args {
        Today => edit_today(),
        JustPush => tmp_push(),
        JustFetch => tmp_fetch(),
        JustView => tmp_view(),
        Path => show_path(),
        args => todo(args),
    };
    if let Err(e) = res {
        eprintln!("error: {e:?}");
    }
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

    let log_file = Journal::log_file("today.log")?;
    let journal_file = this_week.path();
    std::thread::spawn(move || {
        tmp_inotify(log_file, journal_file);
    });
    //std::thread::sleep(std::time::Duration::from_secs(3));

    // TODO - run the editor concurrently with a filesystem watcher that commits after every save.
    let editor = std::env::var("EDITOR")?;
    std::process::Command::new(editor)
        .arg(this_week.path())
        .status()?;

    if let Err(e) = this_week.commit() {
        eprintln!("failed to sync with upstream: {e:?}");
    }

    Ok(())
}

fn tmp_inotify(log_file: PathBuf, journal_file: PathBuf) {
    eprintln!("sup lets see {log_file:?} // {journal_file:?}");

    use inotify::{Inotify, WatchMask};
    use std::fs::OpenOptions;
    use std::time::Instant;

    fn go<W: Write>(mut lf: W, target: PathBuf) -> errs::Result<()> {
        let start = Instant::now();
        let mut inotify = Inotify::init()?;
        eprintln!("watching {target:?}...");
        writeln!(lf, "tmp_inotify: watching {target:?}")?;
        inotify.watches().add(target, WatchMask::ALL_EVENTS)?;
        let mut buffer = [0u8; 4096];
        loop {
            let events = inotify.read_events_blocking(&mut buffer)?;
            let t = Instant::now().duration_since(start);
            writeln!(lf, "tmp_inotify: [{t:?}] found some events!")?;
            for event in events {
                writeln!(lf, "tmp_inotify: [{t:?}] {event:?}")?;
            }
        }
    }

    match OpenOptions::new().create(true).append(true).open(&log_file) {
        Ok(mut f) => match go(&mut f, journal_file) {
            Err(e) => writeln!(f, "tmp_inotify: fatal: {e:?}").unwrap(),
            Ok(()) => eprintln!("tmp_inotify: go finished"),
        },
        Err(e) => eprintln!("{log_file:?}: {e:?}"),
    };

    eprintln!("tmp_inotify: DONE!");
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

fn tmp_view() -> errs::Result<()> {
    std::process::Command::new("gh")
        .arg("repo")
        .arg("view")
        .arg(REMOTE_REPO_URL)
        .status()?
        .check()?;

    Ok(())
}
