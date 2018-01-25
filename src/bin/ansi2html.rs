extern crate ansi_shim;

// TODO: Handle html entities
// TODO: Provide finalize method to check for errors when writing html footer

use ansi_shim::{Terminal,Shim,Color,Style};
use std::io::{self, Write};
use std::env::args;
use std::fs::File;

struct HtmlWriter<W: Write> {
    writer: W,
    fg_color: Color,
    bg_color: Color,
    style: Style,
}

const HEADER: &str = "\
<html>
<head>
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
            style: Style::Reset,
        };
        writer.write_all(HEADER.as_bytes())?;
        writer.open_span()?;

        Ok(writer)
    }

    fn open_span(&mut self) -> io::Result<()> {
        write!(self.writer, "<span style='color: {fg}; background: {bg}; {style}'>",
            fg = ansi_color_to_html(self.fg_color, self.style == Style::Bright),
            bg = ansi_color_to_html(self.bg_color, false),
            style = ansi_style_to_html(self.style),
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
    fn set_fg_color(&mut self, color: Color) -> io::Result<()> {
        self.fg_color = color;
        self.reopen_span()
    }

    fn set_bg_color(&mut self, color: Color) -> io::Result<()> {
        self.bg_color = color;
        self.reopen_span()
    }

    fn set_style(&mut self, style: Style) -> io::Result<()> {
        if style == Style::Reset {
            self.bg_color = Color::Black;
            self.fg_color = Color::White;
        };
        self.style = style;
        self.reopen_span()
    }
}

fn ansi_color_to_html(color: Color, bright: bool) -> &'static str {
    use ansi_shim::Color::*;

    match (color, bright) {
        (Black, true) => "#9E9E9E",
        (Red, true) => "#FF5177",
        (Green, true) => "#5AF158",
        (Yellow, true) => "#FFFF00",
        (Blue, true) => "#6889FF",
        (Magenta, true) => "#E040FB",
        (Cyan, true) => "#18FFFF",
        (White, true) => "#FFFFFF",
        (Black, _) => "#212121",
        (Red, _) => "#E51C23",
        (Green, _) => "#259B24",
        (Yellow, _) => "#FFEB3B",
        (Blue, _) => "#5677FC",
        (Magenta, _) => "#9C27B0",
        (Cyan, _) => "#00BCD4",
        (White, _) => "#F5F5F5",
    }
}

fn ansi_style_to_html(style: Style) -> &'static str {
    match style {
        Style::Reset => "",
        Style::Bright => "font-weight: bold;",
        Style::Dim => "",
        Style::Underscore => "text-decoration: underline;",
        Style::Blink => "",
        Style::Reverse => "",
        Style::Hidden => "visibility: hidden;",
    }
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
