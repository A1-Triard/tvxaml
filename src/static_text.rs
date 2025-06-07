use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;
use crate::template::{Template, NameResolver};

mod text_renderer {
    use either::{Left, Right};
    use int_vec_2d::{Point, Rect, Vector, HAlign, Range1d};
    use iter_identify_first_last::IteratorIdentifyFirstLastExt;
    use itertools::Itertools;
    use serde::{Serialize, Deserialize};
    use std::cmp::{max, min};
    use std::iter::{self};
    use std::mem::{replace, transmute};
    use std::slice::{self};
    use tvxaml_screen_base::{text_width, trim_text};
    use unicode_width::UnicodeWidthChar;

    #[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[derive(Serialize, Deserialize)]
    pub enum TextWrapping {
        NoWrap,
        Wrap,
        WrapWithOverflow,
    }

    pub fn render_text(
        mut r: impl FnMut(Point, &str),
        bounds: Rect,
        align: Option<HAlign>,
        wrapping: TextWrapping,
        text: &str,
    ) -> Rect {
        let mut rendered = Rect { tl: bounds.tl, size: Vector::null() };
        for block in text.split('\n') {
            let block_bounds = Rect::from_tl_br(rendered.bl(), bounds.br());
            if block_bounds.is_empty() { break; }
            let mut block_rendered = render_block(&mut r, block_bounds, align, wrapping, block);
            block_rendered.size.x = max(block_rendered.size.x as u16, rendered.size.x as u16) as i16;
            rendered = Rect::from_tl_br(bounds.tl, block_rendered.br());
        }
        rendered
    }

    fn render_block(
        mut r: impl FnMut(Point, &str),
        bounds: Rect,
        align: Option<HAlign>,
        wrapping: TextWrapping,
        text: &str,
    ) -> Rect {
        if text.is_empty() {
            return Rect { tl: bounds.tl, size: Vector { x: 0, y: 1 } };
        }
        if wrapping != TextWrapping::NoWrap {
            let words = split_words(text);
            let runs = words.identify_first().map(|(f, x)| (!f, x, text_width(x))).flat_map(|(s, x, w)| {
                if wrapping == TextWrapping::WrapWithOverflow || w as u16 <= bounds.w() as u16 {
                    Left(iter::once((s, x, min(w as u16, bounds.w() as u16) as i16)))
                } else {
                    Right(
                        graphemes(x)
                            .identify_first()
                            .map(move |(first, (g, w))| (
                                if first { s } else { false },
                                g,
                                min(w as u16, bounds.w() as u16) as i16
                            ))
                    )
                }
            });
            let lines = wrap(bounds, runs);
            let mut y = bounds.t();
            let mut range = Range1d { start: 0, end: 0 };
            for line in lines {
                let line_range = render_line(
                    &mut r,
                    Rect { tl: Point { x: bounds.l(), y }, size: Vector { x: bounds.w(), y: 1 } },
                    align,
                    line,
                );
                range = range.union(line_range).unwrap_or(bounds.h_range());
                y = y.wrapping_add(1);
            }
            Rect::from_h_v_ranges(range, Range1d { start: bounds.t(), end: y })
        } else {
            let words = split_words(text);
            let runs = words.identify_first().map(|(f, x)| (!f, x, text_width(x))).collect();
            let range = render_line(r, bounds, align, runs);
            Rect::from_h_v_ranges(range, Range1d { start: bounds.t(), end: bounds.t().wrapping_add(1) })
        }
    }

    fn split_words(text: &str) -> impl Iterator<Item=&str> {
        let mut words = text.split(' ').map(trim_text).filter(|x| !x.is_empty());
        let first_word = if let Some(first_word) = words.next() {
            unsafe { transmute::<&[u8], &str>(
                slice::from_ptr_range(text.as_ptr() .. first_word.as_ptr().add(first_word.len()))
            ) }
        } else {
            text
        };
        let last_word = if let Some(last_word) = words.next_back() {
            Some(unsafe { transmute::<&[u8], &str>(
                slice::from_ptr_range(last_word.as_ptr() .. text.as_ptr().add(text.len()))
            ) })
        } else {
            None
        };
        iter::once(first_word).chain(words.chain(last_word.into_iter()))
    }
 
    fn render_line(
        mut r: impl FnMut(Point, &str),
        bounds: Rect,
        align: Option<HAlign>,
        line: Vec<(bool, &str, i16)>,
    ) -> Range1d {
        match align {
            None => {
                let space_runs_count = line.iter().filter(|x| x.0).count();
                if space_runs_count == 0 {
                    let mut x = bounds.l();
                    for run in line {
                        r(Point { x, y: bounds.t() }, run.1);
                        x = x.wrapping_add(run.2);
                    }
                    Range1d { start: bounds.l(), end: x }
                } else {
                    let min_width = line.iter().map(|x| x.2).fold(0i16, |s, w| s.wrapping_add(w));
                    let spaces_count = ((bounds.w() as u16) - (min_width as u16)) as usize;
                    let spaces_per_run = spaces_count / space_runs_count;
                    let spaces_runs_head_len = spaces_count % space_runs_count;
                    let mut x = bounds.l();
                    for (n, run) in line.into_iter().enumerate() {
                        if n == 0 || !run.0 {
                        } else if n <= spaces_runs_head_len {
                            x = x.wrapping_add((spaces_per_run + 1) as u16 as i16);
                        } else {
                            x = x.wrapping_add(spaces_per_run as u16 as i16);
                        }
                        r(Point { x, y: bounds.t() }, run.1);
                        x = x.wrapping_add(run.2);
                    }
                    bounds.h_range()
                }
            },
            Some(HAlign::Left) => {
                let mut x = bounds.l();
                for run in line {
                    if run.0 {
                        x = x.wrapping_add(1);
                    }
                    r(Point { x, y: bounds.t() }, run.1);
                    x = x.wrapping_add(run.2);
                }
                Range1d { start: bounds.l(), end: x }
            },
            Some(HAlign::Right) => {
                let mut x = bounds.r();
                for run in line.into_iter().rev() {
                    x = x.wrapping_sub(run.2);
                    r(Point { x, y: bounds.t() }, run.1);
                    if run.0 {
                        x = x.wrapping_sub(1);
                    }
                }
                Range1d { start: x, end: bounds.r() }
            },
            Some(HAlign::Center) => {
                let line_width = line.iter().map(|x| {
                    let space_width = if x.0 { 1i16 } else { 0 };
                    space_width.wrapping_add(x.2)
                }).fold(0i16, |s, w| s.wrapping_add(w));
                let start = bounds.l().wrapping_add((((bounds.w() as u16) - (line_width as u16)) / 2) as i16);
                let mut x = start;
                for run in line {
                    if run.0 {
                        x = x.wrapping_add(1);
                    }
                    r(Point { x, y: bounds.t() }, run.1);
                    x = x.wrapping_add(run.2);
                }
                Range1d { start, end: x }
            },
        }
    }

    fn graphemes(word: &str) -> impl Iterator<Item=(&str, i16)> {
        word.char_indices().peekable().batching(|i| {
            let (start, w) = loop {
                let Some(c) = i.next() else { return None; };
                let Some(w) = c.1.width() else { continue; };
                if w == 0 { continue; }
                break (c, w as u16 as i16);
            };
            let mut end = start;
            loop {
                let Some(&c) = i.peek() else { break; };
                let Some(w) = c.1.width() else { continue; };
                if w != 0 { break; }
                i.next();
                end = c;
            }
            Some((&word[start.0 .. end.0 + end.1.len_utf8()], w))
        })
    }

    fn wrap<'a>(
        bounds: Rect,
        runs: impl Iterator<Item=(bool, &'a str, i16)>
    ) -> impl Iterator<Item=Vec<(bool, &'a str, i16)>> {
        let mut line = Vec::new();
        let mut p = bounds.tl;
        runs.batching(move |i| {
            if (p.y - bounds.t()) as u16 >= bounds.h() as u16 { return None; }
            loop {
                let Some((space, run, run_width)) = i.next() else {
                    let res = replace(&mut line, Vec::new());
                    break if res.is_empty() { None } else { Some(res) };
                };
                let space_width = if space { 1i16 } else { 0 };
                if
                       run_width as u16 > (bounds.r() - p.x) as u16
                    || space_width.wrapping_add(run_width) as u16 > (bounds.r() - p.x) as u16
                {
                    let res = replace(&mut line, Vec::new());
                    p.x = bounds.l().wrapping_add(run_width);
                    p.y = p.y.wrapping_add(1);
                    line.push((false, run, run_width));
                    debug_assert!(!res.is_empty());
                    break Some(res);
                } else {
                    line.push((space, run, run_width));
                    p.x = p.x.wrapping_add(space_width.wrapping_add(run_width));
                }
            }
        })
    }
}

use text_renderer::render_text;

import! { pub static_text:
    use [view crate::view];
    use tvxaml_screen_base::{Fg, Bg};
}

pub use text_renderer::TextWrapping;

struct StaticTextData {
    text: Rc<String>,
    text_align: TextAlign,
    text_wrapping: TextWrapping,
    color: (Fg, Bg),
}

#[class_unsafe(inherits_View)]
pub struct StaticText {
    data: RefCell<StaticTextData>,
    #[non_virt]
    text: fn() -> Rc<String>,
    #[non_virt]
    set_text: fn(value: Rc<String>),
    #[non_virt]
    text_align: fn() -> TextAlign,
    #[non_virt]
    set_text_align: fn(value: TextAlign),
    #[non_virt]
    text_wrapping: fn() -> TextWrapping,
    #[non_virt]
    set_text_wrapping: fn(value: TextWrapping),
    #[non_virt]
    color: fn() -> (Fg, Bg),
    #[non_virt]
    set_color: fn(value: (Fg, Bg)),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[over]
    render: (),
}

impl StaticText {
    pub fn new() -> Rc<dyn IsStaticText> {
        Rc::new(unsafe { Self::new_raw(STATIC_TEXT_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        StaticText {
            view: unsafe { View::new_raw(vtable) },
            data: RefCell::new(StaticTextData {
                text: Rc::new(String::new()),
                text_align: TextAlign::Left,
                text_wrapping: TextWrapping::NoWrap,
                color: (Fg::White, Bg::Blue),
            }),
        }
    }

    pub fn text_impl(this: &Rc<dyn IsStaticText>) -> Rc<String> {
        this.static_text().data.borrow().text.clone()
    }

    pub fn set_text_impl(this: &Rc<dyn IsStaticText>, value: Rc<String>) {
        this.static_text().data.borrow_mut().text = value;
        this.invalidate_measure();
        this.invalidate_render();
    }

    pub fn text_align_impl(this: &Rc<dyn IsStaticText>) -> TextAlign {
        this.static_text().data.borrow().text_align
    }

    pub fn set_text_align_impl(this: &Rc<dyn IsStaticText>, value: TextAlign) {
        this.static_text().data.borrow_mut().text_align = value;
        this.invalidate_render();
    }

    pub fn text_wrapping_impl(this: &Rc<dyn IsStaticText>) -> TextWrapping {
        this.static_text().data.borrow().text_wrapping
    }

    pub fn set_text_wrapping_impl(this: &Rc<dyn IsStaticText>, value: TextWrapping) {
        this.static_text().data.borrow_mut().text_wrapping = value;
        this.invalidate_measure();
        this.invalidate_render();
    }

    pub fn color_impl(this: &Rc<dyn IsStaticText>) -> (Fg, Bg) {
        this.static_text().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsStaticText>, value: (Fg, Bg)) {
        this.static_text().data.borrow_mut().color = value;
        this.invalidate_render();
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsStaticText> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.static_text().data.borrow();
        render_text(
            |_, _| { },
            Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: w.unwrap_or(-1), y: h.unwrap_or(-1) } },
            data.text_align.into(),
            data.text_wrapping,
            &data.text
        ).size
    }

    pub fn arrange_override_impl(_this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        bounds.size
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let bounds = this.inner_render_bounds();
        let this: Rc<dyn IsStaticText> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.static_text().data.borrow();
        rp.fill_bg(data.color);
        render_text(
            |p, s| rp.text(p, data.color, s),
            bounds,
            data.text_align.into(),
            data.text_wrapping,
            &data.text
        );
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum TextAlign { Left, Center, Right, Justify }

impl From<Option<HAlign>> for TextAlign {
    fn from(value: Option<HAlign>) -> Self {
        match value {
            Some(HAlign::Left) => TextAlign::Left,
            Some(HAlign::Center) => TextAlign::Center,
            Some(HAlign::Right) => TextAlign::Right,
            None => TextAlign::Justify,
        }
    }
}

impl From<TextAlign> for Option<HAlign> {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Left => Some(HAlign::Left),
            TextAlign::Center => Some(HAlign::Center),
            TextAlign::Right => Some(HAlign::Right),
            TextAlign::Justify => None,
        }
    }
}

#[macro_export]
macro_rules! static_text_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::view_template! {
            $(#[$attr])*
            $vis struct $name {
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub text: Option<String>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub text_align: Option<$crate::static_text::TextAlign>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub text_wrapping: Option<$crate::static_text::TextWrapping>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color: Option<($crate::tvxaml_screen_base_Fg, $crate::tvxaml_screen_base_Bg)>,
                $($(
                    $(#[$field_attr])*
                    $field_vis $field_name : $field_ty
                ),+)?
            }
        }
    };
}
 
#[macro_export]
macro_rules! static_text_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
        {
            use $crate::static_text::StaticTextExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::static_text::IsStaticText>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.text.as_ref().map(|x| obj.set_text($crate::alloc_rc_Rc::new(x.clone())));
            $this.text_align.map(|x| obj.set_text_align(x));
            $this.text_wrapping.map(|x| obj.set_text_wrapping(x));
            $this.color.map(|x| obj.set_color(x));
        }
    };
}

static_text_template! {
    #[derive(Serialize, Deserialize, Default)]
    #[serde(rename="StaticText")]
    pub struct StaticTextTemplate { }
}

#[typetag::serde(name="StaticText")]
impl Template for StaticTextTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = StaticText::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        static_text_apply_template!(this, instance, names);
    }
}
