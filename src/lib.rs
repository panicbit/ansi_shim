extern crate vte;
#[macro_use] extern crate log;

// TODO: Improve code interpretation
// TODO: Handle I/O errors somehow

use vte::{Parser,Perform};
use std::io::{self, Write};

pub trait Terminal: Write {
    fn print(&mut self, ch: char) -> io::Result<()>;
    fn set_fg_color(&mut self, color: Color) -> io::Result<()>;
    fn set_bg_color(&mut self, color: Color) -> io::Result<()>;
    fn reset_style(&mut self) -> io::Result<()>;
    fn add_style(&mut self, style: Style) -> io::Result<()>;
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl Color {
    pub fn normal(self) -> Color {
        match self {
            Color::BrightBlack => Color::Black,
            Color::BrightRed => Color::Red,
            Color::BrightGreen => Color::Green,
            Color::BrightYellow => Color::Yellow,
            Color::BrightBlue => Color::Blue,
            Color::BrightMagenta => Color::Magenta,
            Color::BrightCyan => Color::Cyan,
            Color::BrightWhite => Color::White,
            _ => self
        }
    }

    pub fn bright(self) -> Color {
        match self {
            Color::Black => Color::BrightBlack,
            Color::Red => Color::BrightRed,
            Color::Green => Color::BrightGreen,
            Color::Yellow => Color::BrightYellow,
            Color::Blue => Color::BrightBlue,
            Color::Magenta => Color::BrightMagenta,
            Color::Cyan => Color::BrightCyan,
            Color::White => Color::BrightWhite,
            _ => self
        }
    }
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum Style {
    Bold,
    Faint,
    Italic,
    Underline,
    BlinkSlow,
    BlinkFast,
    Reverse,
    Hidden,
    Crossed,
}

pub struct Shim<T: Terminal> {
    parser: Parser,
    terminal: VteTerm<T>,
}

impl<T: Terminal> Shim<T> {
    pub fn new(terminal: T) -> Self {
        Self {
            parser: Parser::new(),
            terminal: VteTerm(terminal),
        }
    }
}

impl<T: Terminal> Write for Shim<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &byte in buf {
            self.parser.advance(&mut self.terminal, byte);
        }
        
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.terminal.0.flush()
    }
}

struct VteTerm<T: Terminal>(T);

impl<T: Terminal> VteTerm<T> {
    fn handle_formatting(&mut self, params: &[i64]) {
        use self::Color::*;
        use self::Style::*;

        if params.is_empty() {
            self.0.reset_style();
        }

        for &param in params {
            match param {
                0 => self.0.reset_style(),
                1 => self.0.add_style(Bold),
                2 => self.0.add_style(Faint),
                3 => self.0.add_style(Italic),
                4 => self.0.add_style(Underline),
                5 => self.0.add_style(BlinkSlow),
                6 => self.0.add_style(BlinkFast),
                7 => self.0.add_style(Reverse),
                8 => self.0.add_style(Hidden),
                30 => self.0.set_fg_color(Black),
                31 => self.0.set_fg_color(Red),
                32 => self.0.set_fg_color(Green),
                33 => self.0.set_fg_color(Yellow),
                34 => self.0.set_fg_color(Blue),
                35 => self.0.set_fg_color(Magenta),
                36 => self.0.set_fg_color(Cyan),
                37 => self.0.set_fg_color(White),
                40 => self.0.set_bg_color(Black),
                41 => self.0.set_bg_color(Red),
                42 => self.0.set_bg_color(Green),
                43 => self.0.set_bg_color(Yellow),
                44 => self.0.set_bg_color(Blue),
                45 => self.0.set_bg_color(Magenta),
                46 => self.0.set_bg_color(Cyan),
                47 => self.0.set_bg_color(White),
                90 => self.0.set_fg_color(BrightBlack),
                91 => self.0.set_fg_color(BrightRed),
                92 => self.0.set_fg_color(BrightGreen),
                93 => self.0.set_fg_color(BrightYellow),
                94 => self.0.set_fg_color(BrightBlue),
                95 => self.0.set_fg_color(BrightMagenta),
                96 => self.0.set_fg_color(BrightCyan),
                97 => self.0.set_fg_color(BrightWhite),
                100 => self.0.set_bg_color(BrightBlack),
                101 => self.0.set_bg_color(BrightRed),
                102 => self.0.set_bg_color(BrightGreen),
                103 => self.0.set_bg_color(BrightYellow),
                104 => self.0.set_bg_color(BrightBlue),
                105 => self.0.set_bg_color(BrightMagenta),
                106 => self.0.set_bg_color(BrightCyan),
                107 => self.0.set_bg_color(BrightWhite),
                _ => {
                    debug!("Unhandled SGR param: {}", param);
                    Ok(())
                }
            };
        }
    }
}

impl<T: Terminal> Perform for VteTerm<T> {
    fn print(&mut self, ch: char) {
        self.0.print(ch);
    }

    fn execute(&mut self, byte: u8) {
        self.0.write(&[byte]);
    }

    fn hook(&mut self, params: &[i64], intermediates: &[u8], ignore: bool) {}

    fn put(&mut self, byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]]) {}

    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, cmd: char) {
        match cmd {
            'm' => self.handle_formatting(params),
            _ => debug!("CSI: {:?} {:?} {:?} {:?}", params, intermediates, ignore, cmd),
        }
    }

    fn esc_dispatch( &mut self, params: &[i64], intermediates: &[u8], ignore: bool, byte: u8) {
        debug!("Esc: {:?} {:?} {:?} {:?}", params, intermediates, ignore, byte);
    }
}
