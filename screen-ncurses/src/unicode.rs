#![allow(clippy::never_loop)]

use crate::common::*;
use crate::ncurses::*;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::char::{self};
use core::cmp::{min};
use core::mem::{size_of};
use core::ptr::NonNull;
use either::{Left, Right};
use int_vec_2d::{Point, Range1d, Vector, Rect};
use libc::*;
use panicking::panicking;
use tvxaml_screen_base::*;
use tvxaml_screen_base::Screen as base_Screen;

struct Line {
    window: NonNull<WINDOW>,
    invalidated: bool,
}

pub struct Screen<A: Allocator> {
    error_alloc: &'static dyn Allocator,
    max_size: Option<(u16, u16)>,
    lines: Vec<Line, A>,
    cols: usize,
    chs: Vec<([char; CCHARW_MAX], attr_t), A>,
}

impl<A: Allocator> !Sync for Screen<A> { }
impl<A: Allocator> !Send for Screen<A> { }

impl<A: Allocator> Screen<A> {
    pub unsafe fn new_in(
        max_size: Option<(u16, u16)>,
        error_alloc: Option<&'static dyn Allocator>,
        alloc: A
    ) -> Result<Self, Error> where A: Clone {
        let error_alloc = error_alloc.unwrap_or(&GLOBAL);
        set_err(non_null(initscr()), "initscr", error_alloc)?;
        let size = size(max_size);
        let mut s = Screen {
            error_alloc,
            max_size,
            lines: Vec::new_in(alloc.clone()),
            cols: usize::from(size.x as u16),
            chs: Vec::new_in(alloc),
        };
        init_settings(error_alloc)?;
        s.resize()?;
        Ok(s)
    }

    fn resize(&mut self) -> Result<(), Error> {
        let size = self.size();
        let reserve = self.max_size.unwrap_or((size.x as u16, size.y as u16));
        self.lines.try_reserve(usize::from(reserve.1).saturating_sub(self.lines.len())).map_err(|_| Error::Oom)?;
        let chs_len = usize::from(reserve.1).checked_mul(usize::from(reserve.0)).ok_or(Error::Oom)?;
        self.chs.try_reserve(chs_len.saturating_sub(self.chs.len())).map_err(|_| Error::Oom)?;
        for line in &self.lines {
            set_err(non_err(unsafe { delwin(line.window.as_ptr()) }), "delwin", self.error_alloc)?;
        }
        self.lines.clear();
        let mut space_gr = ['\0'; CCHARW_MAX];
        space_gr[0] = ' ';
        space_gr[1] = '\0';
        let space = (space_gr, WA_NORMAL);
        self.cols = usize::from(size.x as u16);
        self.chs.resize(chs_len, space);
        for y in 0 .. size.y {
            let window = non_null(unsafe { newwin(1, 0, y as _, 0) }).unwrap();
            set_err(non_err(unsafe { keypad(window.as_ptr(), true) }), "keypad", self.error_alloc)?;
            self.lines.push(Line { window, invalidated: false });
        }
        Ok(())
    }

    fn start_text(line: &mut [([char; CCHARW_MAX], attr_t)], x: i16) {
        if x <= 0 { return; }
        let mut x = x as u16;
        if let Some(col) = line.get(x as usize) {
            if col.0[0] != '\0' { return; }
        } else {
            return;
        }
        loop {
            debug_assert!(x > 0);
            x -= 1;
            let col = &mut line[x as usize];
            let stop = col.0[0] != '\0';
            col.0[0] = ' ';
            col.0[1] = '\0';
            if stop { break; }
        }
    }

    fn end_text(line: &mut [([char; CCHARW_MAX], attr_t)], mut x: i16) {
        if x <= 0 { return; }
        while let Some(ref mut col) = line.get_mut(x as u16 as usize) {
            if col.0[0] != '\0' { break; }
            col.0[0] = ' ';
            col.0[1] = '\0';
            x += 1;
        }
    }

    fn update_raw(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error> {
        set_err(non_err(unsafe { curs_set(0) }), "curs_set", self.error_alloc)?;
        assert_eq!(size_of::<char>(), size_of::<wchar_t>());
        for (chs, line) in self.chs.chunks(self.cols).zip(self.lines.iter_mut()).filter(|(_, l)| l.invalidated) {
            line.invalidated = false;
            if chs.is_empty() { continue; }
            set_err(non_err(unsafe { wmove(line.window.as_ptr(), 0, 0) }), "wmove", self.error_alloc)?;
            for &ch in chs {
                if ch.0[0] == '\0' { continue; }
                set_err(non_err(unsafe { wattrset(line.window.as_ptr(), ch.1 as _) }), "wattrset", self.error_alloc)?;
                let _ = unsafe { waddnwstr(line.window.as_ptr(), ch.0.as_ptr() as _, CCHARW_MAX as _) };
            }
            set_err(non_err(unsafe { wnoutrefresh(line.window.as_ptr()) }), "wnoutrefresh", self.error_alloc)?;
        }
        set_err(non_err(unsafe { doupdate() }), "doupdate", self.error_alloc)?;
        let cursor = cursor.and_then(|cursor| {
            if (Rect { tl: Point { x: 0, y: 0 }, size: self.size() }).contains(cursor) {
                Some(cursor)
            } else {
                None
            }
        });
        let window = if let Some(cursor) = cursor {
            let window = self.lines[cursor.y as u16 as usize].window;
            set_err(non_err(unsafe { wmove(window.as_ptr(), 0, cursor.x as _) }), "wmove", self.error_alloc)?;
            set_err(non_err(unsafe { curs_set(1) }), "curs_set", self.error_alloc)?;
            Some(window)
        } else if let Some(line) = self.lines.first() {
            if self.cols == 0 {
                None
            } else {
                let window = line.window;
                set_err(non_err(unsafe { wmove(window.as_ptr(), 0, 0) }), "wmove", self.error_alloc)?;
                Some(window)
            }
        } else {
            None
        };
        let window = window.unwrap_or_else(|| unsafe { NonNull::new(stdscr).unwrap() });
        set_err(non_err(unsafe { nodelay(window.as_ptr(), !wait) }), "nodelay", self.error_alloc)?;
        let e = read_event(window, |w| {
            let mut c: wint_t = 0;
            let key = unsafe { wget_wch(w.as_ptr(), &mut c as *mut _) };
            if key == ERR { return None; }
            if key != KEY_CODE_YES { return Some(Right(char::from_u32(c as wchar_t as u32).unwrap())); }
            Some(Left(c as _))
        }, self.error_alloc)?;
        match e {
            Some(Event::Resize) => self.resize()?,
            Some(Event::Key(_, Key::Ctrl(Ctrl::L))) => unsafe { clearok(curscr, true); },
            _ => { }
        }
        Ok(e)
    }
}

fn size(max_size: Option<(u16, u16)>) -> Vector {
    let mut x = (unsafe { COLS }).clamp(0, i16::MAX.into()) as i16;
    let mut y = (unsafe { LINES }).clamp(0, i16::MAX.into()) as i16;
    if let Some(max_size) = max_size {
        x = min(max_size.0, x as u16) as i16;
        y = min(max_size.1, y as u16) as i16;
    }
    Vector { x, y }
}

impl<A: Allocator> Drop for Screen<A> {
    #![allow(clippy::panicking_unwrap)]
    fn drop(&mut self) {
        let e = unsafe { non_err(endwin()) };
        if e.is_err() && !panicking() { e.unwrap(); }
    }
}

impl<A: Allocator> base_Screen for Screen<A> {
    fn size(&self) -> Vector { size(self.max_size) }

    fn out(
        &mut self,
        p: Point,
        fg: Fg,
        bg: Bg,
        text: &str,
        hard: Range1d,
        soft: Range1d
    ) -> Range1d {
        debug_assert!(p.y >= 0 && p.y < self.size().y);
        debug_assert!(hard.start >= 0 && hard.end > hard.start && hard.end <= self.size().x);
        debug_assert!(soft.start >= 0 && soft.end > soft.start && soft.end <= self.size().x);
        let text_end = if soft.end <= p.x { return Range1d { start: 0, end: 0 } } else { soft.end.saturating_sub(p.x) };
        let text_start = if soft.start <= p.x { 0 } else { soft.start.saturating_sub(p.x) };
        let line = &mut self.chs[usize::from(p.y as u16) * self.cols .. (usize::from(p.y as u16) + 1) * self.cols];
        self.lines[p.y as u16 as usize].invalidated = true;
        let attr = unsafe { attr_ch(fg, bg) };
        let graphemes = graphemes(text).map(|(g, w)| {
            let mut grapheme = ['\0'; CCHARW_MAX];
            let mut chars = text[g].chars();
            grapheme[0] = chars.next().unwrap();
            for g in grapheme[1 ..].iter_mut() {
                if let Some(c) = chars.next() {
                    *g = c;
                } else {
                    break;
                }
            }
            (grapheme, w)
        });
        let mut x0 = None;
        let mut x = p.x;
        let mut n = 0i16;
        for (g, w) in graphemes {
            if x >= hard.end { break; }
            if n >= text_end { break; }
            n = n.saturating_add(w);
            let before_text_start = n <= text_start;
            if before_text_start {
                x = min(hard.end, x.saturating_add(w));
                continue;
            }
            if x < hard.start {
                x = min(hard.end, x.saturating_add(w));
                if x > hard.start {
                    debug_assert!(x0.is_none());
                    Self::start_text(line, hard.start);
                    x0 = Some(hard.start);
                    for i in hard.start .. x {
                        let col = &mut line[i as u16 as usize];
                        col.0[0] = ' ';
                        col.0[1] = '\0';
                    }
                }
                continue;
            }
            if x0.is_none() {
                Self::start_text(line, x);
                x0 = Some(x);
            }
            let next_x = min(hard.end, x.saturating_add(w));
            if next_x - x < w {
                for i in x .. next_x {
                    let col = &mut line[i as u16 as usize];
                    col.0[0] = ' ';
                    col.0[1] = '\0';
                }
                x = next_x;
                break;
            }
            let col = &mut line[x as u16 as usize];
            col.0 = g;
            col.1 = attr;
            for i in x + 1 .. next_x {
               line[i as u16 as usize].0[0] = '\0';
            }
            x = next_x;
        }
        if let Some(x0) = x0 {
            Self::end_text(line, x);
            Range1d { start: x0, end: x }
        } else {
            Range1d { start: 0, end: 0 }
        }
    }

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error> {
        self.update_raw(cursor, wait)
    }
}
