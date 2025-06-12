use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use std::ptr::addr_eq;
use crate::app::Timer;
use crate::base::{label_width, HAlign, VAlign};
use crate::event_handler::EventHandler;
use crate::template::{Template, NameResolver};

import! { pub button:
    use [view crate::view];
    use crate::base::{Fg, Bg};
}

struct ButtonData {
    text: Rc<String>,
    color: (Fg, Bg),
    color_hotkey: (Fg, Bg),
    color_disabled: (Fg, Bg),
    color_hotkey_disabled: (Fg, Bg),
    color_focused: (Fg, Bg),
    color_hotkey_focused: (Fg, Bg),
    color_pressed: (Fg, Bg),
    click_handler: EventHandler<Option<Box<dyn FnMut()>>>,
    press_handler: EventHandler<Option<Box<dyn FnMut()>>>,
    release_handler: EventHandler<Option<Box<dyn FnMut()>>>,
    release_timer: Option<Timer>,
}

#[class_unsafe(inherits_View)]
pub struct Button {
    data: RefCell<ButtonData>,
    #[non_virt]
    text: fn() -> Rc<String>,
    #[non_virt]
    set_text: fn(value: Rc<String>),
    #[non_virt]
    color: fn() -> (Fg, Bg),
    #[non_virt]
    set_color: fn(value: (Fg, Bg)),
    #[non_virt]
    color_hotkey: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_hotkey: fn(value: (Fg, Bg)),
    #[non_virt]
    color_disabled: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_disabled: fn(value: (Fg, Bg)),
    #[non_virt]
    color_hotkey_disabled: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_hotkey_disabled: fn(value: (Fg, Bg)),
    #[non_virt]
    color_focused: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_focused: fn(value: (Fg, Bg)),
    #[non_virt]
    color_hotkey_focused: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_hotkey_focused: fn(value: (Fg, Bg)),
    #[non_virt]
    color_pressed: fn() -> (Fg, Bg),
    #[non_virt]
    set_color_pressed: fn(value: (Fg, Bg)),
    #[over]
    is_enabled_changed: (),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[over]
    render: (),
    #[over]
    is_focused_changed: (),
    #[non_virt]
    is_pressed: fn() -> bool,
    #[non_virt]
    handle_click: fn(handler: Option<Box<dyn FnMut()>>),
    #[non_virt]
    handle_press: fn(handler: Option<Box<dyn FnMut()>>),
    #[non_virt]
    handle_release: fn(handler: Option<Box<dyn FnMut()>>),
    #[over]
    key: (),
    #[over]
    _init: (),
    #[over]
    _detach_from_app: (),
}

impl Button {
    pub fn new() -> Rc<dyn IsButton> {
        let res: Rc<dyn IsButton> = Rc::new(unsafe { Self::new_raw(BUTTON_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Button {
            view: unsafe { View::new_raw(vtable) },
            data: RefCell::new(ButtonData {
                text: Rc::new(String::new()),
                color: (Fg::LightGray, Bg::None),
                color_hotkey: (Fg::White, Bg::None),
                color_disabled: (Fg::DarkGray, Bg::None),
                color_hotkey_disabled: (Fg::LightGray, Bg::None),
                color_focused: (Fg::LightGray, Bg::Blue),
                color_hotkey_focused: (Fg::White, Bg::Blue),
                color_pressed: (Fg::Blue, Bg::None),
                click_handler: Default::default(),
                press_handler: Default::default(),
                release_handler: Default::default(),
                release_timer: None,
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

    pub fn text_impl(this: &Rc<dyn IsButton>) -> Rc<String> {
        this.button().data.borrow().text.clone()
    }

    pub fn set_text_impl(this: &Rc<dyn IsButton>, value: Rc<String>) {
        {
            let mut data = this.button().data.borrow_mut();
            if addr_eq(Rc::as_ptr(&data.text), Rc::as_ptr(&value)) { return; }
            data.text = value;
        }
        this.invalidate_measure();
        this.invalidate_render();
    }

    pub fn color_impl(this: &Rc<dyn IsButton>) -> (Fg, Bg) {
        this.button().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsButton>, value: (Fg, Bg)) {
        {
            let mut data = this.button().data.borrow_mut();
            if data.color == value { return; }
            data.color = value;
        }
        this.invalidate_render();
    }

    pub fn color_hotkey_impl(this: &Rc<dyn IsButton>) -> (Fg, Bg) {
        this.button().data.borrow().color_hotkey
    }

    pub fn set_color_hotkey_impl(this: &Rc<dyn IsButton>, value: (Fg, Bg)) {
        {
            let mut data = this.button().data.borrow_mut();
            if data.color_hotkey == value { return; }
            data.color_hotkey = value;
        }
        this.invalidate_render();
    }

    pub fn color_disabled_impl(this: &Rc<dyn IsButton>) -> (Fg, Bg) {
        this.button().data.borrow().color_disabled
    }

    pub fn set_color_disabled_impl(this: &Rc<dyn IsButton>, value: (Fg, Bg)) {
        {
            let mut data = this.button().data.borrow_mut();
            if data.color_disabled == value { return; }
            data.color_disabled = value;
        }
        this.invalidate_render();
    }

    pub fn color_hotkey_disabled_impl(this: &Rc<dyn IsButton>) -> (Fg, Bg) {
        this.button().data.borrow().color_hotkey_disabled
    }

    pub fn set_color_hotkey_disabled_impl(this: &Rc<dyn IsButton>, value: (Fg, Bg)) {
        {
            let mut data = this.button().data.borrow_mut();
            if data.color_hotkey_disabled == value { return; }
            data.color_hotkey_disabled = value;
        }
        this.invalidate_render();
    }

    pub fn color_focused_impl(this: &Rc<dyn IsButton>) -> (Fg, Bg) {
        this.button().data.borrow().color_focused
    }

    pub fn set_color_focused_impl(this: &Rc<dyn IsButton>, value: (Fg, Bg)) {
        {
            let mut data = this.button().data.borrow_mut();
            if data.color_focused == value { return; }
            data.color_focused = value;
        }
        this.invalidate_render();
    }

    pub fn color_hotkey_focused_impl(this: &Rc<dyn IsButton>) -> (Fg, Bg) {
        this.button().data.borrow().color_hotkey_focused
    }

    pub fn set_color_hotkey_focused_impl(this: &Rc<dyn IsButton>, value: (Fg, Bg)) {
        {
            let mut data = this.button().data.borrow_mut();
            if data.color_hotkey_focused == value { return; }
            data.color_hotkey_focused = value;
        }
        this.invalidate_render();
    }

    pub fn color_pressed_impl(this: &Rc<dyn IsButton>) -> (Fg, Bg) {
        this.button().data.borrow().color_pressed
    }

    pub fn set_color_pressed_impl(this: &Rc<dyn IsButton>, value: (Fg, Bg)) {
        {
            let mut data = this.button().data.borrow_mut();
            if data.color_pressed == value { return; }
            data.color_pressed = value;
        }
        this.invalidate_render();
    }

    pub fn is_pressed_impl(this: &Rc<dyn IsButton>) -> bool {
        this.button().data.borrow().release_timer.is_some()
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, _w: Option<i16>, _h: Option<i16>) -> Vector {
        let this: Rc<dyn IsButton> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.button().data.borrow();
        Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
    }

    pub fn arrange_override_impl(_this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        Vector { x: bounds.w(), y: 1 }
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let bounds = this.inner_render_bounds();
        let is_enabled = this.is_enabled();
        let is_focused = this.is_focused(None);
        let is_focused_primary = this.is_focused(Some(true));
        let this: Rc<dyn IsButton> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.button().data.borrow();
        let is_pressed = data.release_timer.is_some();
        let (color, color_hotkey) = if is_pressed {
            (data.color_pressed, data.color_pressed)
        } else {
            match (is_enabled, is_focused) {
                (true, false) => (data.color, data.color_hotkey),
                (true, true) => (data.color_focused, data.color_hotkey_focused),
                (false, false) => (data.color_disabled, data.color_hotkey_disabled),
                (false, true) => (
                    (data.color_disabled.0, data.color_focused.1),
                    (data.color_hotkey_disabled.0, data.color_hotkey_focused.1)
                )
            }
        };
        rp.fill_bg(color);
        let text_tl = Thickness::align(
            Vector { x: label_width(&data.text), y: 1 },
            bounds.size,
            HAlign::Center,
            VAlign::Top
        ).shrink_rect(bounds).tl;
        rp.label(text_tl, color, color_hotkey, &data.text);
        if !is_pressed {
            rp.text(Point { x: 0, y: 0 }, color, "[");
            rp.text(Point { x: bounds.r_inner(), y: 0 }, color, "]");
        }
        if is_focused_primary {
            rp.cursor(text_tl);
        }
    }

    pub fn is_focused_changed_impl(this: &Rc<dyn IsView>, primary_focus: bool) {
        View::is_focused_changed_impl(this, primary_focus);
        this.invalidate_render();
    }

    pub fn handle_click_impl(this: &Rc<dyn IsButton>, handler: Option<Box<dyn FnMut()>>) {
        this.button().data.borrow_mut().click_handler.set(handler);
    }

    pub fn handle_press_impl(this: &Rc<dyn IsButton>, handler: Option<Box<dyn FnMut()>>) {
        this.button().data.borrow_mut().press_handler.set(handler);
    }

    pub fn handle_release_impl(this: &Rc<dyn IsButton>, handler: Option<Box<dyn FnMut()>>) {
        this.button().data.borrow_mut().release_handler.set(handler);
    }

    fn click(this: &Rc<dyn IsButton>) {
        let app = this.app().unwrap();
        let release_timer = {
            let this = Rc::downgrade(this);
            Timer::new(&app, 100, Box::new(move || {
                let this = this.upgrade().unwrap();
                this.button().data.borrow_mut().release_timer = None;
                this.invalidate_render();
                let mut invoke = this.button().data.borrow_mut().release_handler.begin_invoke();
                invoke.as_mut().map(|x| x());
                this.button().data.borrow_mut().release_handler.end_invoke(invoke);
            }))
        };
        if let Some(old_timer) = this.button().data.borrow_mut().release_timer.replace(release_timer) {
            old_timer.drop_timer(&app);
        }
        this.invalidate_render();
        let mut invoke = this.button().data.borrow_mut().press_handler.begin_invoke();
        invoke.as_mut().map(|x| x());
        this.button().data.borrow_mut().press_handler.end_invoke(invoke);
        let mut invoke = this.button().data.borrow_mut().click_handler.begin_invoke();
        invoke.as_mut().map(|x| x());
        this.button().data.borrow_mut().click_handler.end_invoke(invoke);
    }

    pub fn _detach_from_app_impl(this: &Rc<dyn IsView>) {
        {
            let this: Rc<dyn IsButton> = dyn_cast_rc(this.clone()).unwrap();
            if let Some(old_timer) = this.button().data.borrow_mut().release_timer.take() {
                let app = this.app().unwrap();
                old_timer.drop_timer(&app);
            }
        }
        View::_detach_from_app_impl(this);
    }

    pub fn key_impl(this: &Rc<dyn IsView>, key: Key, original_source: &Rc<dyn IsView>) -> bool {
        if key == Key::Enter {
            Self::click(&dyn_cast_rc(this.clone()).unwrap());
            return true;
        }
        View::key_impl(this, key, original_source)
    }
}

#[macro_export]
macro_rules! button_template {
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
                pub text: Option<String>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub is_checked: Option<bool>,
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
                pub color_hotkey_focused: Option<($crate::base::Fg, $crate::base::Bg)>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color_disabled: Option<($crate::base::Fg, $crate::base::Bg)>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color_hotkey_disabled: Option<($crate::base::Fg, $crate::base::Bg)>,
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub color_pressed: Option<($crate::base::Fg, $crate::base::Bg)>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}
 
#[macro_export]
macro_rules! button_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
        {
            use $crate::button::ButtonExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::button::IsButton>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.text.as_ref().map(|x| obj.set_text($crate::alloc_rc_Rc::new(x.clone())));
            $this.color.map(|x| obj.set_color(x));
            $this.color_hotkey.map(|x| obj.set_color_hotkey(x));
            $this.color_focused.map(|x| obj.set_color_focused(x));
            $this.color_hotkey_focused.map(|x| obj.set_color_hotkey_focused(x));
            $this.color_disabled.map(|x| obj.set_color_disabled(x));
            $this.color_hotkey_disabled.map(|x| obj.set_color_hotkey_disabled(x));
            $this.color_pressed.map(|x| obj.set_color_pressed(x));
        }
    };
}

button_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Button@Text")]
    pub struct ButtonTemplate in template { }
}

#[typetag::serde(name="Button")]
impl Template for ButtonTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Button::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        button_apply_template!(this, instance, names);
    }
}
