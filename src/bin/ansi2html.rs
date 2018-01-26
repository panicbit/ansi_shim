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
    span_needs_reopen: bool,
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

        .black { color: #212121; }
        .red { color: #E51C23; }
        .green { color: #259B24; }
        .yellow { color: #FFEB3B; }
        .blue { color: #5677FC; }
        .magenta { color: #9C27B0; }
        .cyan { color: #00BCD4; }
        .white { color: #F5F5F5; }
        .bright-black, .bold.black { color: #9E9E9E; }
        .bright-red, .bold.red { color: #FF5177; }
        .bright-green, .bold.green { color: #5AF158; }
        .bright-yellow, .bold.yellow { color: #FFFF00; }
        .bright-blue, .bold.blue { color: #6889FF; }
        .bright-magenta, .bold.magenta { color: #E040FB; }
        .bright-cyan, .bold.cyan { color: #18FFFF; }
        .bright-white, .bold.white { color: #FFFFFF; }

        .bg-black { background-color: #212121; }
        .bg-red { background-color: #E51C23; }
        .bg-green { background-color: #259B24; }
        .bg-yellow { background-color: #FFEB3B; }
        .bg-blue { background-color: #5677FC; }
        .bg-magenta { background-color: #9C27B0; }
        .bg-cyan { background-color: #00BCD4; }
        .bg-white { background-color: #F5F5F5; }
        .bg-bright-black { background-color: #9E9E9E; }
        .bg-bright-red { background-color: #FF5177; }
        .bg-bright-green { background-color: #5AF158; }
        .bg-bright-yellow { background-color: #FFFF00; }
        .bg-bright-blue { background-color: #6889FF; }
        .bg-bright-magenta { background-color: #E040FB; }
        .bg-bright-cyan { background-color: #18FFFF; }
        .bg-bright-white { background-color: #FFFFFF; }

        .bold { font-weight: bold; }

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
            span_needs_reopen: false,
        };
        writer.write_all(HEADER.as_bytes())?;
        writer.open_span()?;

        Ok(writer)
    }

    fn open_span(&mut self) -> io::Result<()> {
        let fg = self.fg_color_class();
        let bg = self.bg_color_class();
        write!(self.writer, "<span class='{fg} {bg}{bold}' style='{style}'>",
            fg = fg,
            bg = bg,
            bold = if self.bold { " bold" } else { "" },
            style = ansi_style_to_html(&self.styles),
        )
    }

    fn close_span(&mut self) -> io::Result<()> {
        self.writer.write_all("</span>".as_bytes())
    }

    fn reopen_span(&mut self) -> io::Result<()> {
        self.span_needs_reopen = true;
        Ok(())
    }

    fn execute_reopen_span(&mut self) -> io::Result<()> {
        if self.span_needs_reopen {
            self.close_span()?;
            self.open_span()?;
            self.span_needs_reopen = false;
        }

        Ok(())
    }

    fn fg_color_class(&self) -> &'static str {
        use self::Color::*;
        match self.fg_color {
            Black => "black",
            Red => "red",
            Green => "green",
            Yellow => "yellow",
            Blue => "blue",
            Magenta => "magenta",
            Cyan => "cyan",
            White => "white",
            BrightBlack => "bright-black",
            BrightRed => "bright-red",
            BrightGreen => "bright-green",
            BrightYellow => "bright-yellow",
            BrightBlue => "bright-blue",
            BrightMagenta => "bright-magenta",
            BrightCyan => "bright-cyan",
            BrightWhite => "bright-white",
        }
    }

    fn bg_color_class(&self) -> &'static str {
        use self::Color::*;
        match self.bg_color {
            Black => "bg-black",
            Red => "bg-red",
            Green => "bg-green",
            Yellow => "bg-yellow",
            Blue => "bg-blue",
            Magenta => "bg-magenta",
            Cyan => "bg-cyan",
            White => "bg-white",
            BrightBlack => "bg-bright-black",
            BrightRed => "bg-bright-red",
            BrightGreen => "bg-bright-green",
            BrightYellow => "bg-bright-yellow",
            BrightBlue => "bg-bright-blue",
            BrightMagenta => "bg-bright-magenta",
            BrightCyan => "bg-bright-cyan",
            BrightWhite => "bg-bright-white",
        }
    }
}

impl<W: Write> Write for HtmlWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.execute_reopen_span()?;
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> Terminal for HtmlWriter<W> {
    fn print(&mut self, ch: char) -> io::Result<()> {
        self.execute_reopen_span()?;
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

fn ansi_style_to_html(styles: &BTreeSet<Style>) -> String {
    let mut css = String::new();
    let mut deco = String::new();

    for &style in styles {
        match style {
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
