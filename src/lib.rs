extern crate pest;
#[macro_use] extern crate pest_derive;

use pest::Parser;

use std::io::{self, Write};
use std::str;

pub trait Terminal: Write {
    fn set_fg_color(&mut self, color: Color) -> io::Result<()>;
    fn set_bg_color(&mut self, color: Color) -> io::Result<()>;
    fn set_style(&mut self, style: Style) -> io::Result<()>;
}

const ESC: u8 = b'\x1B';

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

impl Color {
    fn from_ansi(ch: char) -> Self {
        match ch {
            '0' => Color::Black,
            '1' => Color::Red,
            '2' => Color::Green,
            '3' => Color::Yellow,
            '4' => Color::Blue,
            '5' => Color::Magenta,
            '6' => Color::Cyan,
            '7' => Color::White,
            _ => unreachable!("BUG, invalid color {:?}", ch)
        }
    } 
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

impl Style {
    fn from_ansi(ch: char) -> Self {
        match ch {
            '0' => Style::Reset,
            '1' => Style::Bright,
            '2' => Style::Dim,
            '3' => Style::Underscore,
            '4' => Style::Blink,
            '5' => Style::Reverse,
            '6' => Style::Hidden,
            _ => unreachable!("BUG, invalid style {:?}", ch)
        }
    } 
}

pub struct Shim<T: Terminal> {
    terminal: T,
    code_buf: Vec<u8>,
}

impl<T: Terminal> Shim<T> {
    pub fn new(terminal: T) -> Self {
        Self {
            terminal,
            code_buf: Vec::with_capacity(10),
        }
    }

    fn try_parse(&mut self) -> ParseResult {
        debug_assert!(!self.code_buf.is_empty());

        if self.code_buf.len() >= 10 {
            return ParseResult::Invalid;
        }

        let code = match str::from_utf8(&self.code_buf) {
            Ok(code) => code,
            Err(_) => return ParseResult::Invalid,
        };

        println!("code: {:?}", code);
        
        Self::parse_escape(code, &mut self.terminal).unwrap_or(ParseResult::Incomplete)
    }

    fn parse_escape<'i>(code: &'i str, terminal: &mut T) -> Result<ParseResult, pest::Error<'i, Rule>> {
        let pairs = EscapeParser::parse(Rule::escape, code)?;

        for pair in pairs {
            println!("{:?}", pair);
            match pair.as_rule() {
                Rule::fg_color => {
                    let res = terminal.set_fg_color(Color::from_ansi(pair.as_str().chars().next().unwrap()));
                    if res.is_err() { return Ok(ParseResult::Consumed(res)) }
                },
                Rule::bg_color => {
                    let res = terminal.set_bg_color(Color::from_ansi(pair.as_str().chars().next().unwrap()));
                    if res.is_err() { return Ok(ParseResult::Consumed(res)) }
                },
                Rule::style => {
                    let res = terminal.set_style(Style::from_ansi(pair.as_str().chars().next().unwrap()));
                    if res.is_err() { return Ok(ParseResult::Consumed(res)) }
                },
                _ => unreachable!("BUG, unexpected {:?}", pair.as_rule()),
            }
        }
        
        Ok(ParseResult::Consumed(Ok(())))
    }

}

impl<T: Terminal> Write for Shim<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        if self.code_buf.is_empty() {
            let pos = buf.iter().position(|&b| b == ESC).unwrap_or(buf.len());
            if pos > 0 {
                return self.terminal.write(&buf[..pos]);
            }
        }

        if buf[0] == ESC {
            self.terminal.write_all(&self.code_buf)?;
            self.code_buf.clear();
            self.code_buf.push(ESC);
            return Ok(1)
        }

        self.code_buf.push(buf[0]);

        match self.try_parse() {
            ParseResult::Invalid => {
                self.terminal.write_all(&self.code_buf)?;
                self.code_buf.clear();
            },
            ParseResult::Consumed(Ok(_)) => self.code_buf.clear(),
            ParseResult::Consumed(Err(e)) => {
                // Rollback to be able to retry
                self.code_buf.pop();
                return Err(e);
            },
            ParseResult::Incomplete => {}
        }
        
        Ok(1)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.terminal.flush()
    }
}

enum ParseResult {
    Consumed(io::Result<()>),
    Incomplete,
    Invalid
}

#[derive(Parser)]
#[grammar = "escapes.pest"]
struct EscapeParser;

#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("escapes.pest");
