extern crate vte;

// TODO: Improve code interpretation
// TODO: Handle I/O errors somehow

use vte::{Parser,Perform};
use std::io::{self, Write};

pub trait Terminal: Write {
    fn print(&mut self, ch: char) -> io::Result<()>;
    fn set_fg_color(&mut self, color: Color) -> io::Result<()>;
    fn set_bg_color(&mut self, color: Color) -> io::Result<()>;
    fn set_style(&mut self, style: Style) -> io::Result<()>;
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum Style {
    Reset,
    Bright,
    Dim,
    Underscore,
    Blink,
    Reverse,
    Hidden,
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
    fn handle_formatting(&mut self, param: i64) {
        match param {
            0 => self.0.set_style(Style::Reset),
            1 => self.0.set_style(Style::Bright),
            2 => self.0.set_style(Style::Dim),
            3 => self.0.set_style(Style::Underscore),
            4 => self.0.set_style(Style::Blink),
            5 => self.0.set_style(Style::Reverse),
            6 => self.0.set_style(Style::Hidden),
            30 => self.0.set_fg_color(Color::Black),
            31 => self.0.set_fg_color(Color::Red),
            32 => self.0.set_fg_color(Color::Green),
            33 => self.0.set_fg_color(Color::Yellow),
            34 => self.0.set_fg_color(Color::Blue),
            35 => self.0.set_fg_color(Color::Magenta),
            36 => self.0.set_fg_color(Color::Cyan),
            37 => self.0.set_fg_color(Color::White),
            40 => self.0.set_bg_color(Color::Black),
            41 => self.0.set_bg_color(Color::Red),
            42 => self.0.set_bg_color(Color::Green),
            43 => self.0.set_bg_color(Color::Yellow),
            44 => self.0.set_bg_color(Color::Blue),
            45 => self.0.set_bg_color(Color::Magenta),
            46 => self.0.set_bg_color(Color::Cyan),
            47 => self.0.set_bg_color(Color::White),
            _ => Ok(())
        };
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
            'm' => for &param in params { self.handle_formatting(param) },
            _ => {}
        }
        println!("CSI: {:?} {:?} {:?} {:?}", params, intermediates, ignore, cmd);
    }

    fn esc_dispatch( &mut self, params: &[i64], intermediates: &[u8], ignore: bool, byte: u8) {
        println!("Esc: {:?} {:?} {:?} {:?}", params, intermediates, ignore, byte);
    }
}
