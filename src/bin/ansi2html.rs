extern crate ansi_shim;

// TODO: Provide finalize method to check for errors when writing html footer

use ansi_shim::{Terminal,Shim,Color,Style};
use std::io::{self, Write};
use std::env::args;
use std::fs::File;
use std::collections::BTreeSet;

struct HtmlWriter<W: Write> {
    writer: W,
    fg_color: Color,
    bg_color: Color,
    bold: bool,
    styles: BTreeSet<Style>,
}

const HEADER: &str = "\
<!DOCTYPE html>
<html>
<head>
    <title>ansi2html</title>
    <meta charset='utf-8'/>
    <style>
        html {
            background: #212121;
            color: #FFFFFF;
        }
    </style>
</head>
<body>

<pre>
";

impl<W: Write> HtmlWriter<W> {
    pub fn new(writer: W) -> io::Result<Self> {
        let mut writer = HtmlWriter {
            writer,
            fg_color: Color::White,
            bg_color: Color::Black,
            bold: false,
            styles: BTreeSet::new(),
        };
        writer.write_all(HEADER.as_bytes())?;
        writer.open_span()?;

        Ok(writer)
    }

    fn open_span(&mut self) -> io::Result<()> {
        let fg = if self.bold { self.fg_color.bright() } else { self.fg_color };
        let fg = ansi_color_to_html(fg);
        let bg = ansi_color_to_html(self.bg_color);

        write!(self.writer, "<span style='color: {fg}; background: {bg}; {style}'>",
            fg = fg,
            bg = bg,
            style = ansi_style_to_html(&self.styles),
        )
    }

    fn close_span(&mut self) -> io::Result<()> {
        self.writer.write_all("</span>".as_bytes())
    }

    fn reopen_span(&mut self) -> io::Result<()> {
        self.close_span()?;
        self.open_span()
    }
}

impl<W: Write> Write for HtmlWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> Terminal for HtmlWriter<W> {
    fn print(&mut self, ch: char) -> io::Result<()> {
        match ch {
            '&' => write!(self.writer, "&amp;"),
            '<' => write!(self.writer, "&lt;"),
            '>' => write!(self.writer, "&gt;"),
            '"' => write!(self.writer, "&#x27;"),
            '\'' => write!(self.writer, "&#x2F;"),
            _ => write!(self.writer, "{}", ch),
        }
    }

    fn set_fg_color(&mut self, color: Color) -> io::Result<()> {
        self.fg_color = color;
        self.reopen_span()
    }

    fn set_bg_color(&mut self, color: Color) -> io::Result<()> {
        self.bg_color = color;
        self.reopen_span()
    }

    fn reset_style(&mut self) -> io::Result<()> {
        self.styles.clear();
        self.fg_color = Color::White;
        self.bg_color = Color::Black;
        self.bold = false;
        self.reopen_span()
    }

    fn add_style(&mut self, style: Style) -> io::Result<()> {
        match style {
            Style::Bold => {
                self.bold = true;
                return Ok(())
            },
            Style::Faint => {
                self.bold = false;
                return Ok(())
            },
            _ => {}
        };

        self.styles.insert(style);
        self.reopen_span()
    }
}

fn ansi_color_to_html(color: Color) -> &'static str {
    use ansi_shim::Color::*;

    match color {
        Black => "#212121",
        Red => "#E51C23",
        Green => "#259B24",
        Yellow => "#FFEB3B",
        Blue => "#5677FC",
        Magenta => "#9C27B0",
        Cyan => "#00BCD4",
        White => "#F5F5F5",
        BrightBlack => "#9E9E9E",
        BrightRed => "#FF5177",
        BrightGreen => "#5AF158",
        BrightYellow => "#FFFF00",
        BrightBlue => "#6889FF",
        BrightMagenta => "#E040FB",
        BrightCyan => "#18FFFF",
        BrightWhite => "#FFFFFF",
    }
}

fn ansi_style_to_html(styles: &BTreeSet<Style>) -> String {
    let mut css = String::new();
    let mut deco = String::new();

    for &style in styles {
        match style {
            Style::Bold => css += "font-weight: bold;",
            Style::Faint => css += "font-weight: lighter;",
            Style::Italic => css += "font-style: italic;",
            Style::Hidden => css += "visibility: hidden;",
            Style::Underline => deco += " underline",
            Style::Crossed => deco += " line-through",
            _ => {},
        };
    }

    if !deco.is_empty() {
        use std::fmt::Write;
        write!(css, "text-decoration: {};", deco);
    }

    css
}

impl<W: Write> Drop for HtmlWriter<W> {
    fn drop(&mut self) {
        self.close_span().and_then(|_| {
            self.writer.write_all("\n</pre>\n\n</body>\n</html>".as_bytes())
        }).expect("writing html footer");
    }
}

fn main() {
    let path = args().nth(1).expect("The first argument needs to be the output file");
    let out = File::create(path).expect("output file");
    let out = HtmlWriter::new(out).expect("writing html init");
    let mut out = Shim::new(out);
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    io::copy(&mut stdin, &mut out).unwrap();
}
