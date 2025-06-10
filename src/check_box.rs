use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::base::label_width;
use crate::event_handler::EventHandler;
use crate::template::{Template, NameResolver};

import! { pub check_box:
    use [view crate::view];
    use crate::base::{Fg, Bg};
}

struct CheckBoxData {
    text: Rc<String>,
    is_checked: bool,
    color: (Fg, Bg),
    color_hotkey: (Fg, Bg),
    color_disabled: (Fg, Bg),
    toggle_handler: EventHandler<Option<Box<dyn FnMut()>>>,
    click_handler: EventHandler<Option<Box<dyn FnMut()>>>,
}

#[class_unsafe(inherits_View)]
pub struct CheckBox {
    data: RefCell<CheckBoxData>,
    #[non_virt]
    text: fn() -> Rc<String>,
    #[non_virt]
    set_text: fn(value: Rc<String>),
    #[non_virt]
    is_checked: fn() -> bool,
    #[virt]
    is_checked_changed: fn(),
    #[non_virt]
    set_is_checked: fn(value: bool),
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
    #[virt]
    allow_click: fn() -> bool,
    #[non_virt]
    handle_toggle: fn(handler: Option<Box<dyn FnMut()>>),
    #[non_virt]
    handle_click: fn(handler: Option<Box<dyn FnMut()>>),
    #[over]
    key: (),
    #[over]
    _init: (),
}

impl CheckBox {
    pub fn new() -> Rc<dyn IsCheckBox> {
        let res: Rc<dyn IsCheckBox> = Rc::new(unsafe { Self::new_raw(CHECK_BOX_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        CheckBox {
            view: unsafe { View::new_raw(vtable) },
            data: RefCell::new(CheckBoxData {
                text: Rc::new(String::new()),
                is_checked: false,
                color: (Fg::White, Bg::Blue),
                color_hotkey: (Fg::Yellow, Bg::Blue),
                color_disabled: (Fg::DarkGray, Bg::Blue),
                toggle_handler: Default::default(),
                click_handler: Default::default(),
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

    pub fn text_impl(this: &Rc<dyn IsCheckBox>) -> Rc<String> {
        this.check_box().data.borrow().text.clone()
    }

    pub fn set_text_impl(this: &Rc<dyn IsCheckBox>, value: Rc<String>) {
        this.check_box().data.borrow_mut().text = value;
        this.invalidate_measure();
        this.invalidate_render();
    }

    pub fn is_checked_impl(this: &Rc<dyn IsCheckBox>) -> bool {
        this.check_box().data.borrow().is_checked
    }

    pub fn set_is_checked_impl(this: &Rc<dyn IsCheckBox>, value: bool) {
        {
            let mut data = this.check_box().data.borrow_mut();
            if data.is_checked == value { return; }
            data.is_checked = value;
        }
        this.is_checked_changed();
    }

    pub fn is_checked_changed_impl(this: &Rc<dyn IsCheckBox>) {
        this.invalidate_render();
        let mut invoke = this.check_box().data.borrow_mut().toggle_handler.begin_invoke();
        invoke.as_mut().map(|x| x());
        this.check_box().data.borrow_mut().toggle_handler.end_invoke(invoke);
    }

    pub fn color_impl(this: &Rc<dyn IsCheckBox>) -> (Fg, Bg) {
        this.check_box().data.borrow().color
    }

    pub fn set_color_impl(this: &Rc<dyn IsCheckBox>, value: (Fg, Bg)) {
        this.check_box().data.borrow_mut().color = value;
        this.invalidate_render();
    }

    pub fn color_hotkey_impl(this: &Rc<dyn IsCheckBox>) -> (Fg, Bg) {
        this.check_box().data.borrow().color_hotkey
    }

    pub fn set_color_hotkey_impl(this: &Rc<dyn IsCheckBox>, value: (Fg, Bg)) {
        this.check_box().data.borrow_mut().color_hotkey = value;
        this.invalidate_render();
    }

    pub fn color_disabled_impl(this: &Rc<dyn IsCheckBox>) -> (Fg, Bg) {
        this.check_box().data.borrow().color_disabled
    }

    pub fn set_color_disabled_impl(this: &Rc<dyn IsCheckBox>, value: (Fg, Bg)) {
        this.check_box().data.borrow_mut().color_disabled = value;
        this.invalidate_render();
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, _w: Option<i16>, _h: Option<i16>) -> Vector {
        let this: Rc<dyn IsCheckBox> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.check_box().data.borrow();
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
        }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, _bounds: Rect) -> Vector {
        let this: Rc<dyn IsCheckBox> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.check_box().data.borrow();
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
        }
    }

    pub fn render_impl(this: &Rc<dyn IsView>, rp: &mut RenderPort) {
        let is_enabled = this.is_enabled();
        let focused = this.is_focused(Some(true));
        let this: Rc<dyn IsCheckBox> = dyn_cast_rc(this.clone()).unwrap();
        let data = this.check_box().data.borrow();
        let color = if is_enabled { data.color } else { data.color_disabled };
        rp.text(Point { x: 1, y: 0 }, color, if data.is_checked { "x" } else { " " });
        rp.text(Point { x: 0, y: 0 }, color, "[");
        rp.text(Point { x: 2, y: 0 }, color, "]");
        if !data.text.is_empty() {
            rp.text(Point { x: 3, y: 0 }, color, " ");
            rp.label(Point { x: 4, y: 0 }, color, data.color_hotkey, &data.text);
        }
        if focused { rp.cursor(Point { x: 1, y: 0 }); }
    }

    pub fn is_focused_changed_impl(this: &Rc<dyn IsView>, primary_focus: bool) {
        View::is_focused_changed_impl(this, primary_focus);
        if primary_focus {
            this.invalidate_render();
        }
    }

    pub fn handle_toggle_impl(this: &Rc<dyn IsCheckBox>, handler: Option<Box<dyn FnMut()>>) {
        this.check_box().data.borrow_mut().toggle_handler.set(handler);
    }

    pub fn handle_click_impl(this: &Rc<dyn IsCheckBox>, handler: Option<Box<dyn FnMut()>>) {
        this.check_box().data.borrow_mut().click_handler.set(handler);
    }

    pub fn allow_click_impl(_this: &Rc<dyn IsCheckBox>) -> bool {
        true
    }

    fn click(this: &Rc<dyn IsCheckBox>) {
        if !this.allow_click() { return; }
        let checked = !this.is_checked();
        this.set_is_checked(checked);
        let mut invoke = this.check_box().data.borrow_mut().click_handler.begin_invoke();
        invoke.as_mut().map(|x| x());
        this.check_box().data.borrow_mut().click_handler.end_invoke(invoke);
    }

    pub fn key_impl(this: &Rc<dyn IsView>, key: Key, original_source: &Rc<dyn IsView>) -> bool {
        if key == Key::Char(' ') {
            Self::click(&dyn_cast_rc(this.clone()).unwrap());
            return true;
        }
        View::key_impl(this, key, original_source)
    }
}

#[macro_export]
macro_rules! check_box_template {
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
macro_rules! check_box_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
        {
            use $crate::check_box::CheckBoxExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::check_box::IsCheckBox>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.text.as_ref().map(|x| obj.set_text($crate::alloc_rc_Rc::new(x.clone())));
            $this.is_checked.map(|x| obj.set_is_checked(x));
            $this.color.map(|x| obj.set_color(x));
            $this.color_hotkey.map(|x| obj.set_color_hotkey(x));
            $this.color_disabled.map(|x| obj.set_color_disabled(x));
        }
    };
}

check_box_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="CheckBox@Text")]
    pub struct CheckBoxTemplate in template { }
}

#[typetag::serde(name="CheckBox")]
impl Template for CheckBoxTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        CheckBox::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        check_box_apply_template!(this, instance, names);
    }
}
