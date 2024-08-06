use std::io;
use std::sync::LazyLock;

use linkify::LinkFinder;
use regex::Regex;

static SPECIAL_CHAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?<x>[\\\\`*_{}\\[\\]()<>#+\-!~])").unwrap());

pub struct MarkdownWriter<T> {
    writer: T,
}

pub enum HeaderLevel {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

impl<T> MarkdownWriter<T>
where
    T: io::Write,
{
    pub fn new(writer: T) -> Self {
        Self { writer }
    }

    pub fn bold(&mut self, text: &str) -> io::Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        write!(self.writer, "**{text}**")
    }

    pub fn header(&mut self, level: HeaderLevel, text: &str) -> io::Result<()> {
        let header_marker = "#".repeat(level as usize);
        write!(self.writer, "{header_marker} {text}")
    }

    pub fn link(&mut self, text: &str, href: &str) -> io::Result<()> {
        if text.is_empty() {
            return Ok(());
        }
        if href.is_empty() {
            return self.write_escaped(text);
        }
        write!(self.writer, "[")?;
        self.write_escaped(text)?;
        write!(self.writer, "]")?;
        write!(self.writer, "{}", href)
    }

    pub fn legacy_code_block(&mut self, code: &str) -> io::Result<()> {
        let code = code.trim();
        let code = code.replace("\n", "\n\t");
        write!(self.writer, "{}", code)
    }

    pub fn fenced_code_block(&mut self, code: &str, language: Option<&str>) -> io::Result<()> {
        write!(
            self.writer,
            "```{}\n{code}\n```",
            language.unwrap_or_default()
        )
    }

    pub fn anchor(&mut self, anchor: &str) -> io::Result<()> {
        write!(
            self.writer,
            "<a name=\"{}\"></a>",
            urlencoding::encode(anchor)
        )
    }

    pub fn anchor_header(
        &mut self,
        level: HeaderLevel,
        text: &str,
        anchor: &str,
    ) -> io::Result<()> {
        self.anchor(anchor)?;
        self.newline()?;
        self.header(level, text)
    }

    pub fn newline(&mut self) -> io::Result<()> {
        writeln!(self.writer)
    }

    pub fn newlines(&mut self, count: usize) -> io::Result<()> {
        let n = "\n".repeat(count);
        write!(self.writer, "{n}")
    }

    fn write_escaped(&mut self, text: &str) -> io::Result<()> {
        let link_finder = LinkFinder::new();
        let mut pos = 0;
        for link in link_finder.links(text) {
            self.write_escaped_raw(&text[pos..link.start()])?;
            write!(self.writer, "{}", &text[link.start()..link.end()])?;
            pos = link.end();
        }
        if pos < text.len() {
            write!(self.writer, "{}", &text[pos..])?;
        }
        Ok(())
    }

    fn write_escaped_raw(&mut self, text: &str) -> io::Result<()> {
        write!(self.writer, "{}", SPECIAL_CHAR_RE.replace_all(text, r"\$x"))
    }
}

impl<W: io::Write> io::Write for MarkdownWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
