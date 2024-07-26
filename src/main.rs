use std::io::Write;
use std::path::PathBuf;

use cli::JournalTimeCli::*;
use errs::CheckStatus;
use toplevel::JournalEntity;

use crate::{books::AddBookArgs, journal::Journal};

mod books;
mod cli;
mod errs;
mod journal;
mod toplevel;

// todo - don't push to a hard coded URL like this. Instead, use 'pt config' to set the remote on
// the Journal so that 'jt today' can just 'git push'.
//
// In the meantime, if you're not spraints, this is where you should configure where your repo will
// go when you run 'jt just-push' or 'jt just-fetch'.
const REMOTE_REPO_URL: &str = "git@github.com:spraints/work-journal.git";

fn main() {
    let args = cli::parse_args();
    let res = match args {
        Today => edit_today(),
        Book(args) => book_main(args),
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

fn book_main(args: cli::BookArgs) -> errs::SimpleResult {
    let cli::BookArgs { cmd } = args;
    match cmd {
        cli::BookCmd::List => list_books(),
        cli::BookCmd::Add(args) => add_book(args),
        cli::BookCmd::Notes(args) => edit_book_notes(args),
    }
}

fn list_books() -> errs::SimpleResult {
    let journal = Journal::new()?;
    for book in journal.books().iter()? {
        println!("{book}");
    }
    Ok(())
}

fn add_book(args: cli::AddBookArgs) -> errs::SimpleResult {
    let cli::AddBookArgs {
        author,
        isbn,
        media,
        slug,
        title,
    } = args;
    let media = match media {
        cli::BookMedia::Kindle => "kindle",
        cli::BookMedia::HardCopy => "hard copy",
    }
    .to_string();

    let book = Journal::new()?.books().add(AddBookArgs {
        authors: author,
        isbn,
        media: media.to_string(),
        slug,
        title: title.join(" "),
    })?;

    println!("added:");
    println!("{book}");
    Ok(())
}

fn edit_book_notes(args: cli::BookNoteArgs) -> errs::SimpleResult {
    let cli::BookNoteArgs { slug } = args;
    let books = Journal::new()?.books();
    let entry = match slug {
        None => books
            .most_recent_notes()?
            .ok_or("no book notes found, maybe add a new book?")?,
        Some(slug) => books
            .get(&slug)?
            .ok_or_else(|| format!("no book like {slug} is in the journal"))?,
    };
    run_editor(&entry)?;
    entry.commit()?;
    Ok(())
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

    run_editor(&this_week)?;

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

fn run_editor<J: JournalEntity>(entity: &J) -> errs::SimpleResult {
    // TODO - finish the filesystem watcher so that it does a 'git commit' on each write.
    let log_file = Journal::log_file("inotify.log")?;
    let target = entity.path();
    std::thread::spawn(move || {
        tmp_inotify(log_file, target);
    });

    let editor = std::env::var("EDITOR")?;
    std::process::Command::new(editor)
        .current_dir(entity.journal_path())
        .arg(entity.relative_path())
        .status()?;
    Ok(())
}
