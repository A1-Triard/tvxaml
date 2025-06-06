use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;
use crate::render_port::label_width;
use crate::template::{Template, Names};

import! { pub check_box:
    use [view crate::view];
    use tvxaml_screen_base::{Fg, Bg};
}

struct CheckBoxData {
    text: Rc<String>,
    is_checked: bool,
    color: (Fg, Bg),
    color_hotkey: (Fg, Bg),
    color_disabled: (Fg, Bg),
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
}

impl CheckBox {
    pub fn new() -> Rc<dyn IsCheckBox> {
        Rc::new(unsafe { Self::new_raw(CHECK_BOX_VTABLE.as_ptr()) })
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
            }),
        }
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
        this.check_box().data.borrow_mut().is_checked = value;
        this.invalidate_render();
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
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename="CheckBox")]
pub struct CheckBoxTemplate {
    #[serde(flatten)]
    pub view: ViewTemplate,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub text: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub is_checked: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub color: Option<(Fg, Bg)>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub color_hotkey: Option<(Fg, Bg)>,
    #[serde(default)]
    #[serde(skip_serializing_if="Option::is_none")]
    pub color_disabled: Option<(Fg, Bg)>,
}

#[typetag::serde(name="CheckBox")]
impl Template for CheckBoxTemplate {
    fn is_name_scope(&self) -> bool {
        self.view.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.view.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = CheckBox::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        self.view.apply(instance, names);
        let obj: Rc<dyn IsCheckBox> = dyn_cast_rc(instance.clone()).unwrap();
        self.text.as_ref().map(|x| obj.set_text(Rc::new(x.clone())));
        self.is_checked.map(|x| obj.set_is_checked(x));
        self.color.map(|x| obj.set_color(x));
        self.color_hotkey.map(|x| obj.set_color_hotkey(x));
        self.color_disabled.map(|x| obj.set_color_disabled(x));
    }
}
