use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::cell::RefCell;
use crate::template::{Template, Names};

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
        Wrap,
        WrapWithOverflow,
    }

    pub fn render_text(
        mut r: impl FnMut(Point, &str),
        bounds: Rect,
        align: Option<HAlign>,
        wrapping: Option<TextWrapping>,
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
        wrapping: Option<TextWrapping>,
        text: &str,
    ) -> Rect {
        if text.is_empty() {
            return Rect { tl: bounds.tl, size: Vector { x: 0, y: 1 } };
        }
        if let Some(wrapping) = wrapping {
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
    text_align: Option<HAlign>,
    text_wrapping: Option<TextWrapping>,
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
    text_align: fn() -> Option<HAlign>,
    #[non_virt]
    set_text_align: fn(value: Option<HAlign>),
    #[non_virt]
    text_wrapping: fn() -> Option<TextWrapping>,
    #[non_virt]
    set_text_wrapping: fn(value: Option<TextWrapping>),
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
                text_align: Some(HAlign::Left),
                text_wrapping: None,
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

    pub fn text_align_impl(this: &Rc<dyn IsStaticText>) -> Option<HAlign> {
        this.static_text().data.borrow().text_align
    }

    pub fn set_text_align_impl(this: &Rc<dyn IsStaticText>, value: Option<HAlign>) {
        this.static_text().data.borrow_mut().text_align = value;
        this.invalidate_render();
    }

    pub fn text_wrapping_impl(this: &Rc<dyn IsStaticText>) -> Option<TextWrapping> {
        this.static_text().data.borrow().text_wrapping
    }

    pub fn set_text_wrapping_impl(this: &Rc<dyn IsStaticText>, value: Option<TextWrapping>) {
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
            data.text_align,
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
            data.text_align,
            data.text_wrapping,
            &data.text
        );
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
enum TextAlign { Left, Center, Right, Justify }

fn serialize_text_align<S>(value: &Option<Option<HAlign>>, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let surrogate = match value {
        Some(Some(HAlign::Left)) => Some(TextAlign::Left),
        Some(Some(HAlign::Center)) => Some(TextAlign::Center),
        Some(Some(HAlign::Right)) => Some(TextAlign::Right),
        Some(None) => Some(TextAlign::Justify),
        None => None,
    };
    surrogate.serialize(s)
}

fn deserialize_text_align<'de, D>(d: D) -> Result<Option<Option<HAlign>>, D::Error> where D: Deserializer<'de> {
    let surrogate: Option<TextAlign> = Deserialize::deserialize(d)?;
    Ok(match surrogate {
        Some(TextAlign::Left) => Some(Some(HAlign::Left)),
        Some(TextAlign::Center) => Some(Some(HAlign::Center)),
        Some(TextAlign::Right) => Some(Some(HAlign::Right)),
        Some(TextAlign::Justify) => Some(None),
        None => None,
    })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
#[serde(rename="TextWrapping")]
enum TextWrappingSurrogate { Wrap, WrapWithOverflow, NoWrap }

fn serialize_text_wrapping<S>(
    value: &Option<Option<TextWrapping>>,
    s: S
) -> Result<S::Ok, S::Error> where S: Serializer {
    let surrogate = match value {
        Some(Some(TextWrapping::Wrap)) => Some(TextWrappingSurrogate::Wrap),
        Some(Some(TextWrapping::WrapWithOverflow)) => Some(TextWrappingSurrogate::WrapWithOverflow),
        Some(None) => Some(TextWrappingSurrogate::NoWrap),
        None => None,
    };
    surrogate.serialize(s)
}

fn deserialize_text_wrapping<'de, D>(
    d: D
) -> Result<Option<Option<TextWrapping>>, D::Error> where D: Deserializer<'de> {
    let surrogate: Option<TextWrappingSurrogate> = Deserialize::deserialize(d)?;
    Ok(match surrogate {
        Some(TextWrappingSurrogate::Wrap) => Some(Some(TextWrapping::Wrap)),
        Some(TextWrappingSurrogate::WrapWithOverflow) => Some(Some(TextWrapping::WrapWithOverflow)),
        Some(TextWrappingSurrogate::NoWrap) => Some(None),
        None => None,
    })
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename="StaticText")]
pub struct StaticTextTemplate {
    #[serde(flatten)]
    pub view: ViewTemplate,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub text: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(serialize_with="serialize_text_align")]
    #[serde(deserialize_with="deserialize_text_align")]
    pub text_align: Option<Option<HAlign>>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(serialize_with="serialize_text_wrapping")]
    #[serde(deserialize_with="deserialize_text_wrapping")]
    pub text_wrapping: Option<Option<TextWrapping>>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub color: Option<(Fg, Bg)>,
}

#[typetag::serde(name="StaticText")]
impl Template for StaticTextTemplate {
    fn is_name_scope(&self) -> bool {
        self.view.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.view.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = StaticText::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        self.view.apply(instance, names);
        let obj: Rc<dyn IsStaticText> = dyn_cast_rc(instance.clone()).unwrap();
        self.text.as_ref().map(|x| obj.set_text(Rc::new(x.clone())));
        self.text_align.map(|x| obj.set_text_align(x));
        self.text_wrapping.map(|x| obj.set_text_wrapping(x));
        self.color.map(|x| obj.set_color(x));
    }
}
