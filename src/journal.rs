use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

use chrono::{prelude::*, Days};

use crate::errs;

pub struct Journal {
    repo_path: PathBuf,
}

impl Journal {
    pub fn new() -> errs::Result<Self> {
        let mut repo_path = Self::data_dir()?;
        repo_path.push("journaltime/journal");

        let mut journal = Self { repo_path };
        journal.ensure_repo()?;

        Ok(journal)
    }

    pub fn current_week(&self) -> errs::Result<JournalFile> {
        let today = Local::now().date_naive();
        let res = JournalFile {
            repo_path: self.repo_path.clone(),
            today,
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
                .status()?;
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
}

pub struct JournalFile {
    repo_path: PathBuf,
    today: NaiveDate,
}

impl JournalFile {
    pub fn start_of_week(&self) -> NaiveDate {
        self.today - Days::new(self.today.weekday().days_since(Weekday::Mon) as u64)
    }

    pub fn path(&self) -> PathBuf {
        self.repo_path.join(self.relative_path())
    }

    pub fn relative_path(&self) -> String {
        self.start_of_week().format("%Y/%m-%d.md").to_string()
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

    pub fn commit(&mut self) -> errs::Result<()> {
        println!("commit changes to journal repository...");
        Command::new("git")
            .current_dir(&self.repo_path)
            .arg("add")
            .arg(self.relative_path())
            .status()?;
        Command::new("git")
            .current_dir(&self.repo_path)
            .arg("commit")
            .arg("-q")
            .arg("-m")
            .arg("edited entry")
            .status()?;

        println!("push journal repository...");
        Command::new("git")
            .current_dir(&self.repo_path)
            .arg("push")
            .status()?;

        Ok(())
    }
}
