extern crate winapi;
extern crate ansi_shim;

use ansi_shim::{Terminal,Color,Style,Shim};
use std::ptr;
use std::mem;
use winapi::shared::minwindef::WORD;

use winapi::um::wincon::*;
use std::io::{self, Write};
use winapi::um::winbase::STD_OUTPUT_HANDLE;
use winapi::um::processenv::GetStdHandle;
use winapi::um::winnt::HANDLE;

fn main() {
    let text = include_bytes!("../demo.txt");

    let mut term = Shim::new(WinTerm::new());

    term.write_all(text).unwrap();

    loop {
        ::std::thread::sleep_ms(5000);
    }
}

struct WinTerm {
    handle: HANDLE, 
    bold: bool,
}

impl WinTerm {
    pub fn new() -> Self {
        Self {
            handle: unsafe { GetStdHandle(STD_OUTPUT_HANDLE) },
            bold: false,
        }
    }
}

impl WinTerm {
    fn get_text_attributes(&self) -> Option<WORD> {
        unsafe {
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = mem::zeroed();

            if GetConsoleScreenBufferInfo(self.handle, &mut info) == 0 {
                return None;
            }

            Some(info.wAttributes)
        }
    }

    fn set_text_attributes(&mut self, attrs: WORD) {
        unsafe {
            self.flush();
            SetConsoleTextAttribute(self.handle, attrs);
        }
    }
}

impl Terminal for WinTerm {
    fn print(&mut self, ch: char) -> io::Result<()> {
        print!("{}", ch);
        Ok(())
    }

    fn set_fg_color(&mut self, color: Color) -> io::Result<()> {
        unsafe {
            let mut attrs = match self.get_text_attributes() {
                None => return Ok(()),
                Some(attrs) => attrs,
            };

            attrs &= !(FOREGROUND_RED | FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_INTENSITY);

            if self.bold {
                attrs |= FOREGROUND_INTENSITY;
            }

            let color = match color {
                Color::Black => 0,
                Color::Red => FOREGROUND_RED,
                Color::Green => FOREGROUND_GREEN,
                Color::Yellow => FOREGROUND_RED | FOREGROUND_GREEN,
                Color::Blue => FOREGROUND_BLUE,
                Color::Magenta => FOREGROUND_RED | FOREGROUND_BLUE,
                Color::Cyan => FOREGROUND_BLUE | FOREGROUND_GREEN,
                Color::White => FOREGROUND_RED | FOREGROUND_BLUE | FOREGROUND_GREEN,
                Color::BrightBlack => FOREGROUND_INTENSITY,
                Color::BrightRed => FOREGROUND_RED | FOREGROUND_INTENSITY,
                Color::BrightGreen => FOREGROUND_GREEN | FOREGROUND_INTENSITY,
                Color::BrightYellow => FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_INTENSITY,
                Color::BrightBlue => FOREGROUND_BLUE | FOREGROUND_INTENSITY,
                Color::BrightMagenta => FOREGROUND_RED | FOREGROUND_BLUE | FOREGROUND_INTENSITY,
                Color::BrightCyan => FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_INTENSITY,
                Color::BrightWhite => FOREGROUND_RED | FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_INTENSITY,
                _ => return Ok(())
            };

            attrs |= color;

            self.set_text_attributes(attrs);

            Ok(())
        }
    }

    fn set_bg_color(&mut self, color: Color) -> io::Result<()> {
        unsafe {
            let mut attrs = match self.get_text_attributes() {
                None => return Ok(()),
                Some(attrs) => attrs,
            };

            attrs &= !(BACKGROUND_RED | BACKGROUND_BLUE | BACKGROUND_GREEN | BACKGROUND_INTENSITY);

            let color = match color {
                Color::Black => 0,
                Color::Red => BACKGROUND_RED,
                Color::Green => BACKGROUND_GREEN,
                Color::Yellow => BACKGROUND_RED | BACKGROUND_GREEN,
                Color::Blue => BACKGROUND_BLUE,
                Color::Magenta => BACKGROUND_RED | BACKGROUND_BLUE,
                Color::Cyan => BACKGROUND_BLUE | BACKGROUND_GREEN,
                Color::White => BACKGROUND_RED | BACKGROUND_BLUE | BACKGROUND_GREEN,
                Color::BrightBlack => BACKGROUND_INTENSITY,
                Color::BrightRed => BACKGROUND_RED | BACKGROUND_INTENSITY,
                Color::BrightGreen => BACKGROUND_GREEN | BACKGROUND_INTENSITY,
                Color::BrightYellow => BACKGROUND_RED | BACKGROUND_GREEN | BACKGROUND_INTENSITY,
                Color::BrightBlue => BACKGROUND_BLUE | BACKGROUND_INTENSITY,
                Color::BrightMagenta => BACKGROUND_RED | BACKGROUND_BLUE | BACKGROUND_INTENSITY,
                Color::BrightCyan => BACKGROUND_BLUE | BACKGROUND_GREEN | BACKGROUND_INTENSITY,
                Color::BrightWhite => BACKGROUND_RED | BACKGROUND_BLUE | BACKGROUND_GREEN | BACKGROUND_INTENSITY,
                _ => return Ok(())
            };

            attrs |= color;

            self.set_text_attributes(attrs);

            Ok(())
        }
    }

    fn reset_style(&mut self) -> io::Result<()> {
        let attrs = FOREGROUND_RED | FOREGROUND_GREEN | FOREGROUND_BLUE | FOREGROUND_INTENSITY;
        self.set_text_attributes(attrs);
        self.bold = false;
        Ok(())
    }

    fn add_style(&mut self, style: Style) -> io::Result<()> {
        unsafe {
            let mut attrs = match self.get_text_attributes() {
                None => return Ok(()),
                Some(attrs) => attrs,
            };

            match style {
                Style::Bold => {
                    self.bold = true;
                    attrs |= FOREGROUND_INTENSITY
                },
                _ => {}
            }

            self.set_text_attributes(attrs);

            Ok(())
        }
    }
}

impl Write for WinTerm {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}
