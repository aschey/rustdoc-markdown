use std::io;

use markdown_writer::{HeaderLevel, MarkdownWriter};
use rustdoc_code_formatter::ModuleRepr;

pub fn write<W: io::Write>(module: &ModuleRepr, writer: W) -> io::Result<()> {
    let mut writer = MarkdownWriter::new(writer);

    writer.header(HeaderLevel::One, "Docs")?;
    writer.newlines(2)?;

    writer.header(HeaderLevel::Two, "Functions")?;
    writer.newlines(2)?;
    for function in &module.functions {
        writer.header(HeaderLevel::Three, &function.name)?;
        writer.newlines(2)?;
        writer.fenced_code_block(&function.repr, Some("rust"))?;
        writer.newlines(2)?;
    }

    writer.header(HeaderLevel::Two, "Structs")?;
    writer.newlines(2)?;
    for struct_ in &module.structs {
        writer.header(HeaderLevel::Three, &struct_.name)?;
        writer.newlines(2)?;
        writer.fenced_code_block(&struct_.repr, Some("rust"))?;
        writer.newlines(2)?;
    }

    writer.header(HeaderLevel::Two, "Traits")?;
    writer.newlines(2)?;
    for trait_ in &module.traits {
        writer.header(HeaderLevel::Three, &trait_.name)?;
        writer.newlines(2)?;
        writer.fenced_code_block(&trait_.repr, Some("rust"))?;
        writer.newlines(2)?;
    }

    Ok(())
}
