use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use std::mem::replace;
use std::ops::RangeInclusive;
use std::rc::{self};
use crate::base::{text_width, VAlign, graphemes, HAlign};
use crate::event_handler::EventHandler;
use crate::template::{Template, NameResolver};

import! { pub input_line:
    use [view crate::view];
    use crate::base::{Fg, Bg};
}

struct InputLineData {
    text: String,
    color: (Fg, Bg),
    color_focused: (Fg, Bg),
    color_disabled: (Fg, Bg),
    view: Option<RangeInclusive<usize>>,
    cursor: usize,
    width: i16,
    is_numeric: bool,
    text_change_handler: EventHandler<Option<Box<dyn FnMut()>>>,
}

#[derive(Clone)]
pub struct TextBuf {
    owner: rc::Weak<dyn IsInputLine>,
}

impl ToString for TextBuf {
    fn to_string(&self) -> String {
        self.owner.upgrade().unwrap().input_line().data.borrow().text.clone()
    }
}

impl TextBuf {
    pub fn change<T>(&self, f: impl FnOnce(&mut String) -> T) -> T {
        let res = {
            let owner = self.owner.upgrade().unwrap();
            let mut data = owner.input_line().data.borrow_mut();
            f(&mut data.text)
        };
        let owner = self.owner.upgrade().unwrap();
        InputLine::reset_view(&owner);
        owner.text_changed();
        res
    }

    pub fn set(&self, s: String) {
        self.change(|x| *x = s);
    }

    pub fn replace(&self, s: String) -> String {
        self.change(|x| replace(x, s))
    }
}

#[class_unsafe(inherits_View)]
pub struct InputLine {
    data: RefCell<InputLineData>,
    #[non_virt]
    text: fn() -> TextBuf,
    #[non_virt]
    is_numeric: fn() -> bool,
    #[non_virt]
    set_is_numeric: fn(value: bool),
    #[non_virt]
    color: fn() -> (Fg, Bg),
    #[non_virt]
    set_color: fn(value: (Fg, Bg)),
    #[non_virt]
    color_focused: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_focused: fn(value: (Fg, Bg)),
    #[non_virt]
    color_disabled: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_disabled: fn(value: (Fg, Bg)),
    #[over]
    is_enabled_changed: (),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[over]
    render: (),
    #[over]
    _init: (),
    #[over]
    is_focused_changed: (),
    #[non_virt]
    handle_text_change: fn(handler: Option<Box<dyn FnMut()>>),
    #[virt]
    text_changed: fn(),
    #[over]
    key: (),
}

impl InputLine {
    pub fn new() -> Rc<dyn IsInputLine> {
        let res: Rc<dyn IsInputLine> = Rc::new(unsafe { Self::new_raw(INPUT_LINE_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        InputLine {
            view: unsafe { View::new_raw(vtable) },
            data: RefCell::new(InputLineData {
                text: String::new(),
                color: (Fg::LightGray, Bg::Blue),
                color_focused: (Fg::LightGray, Bg::Blue),
                color_disabled: (Fg::DarkGray, Bg::Blue),
                text_change_handler: Default::default(),
                cursor: 0,
                view: None,
                is_numeric: false,
                width: 0,
            }),
        }
    }

    pub fn _init_impl(this: &Rc<dyn IsView>) {
        View::_init_impl(this);
        this.set_allow_focus(true);
    }

    pub fn is_enabled_changed_impl(this: &Rc<dyn IsView>) {
        View::is_enabled_changed_impl(this);
        this.invalidate_render();
    }

    pub fn text_impl(this: &Rc<dyn IsInputLine>) -> TextBuf {
        TextBuf { owner: Rc::downgrade(this) }
    }

    pub fn text_changed_impl(this: &Rc<dyn IsInputLine>) {
        let mut invoke = this.input_line().data.borrow_mut().text_change_handler.begin_invoke();
        invoke.as_mut().map(|x| x());
        this.input_line().data.borrow_mut().text_change_handler.end_invoke(invoke);
    }

    pub fn is_numeric_impl(this: &Rc<dyn IsInputLine>) -> bool {
        this.input_line().data.borrow().is_numeric
    }

    pub fn set_is_numeric_impl(this: &Rc<dyn IsInputLine>, value: bool) {
        {
            let mut data = this.input_line().data.borrow_mut();
            if data.is_numeric == value { return; }
            data.is_numeric = value;
        }
        Self::reset_view(this);
    }

    pub fn color_impl(this: &Rc<dyn IsInputLine>) -> (Fg, Bg) {
        this.input_line().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsInputLine>, value: (Fg, Bg)) {
        {
            let mut data = this.input_line().data.borrow_mut();
            if data.color == value { return; }
            data.color = value;
        }
        this.invalidate_render();
    }

    pub fn color_focused_impl(this: &Rc<dyn IsInputLine>) -> (Fg, Bg) {
        this.input_line().data.borrow().color_focused
    }

    pub fn set_color_focused_impl(this: &Rc<dyn IsInputLine>, value: (Fg, Bg)) {
        {
            let mut data = this.input_line().data.borrow_mut();
            if data.color_focused == value { return; }
            data.color_focused = value;
        }
        this.invalidate_render();
    }

    pub fn color_disabled_impl(this: &Rc<dyn IsInputLine>) -> (Fg, Bg) {
        this.input_line().data.borrow().color_disabled
    }

    pub fn set_color_disabled_impl(this: &Rc<dyn IsInputLine>, value: (Fg, Bg)) {
        {
            let mut data = this.input_line().data.borrow_mut();
            if data.color_disabled == value { return; }
            data.color_disabled = value;
        }
        this.invalidate_render();
    }

    pub fn measure_override_impl(_this: &Rc<dyn IsView>, w: Option<i16>, _h: Option<i16>) -> Vector {
        Vector { x: w.unwrap_or(1), y: 1 }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsInputLine> = dyn_cast_rc(this.clone()).unwrap();
        this.input_line().data.borrow_mut().width = Thickness::new(1, 0, 1, 0).shrink_rect(bounds).w();
        Self::reset_view(&this);
        Vector { x: bounds.w(), y: 1 }
    }

    fn reset_view(this: &Rc<dyn IsInputLine>) {
        let is_focused = this.is_focused(None);
        let (is_numeric, text_len) = {
            let mut data = this.input_line().data.borrow_mut();
            data.cursor = data.text.len();
            (data.is_numeric, data.text.len())
        };
        if is_focused {
            Self::calc_view_start(this, text_len);
        } else if is_numeric {
            if text_len == 0 {
                this.input_line().data.borrow_mut().view = None;
            } else {
                Self::calc_view_start(this, text_len - 1);
            }
        } else {
            Self::calc_view_end(this, 0);
        }
        this.invalidate_render();
    }

    fn calc_view_start(this: &Rc<dyn IsInputLine>, view_end: usize) {
        let mut data = this.input_line().data.borrow_mut();
        let with_end = view_end == data.text.len();
        let text = if with_end { &data.text[.. view_end] } else { &data.text[..= view_end] };
        let view = graphemes(text)
            .rev()
            .scan(if with_end { 1i16 } else { 0i16 }, |w, (g, g_w)| {
                *w = (*w).wrapping_add(g_w);
                if *w > data.width { None } else { Some(g) }
            })
            .last().map(|x| x.start ..= view_end);
        data.view = view;
    }

    fn calc_view_end(this: &Rc<dyn IsInputLine>, view_start: usize) {
        let mut data = this.input_line().data.borrow_mut();
        let view = graphemes(&data.text[view_start .. ])
            .map(|(g, g_w)| (g.start ..= g.end - 1, g_w))
            .scan(0i16, |w, (g, g_w)| {
                *w = (*w).wrapping_add(g_w);
                if *w > data.width { None } else { Some(g) }
            })
            .last().map(|x| view_start ..= view_start + *x.end());
        data.view = view;
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let bounds = this.inner_render_bounds();
        let is_enabled = this.is_enabled();
        let is_focused = this.is_focused(None);
        let is_focused_primary = this.is_focused(Some(true));
        let this: Rc<dyn IsInputLine> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.input_line().data.borrow();
        let color = match (is_enabled, is_focused) {
            (true, true) => data.color_focused,
            (true, false) => data.color,
            (false, true) => (data.color_disabled.0, data.color_focused.1),
            (false, false) => data.color_disabled
        };
        rp.fill_bg(color);
        if let Some(view) = data.view.clone() {
            let show_text_end = view.contains(&data.text.len());
            let text = if show_text_end {
                &data.text[*view.start() .. *view.end()]
            } else {
                &data.text[view.clone()]
            };
            let align = Thickness::align(
                Vector { x: text_width(text).wrapping_add(if show_text_end { 1 } else { 0 }), y: 1 },
                Vector { x: data.width, y: 1 },
                if data.is_numeric { HAlign::Right } else { HAlign::Left },
                VAlign::Top
            );
            let text_start = align.shrink_rect(Thickness::new(1, 0, 1, 0).shrink_rect(bounds)).tl;
            rp.text(text_start, color, text);
            if *view.start() > 0 {
                rp.text(Point { x: 0, y: 0 }, color, "◄");
            }
            if !show_text_end && *view.end() < data.text.len() - 1 {
                rp.text(bounds.tr_inner(), color, "►");
            }
            if is_focused_primary && view.contains(&data.cursor) {
                let cursor_x = text_width(&data.text[*view.start() .. data.cursor]);
                rp.cursor(text_start.offset(Vector { x: cursor_x, y: 0 }));
            }
        }
    }

    pub fn is_focused_changed_impl(this: &Rc<dyn IsView>, primary_focus: bool) {
        View::is_focused_changed_impl(this, primary_focus);
        let this: Rc<dyn IsInputLine> = dyn_cast_rc(this.clone()).unwrap();
        Self::reset_view(&this);
    }

    pub fn handle_text_change_impl(this: &Rc<dyn IsInputLine>, handler: Option<Box<dyn FnMut()>>) {
        this.input_line().data.borrow_mut().text_change_handler.set(handler);
    }

    pub fn key_impl(this: &Rc<dyn IsView>, key: Key, original_source: &Rc<dyn IsView>) -> bool {
        match key {
            Key::Left => {
                let this: Rc<dyn IsInputLine> = dyn_cast_rc(this.clone()).unwrap();
                let view_start = {
                    let mut data = this.input_line().data.borrow_mut();
                    if let Some((g, _)) = graphemes(&data.text[.. data.cursor]).next_back() {
                        data.cursor = g.start;
                        if let Some(view) = data.view.clone() && view.contains(&data.cursor) {
                            Some(None)
                        } else {
                            Some(Some(data.cursor))
                        }
                    } else {
                        None
                    }
                };
                if let Some(view_start) = view_start {
                    view_start.map(|x| Self::calc_view_end(&this, x));
                    this.invalidate_render();
                    return true;
                } 
            },
            Key::Right => {
                let this: Rc<dyn IsInputLine> = dyn_cast_rc(this.clone()).unwrap();
                let view_end = {
                    let mut data = this.input_line().data.borrow_mut();
                    if let Some((g, _)) = graphemes(&data.text[data.cursor ..]).next() {
                        data.cursor += g.end;
                        if let Some(view) = data.view.clone() && view.contains(&data.cursor) {
                            Some(None)
                        } else {
                            Some(Some(data.cursor))
                        }
                    } else {
                        None
                    }
                };
                if let Some(view_end) = view_end {
                    view_end.map(|x| Self::calc_view_start(&this, x));
                    this.invalidate_render();
                    return true;
                } 
            }, 
            Key::Home => {
                let this: Rc<dyn IsInputLine> = dyn_cast_rc(this.clone()).unwrap();
                this.input_line().data.borrow_mut().cursor = 0;
                Self::calc_view_end(&this, 0);
                this.invalidate_render();
                return true;
            },
            Key::End => {
                let this: Rc<dyn IsInputLine> = dyn_cast_rc(this.clone()).unwrap();
                let text_len = {
                    let mut data = this.input_line().data.borrow_mut();
                    data.cursor = data.text.len();
                    data.text.len()
                };
                Self::calc_view_start(&this, text_len);
                this.invalidate_render();
                return true;
            },
            _ => { },
        }
        View::key_impl(this, key, original_source)
    }
}

#[macro_export]
macro_rules! input_line_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident in $mod:ident {
            $(use $path:path as $import:ident;)*

            $($(
                $(#[$field_attr:meta])*
                pub $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::view_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub is_numeric: Option<bool>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color: Option<($crate::base::Fg, $crate::base::Bg)>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color_hotkey: Option<($crate::base::Fg, $crate::base::Bg)>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color_focused: Option<($crate::base::Fg, $crate::base::Bg)>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color_disabled: Option<($crate::base::Fg, $crate::base::Bg)>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}
 
#[macro_export]
macro_rules! input_line_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
        {
            use $crate::input_line::InputLineExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::input_line::IsInputLine>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.is_numeric.map(|x| obj.set_is_numeric(x));
            $this.color.map(|x| obj.set_color(x));
            $this.color_focused.map(|x| obj.set_color_focused(x));
            $this.color_disabled.map(|x| obj.set_color_disabled(x));
        }
    };
}

input_line_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="InputLine")]
    pub struct InputLineTemplate in template { }
}

#[typetag::serde(name="InputLine")]
impl Template for InputLineTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        InputLine::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        input_line_apply_template!(this, instance, names);
    }
}
