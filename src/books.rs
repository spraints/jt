use std::fmt::Display;
use std::fs::{create_dir_all, read_dir, read_to_string, DirEntry, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use yaml_front_matter::YamlFrontMatter;

use crate::errs;
use crate::toplevel::{JournalEntity, JournalTopLevel};

pub struct Books<J> {
    j: J,
}

pub struct BookEntry<J> {
    j: J,
    relative_path: String,
    frontmatter: Frontmatter,
}

#[derive(Serialize, Deserialize)]
struct Frontmatter {
    title: String,
    authors: Vec<String>,
    isbn: Option<String>,
    media: Option<String>,
}

pub struct AddBookArgs {
    pub title: String,
    pub authors: Vec<String>,
    pub isbn: Option<String>,
    pub media: String,
    pub slug: Option<String>,
}

impl<J: JournalTopLevel> Books<J> {
    pub fn new(j: J) -> Self {
        Self { j }
    }

    pub fn add(&self, info: AddBookArgs) -> errs::Result<BookEntry<J>> {
        let slug = info.slug.unwrap_or_else(|| make_slug(&info.title));
        let frontmatter = Frontmatter {
            title: info.title,
            authors: info.authors,
            isbn: info.isbn,
            media: Some(info.media),
        };

        let book_dir = self.j.path().join("books");
        create_dir_all(book_dir)?;

        let relative_path = format!("books/{slug}.md");

        let book_path = self.j.path().join(&relative_path);
        let f = File::create_new(book_path)?;
        writeln!(&f, "---")?;
        serde_yaml::to_writer(&f, &frontmatter)?;
        writeln!(&f, "---")?;
        writeln!(&f, "# {}", frontmatter.title)?;

        Ok(BookEntry {
            j: self.j.clone(),
            relative_path,
            frontmatter,
        })
    }

    pub fn get(&self, slug: &str) -> errs::Result<Option<BookEntry<J>>> {
        for book in self.iter()? {
            if book.slug()? == slug {
                return Ok(Some(book));
            }
        }
        Ok(None)
    }

    pub fn most_recent_notes(&self) -> errs::Result<Option<BookEntry<J>>> {
        fn mt<J: JournalTopLevel>(e: BookEntry<J>) -> (BookEntry<J>, SystemTime) {
            let modified = e
                .path()
                .metadata()
                .and_then(|i| i.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            (e, modified)
        }
        Ok(self
            .iter()?
            .map(mt)
            .reduce(|a, b| if a.1 > b.1 { a } else { b })
            .map(|(e, _)| e))
    }

    pub fn iter(&self) -> errs::Result<std::vec::IntoIter<BookEntry<J>>> {
        Ok(self.read_book_list()?.into_iter())
    }

    fn read_book_list(&self) -> errs::Result<Vec<BookEntry<J>>> {
        match read_dir(self.j.path().join("books")) {
            Err(_) => Ok(Vec::new()),
            Ok(dir) => self.parse_book_entries(dir),
        }
    }

    fn parse_book_entries<I: Iterator<Item = std::io::Result<DirEntry>>>(
        &self,
        dir: I,
    ) -> errs::Result<Vec<BookEntry<J>>> {
        let mut res = Vec::new();
        for e in dir {
            let e = e?;
            // assume we're in <journal_path>/books.
            let relative_path = format!(
                "books/{}",
                e.file_name()
                    .to_str()
                    .ok_or_else(|| format!("illegal unicode in {:?}", e.file_name()))?
            );
            let content = read_to_string(self.j.path().join(&relative_path))?;
            let frontmatter = YamlFrontMatter::parse(&content)?.metadata;
            res.push(BookEntry {
                j: self.j.clone(),
                relative_path,
                frontmatter,
            });
        }
        Ok(res)
    }
}

impl<J> Display for BookEntry<J> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slug = self.slug().unwrap_or_else(|_| "????".to_string());
        write!(f, "[{}] {}", slug, self.frontmatter.title)?;
        for author in &self.frontmatter.authors {
            write!(f, "  by {author}")?;
        }
        Ok(())
    }
}

impl<J: JournalTopLevel> JournalEntity for BookEntry<J> {
    fn path(&self) -> PathBuf {
        self.j.path().join(&self.relative_path)
    }

    fn journal_path(&self) -> PathBuf {
        self.j.path()
    }

    fn relative_path(&self) -> PathBuf {
        self.relative_path.to_owned().into()
    }
}

impl<J: JournalTopLevel> BookEntry<J> {
    pub fn commit(&self) -> errs::SimpleResult {
        self.j.commit_file(&self.relative_path)
    }
}

impl<J> BookEntry<J> {
    fn slug(&self) -> errs::Result<String> {
        Ok(PathBuf::from(&self.relative_path)
            .file_stem()
            .ok_or_else(|| format!("invalid book file name: {}", self.relative_path))?
            .to_str()
            .ok_or_else(|| format!("non-unicode filename: {}", self.relative_path))?
            .to_string())
    }
}

fn make_slug(s: &str) -> String {
    s.chars().map(slug_char).collect()
}

fn slug_char(c: char) -> char {
    if c.is_alphabetic() {
        c.to_ascii_lowercase()
    } else {
        '-'
    }
}
