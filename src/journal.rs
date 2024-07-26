use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{prelude::*, Days};

use crate::books::Books;
use crate::errs::{self, CheckStatus};
use crate::toplevel::{JournalEntity, JournalTopLevel};

#[derive(Clone)]
pub struct Journal {
    repo_path: PathBuf,
}

impl Journal {
    pub fn log_file<P: AsRef<Path>>(name: P) -> errs::Result<PathBuf> {
        let mut log_file = Self::data_dir()?;
        log_file.push("journaltime");
        log_file.push(name);
        Ok(log_file)
    }

    pub fn path() -> errs::Result<PathBuf> {
        let mut repo_path = Self::data_dir()?;
        repo_path.push("journaltime/journal");
        Ok(repo_path)
    }

    pub fn new() -> errs::Result<Self> {
        let mut journal = Self {
            repo_path: Self::path()?,
        };
        journal.ensure_repo()?;

        Ok(journal)
    }

    pub fn git_cmd(&self) -> Command {
        let path = &self.repo_path;
        let mut c = Command::new("git");
        c.current_dir(path);
        c
    }

    pub fn current_week(&self) -> errs::Result<JournalFile<Journal>> {
        let today = Local::now().date_naive();
        let res = JournalFile {
            repo_path: self.repo_path.clone(),
            today,
            j: self.clone(),
        };
        create_dir_all(
            res.path()
                .parent()
                .expect("should be able to peel a dir here"),
        )?;
        Ok(res)
    }

    fn ensure_repo(&mut self) -> errs::Result<()> {
        if !self.repo_path.join(".git/HEAD").exists() {
            Command::new("git")
                .arg("init")
                .arg(&self.repo_path)
                .status()?
                .check()?;
        }
        Ok(())
    }

    fn data_dir() -> Result<PathBuf, &'static str> {
        match dirs::data_local_dir() {
            Some(path) => Ok(path),
            None => match std::env::var("HOME") {
                Ok(home) => {
                    let mut path = PathBuf::from(home);
                    path.push(".data");
                    Ok(path)
                }
                Err(_) => Err("could not determine where journal should be stored"),
            },
        }
    }

    pub fn books(&self) -> Books<Self> {
        Books::new(self.clone())
    }
}

pub struct JournalFile<J: JournalTopLevel> {
    repo_path: PathBuf,
    today: NaiveDate,
    j: J,
}

impl<J: JournalTopLevel> JournalEntity for JournalFile<J> {
    fn path(&self) -> PathBuf {
        self.repo_path.join(self.relative_path())
    }

    fn journal_path(&self) -> PathBuf {
        self.repo_path.clone()
    }

    fn relative_path(&self) -> PathBuf {
        self.start_of_week()
            .format("%Y/%m-%d.md")
            .to_string()
            .into()
    }
}

impl<J: JournalTopLevel> JournalFile<J> {
    pub fn start_of_week(&self) -> NaiveDate {
        self.today - Days::new(self.today.weekday().days_since(Weekday::Mon) as u64)
    }

    pub fn prepare_today(&mut self) -> errs::Result<()> {
        let mut has_week_header = false;
        let week_header = self
            .start_of_week()
            .format("# Week starting %Y-%m-%d")
            .to_string();

        let mut has_day_header = false;
        let day_header = self.today.format("## %Y-%m-%d").to_string();

        let mut ends_with_blank_line = false;

        let mut f = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(self.path())?;

        {
            let mut r = BufReader::new(&mut f);
            let mut line = String::new();
            while r.read_line(&mut line)? > 0 {
                ends_with_blank_line = false;
                match line.trim() {
                    s if s == week_header => has_week_header = true,
                    s if s == day_header => has_day_header = true,
                    "" => ends_with_blank_line = true,
                    _ => (),
                };
                line.clear();
            }
        }

        {
            let mut w = BufWriter::new(&mut f);

            if !has_week_header {
                writeln!(&mut w, "{week_header}")?;
                ends_with_blank_line = false;
            }

            if !has_day_header {
                if !ends_with_blank_line {
                    writeln!(&mut w)?;
                }
                writeln!(&mut w, "{day_header}")?;
            }
        }

        Ok(())
    }

    pub fn commit(&mut self) -> errs::SimpleResult {
        self.j.commit_file(self.relative_path())
    }
}

impl JournalTopLevel for Journal {
    fn path(&self) -> PathBuf {
        self.repo_path.clone()
    }

    fn commit_file(&self, relative_path: impl AsRef<Path>) -> errs::SimpleResult {
        // todo - suppress output maybe, or show commands?
        println!("commit changes to journal repository...");
        self.git_cmd()
            .arg("add")
            .arg(relative_path.as_ref().as_os_str())
            .status()?
            .check()?;

        // TODO - this will exit with an error if there's nothing to commit, but that's something
        // we can ignore here.
        self.git_cmd()
            .arg("commit")
            .arg("-q")
            .arg("-m")
            .arg("edited entry")
            .status()?
            .check()?;

        // todo - if the push doesn't work, add a hint to run 'pt config', which will be able to
        // set up a remote.
        println!("push journal repository...");

        self.git_cmd().arg("push").status()?.check()?;

        Ok(())
    }
}
