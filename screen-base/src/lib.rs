#![feature(allocator_api)]
#![feature(generic_arg_infer)]
#![feature(iter_advance_by)]
#![feature(stmt_expr_attributes)]
#![feature(trusted_len)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_map)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::alloc::Allocator;
use core::fmt::{self, Debug, Display, Formatter};
use core::iter::{Iterator, DoubleEndedIterator, FusedIterator};
use core::num::NonZeroU16;
use core::ops::Range;
use enum_derive_2018::{EnumDisplay, EnumFromStr, IterVariants};
use int_vec_2d::{Point, Vector, Range1d};
use macro_attr_2018::macro_attr;
use serde::{Serialize, Deserialize};
use unicode_width::UnicodeWidthChar;

pub fn char_width(c: char) -> i16 {
    if c == '\0' { 0 } else { c.width().map_or(0, |x| i16::try_from(x).unwrap()) }
}

pub fn text_width(s: &str) -> i16 {
    s.chars().map(char_width).fold(0, |s, c| s.wrapping_add(c))
}

pub fn trim_text(mut s: &str) -> &str {
    loop {
        let Some(c) = s.chars().next() else { break; };
        if c == ' ' || char_width(c) == 0 {
            s = &s[c.len_utf8() ..];
        } else {
            break;
        }
    }
    loop {
        let Some(c) = s.chars().next_back() else { break; };
        if c == ' ' || char_width(c) == 0 {
            s = &s[.. s.len() - c.len_utf8()];
        } else {
            break;
        }
    }
    s
}

pub fn is_text_fit_in(w: i16, s: &str) -> bool {
    let mut w = w as u16;
    for c in s.chars() {
        if let Some(new_w) = w.checked_sub(char_width(c) as u16) {
            w = new_w;
        } else {
            return false;
        }
    }
    true
}

pub fn graphemes(text: &str) -> Graphemes {
    Graphemes(text)
}

pub struct Graphemes<'a>(&'a str);

impl<'a> Iterator for Graphemes<'a> {
    type Item = (Range<usize>, i16);

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.0.char_indices();
        let (start, w) = loop {
            let Some(c) = chars.next() else { return None; };
            if c.1 == '\0' { continue; }
            let Some(w) = c.1.width() else { continue; };
            if w == 0 { continue; }
            break (c, i16::try_from(w).unwrap());
        };
        let mut end = start;
        loop {
            let Some(c) = chars.next() else { break; };
            if c.1 == '\0' { break; }
            let Some(w) = c.1.width() else { break; };
            if w != 0 { break; }
            end = c;
        }
        let end = end.0 + end.1.len_utf8();
        let item = (start.0 .. end, w);
        self.0 = &self.0[end ..];
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.0.len()))
    }
}

impl<'a> DoubleEndedIterator for Graphemes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut chars = self.0.char_indices();
        'end: loop {
            let Some(c) = chars.next_back() else { return None; };
            if c.1 == '\0' { continue; }
            let Some(w) = c.1.width() else { continue; };
            let end = c;
            let mut start = (c, i16::try_from(w).unwrap());
            while start.1 == 0 {
                let Some(c) = chars.next_back() else { return None; };
                if c.1 == '\0' { continue 'end; }
                let Some(w) = c.1.width() else { continue 'end; };
                start = (c, i16::try_from(w).unwrap());
            }
            let item = (start.0.0 .. end.0 + end.1.len_utf8(), start.1);
            self.0 = &self.0[.. start.0.0];
            break Some(item);
        }
    }
}

impl<'a> FusedIterator for Graphemes<'a> { }

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!, IterVariants!(BgVariants))]
    #[derive(Serialize, Deserialize)]
    pub enum Bg {
        None,
        Black,
        Red,
        Green,
        Brown,
        Blue,
        Magenta,
        Cyan,
        LightGray
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!, IterVariants!(FgVariants))]
    #[derive(Serialize, Deserialize)]
    pub enum Fg {
        Black,
        Red,
        Green,
        Brown,
        Blue,
        Magenta,
        Cyan,
        LightGray,
        DarkGray,
        BrightRed,
        BrightGreen,
        Yellow,
        BrightBlue,
        BrightMagenta,
        BrightCyan,
        White
    }
}

#[derive(Debug)]
pub struct TryFromBgError;

impl TryFrom<Bg> for Fg {
    type Error = TryFromBgError;

    fn try_from(bg: Bg) -> Result<Fg, Self::Error> {
        match bg {
            Bg::None => Err(TryFromBgError),
            Bg::Black => Ok(Fg::Black),
            Bg::Red => Ok(Fg::Red),
            Bg::Green => Ok(Fg::Green),
            Bg::Brown => Ok(Fg::Brown),
            Bg::Blue => Ok(Fg::Blue),
            Bg::Magenta => Ok(Fg::Magenta),
            Bg::Cyan => Ok(Fg::Cyan),
            Bg::LightGray => Ok(Fg::LightGray),
        }
    }
}

#[derive(Debug)]
pub struct TryFromFgError;

impl TryFrom<Fg> for Bg {
    type Error = TryFromFgError;

    fn try_from(fg: Fg) -> Result<Bg, Self::Error> {
        match fg {
            Fg::Black => Ok(Bg::Black),
            Fg::Red => Ok(Bg::Red),
            Fg::Green => Ok(Bg::Green),
            Fg::Brown => Ok(Bg::Brown),
            Fg::Blue => Ok(Bg::Blue),
            Fg::Magenta => Ok(Bg::Magenta),
            Fg::Cyan => Ok(Bg::Cyan),
            Fg::LightGray => Ok(Bg::LightGray),
            _ => Err(TryFromFgError),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
pub enum Ctrl {
    At, A, B, C, D, E, F, G, J, K, L, N,
    O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Backslash, Bracket, Caret, Underscore
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
#[non_exhaustive]
pub enum Key {
    Char(char),
    Alt(char),
    Ctrl(Ctrl),
    Enter,
    Escape,
    Down,
    Up,
    Left,
    Right,
    Home,
    End,
    Backspace,
    Delete,
    Insert,
    PageDown,
    PageUp,
    Tab,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
#[non_exhaustive]
pub enum Event {
    Resize,
    Key(NonZeroU16, Key),
    LmbDown(Point),
    LmbUp(Point),
}

pub enum Error {
    Oom,
    System(Box<dyn Display, &'static dyn Allocator>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::Oom => write!(f, "out of memory"),
            Error::System(msg) => write!(f, "{msg}")
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

pub trait Screen {
    fn size(&self) -> Vector;

    fn out(
        &mut self,
        p: Point,
        fg: Fg,
        bg: Bg,
        text: &str,
        hard: Range1d,
        soft: Range1d,
    ) -> Range1d;

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error>;
}
