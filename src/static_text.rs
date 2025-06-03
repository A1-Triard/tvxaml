use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;
use std::cmp::max;
use tvxaml_screen_base::{text_width, Fg, Bg};
use crate::template::Template;

import! { pub static_text:
    use [view crate::view];
}

struct StaticTextData {
    text: Rc<String>,
    text_size: Vector,
    text_align: Option<HAlign>,
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
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[over]
    render: (),
}

impl StaticText {
    pub fn new() -> Rc<dyn TStaticText> {
        Rc::new(unsafe { Self::new_raw(STATIC_TEXT_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        StaticText {
            view: unsafe { View::new_raw(vtable) },
            data: RefCell::new(StaticTextData {
                text: Rc::new(String::new()),
                text_size: Vector::null(),
                text_align: Some(HAlign::Left),
            }),
        }
    }

    pub fn text_impl(this: &Rc<dyn TStaticText>) -> Rc<String> {
        this.static_text().data.borrow().text.clone()
    }

    pub fn set_text_impl(this: &Rc<dyn TStaticText>, value: Rc<String>) {
        {
            let mut data = this.static_text().data.borrow_mut();
            data.text = value;
            let mut width = 0u16;
            let mut height = 0u16;
            for line in data.text.split('\n') {
                width = max(width, text_width(line) as u16);
                height = height.wrapping_add(1);
            }
            data.text_size = Vector { x: width as i16, y: height as i16 };
        }
        this.invalidate_measure();
        this.invalidate_render();
    }

    pub fn text_align_impl(this: &Rc<dyn TStaticText>) -> Option<HAlign> {
        this.static_text().data.borrow().text_align
    }

    pub fn set_text_align_impl(this: &Rc<dyn TStaticText>, value: Option<HAlign>) {
        this.static_text().data.borrow_mut().text_align = value;
        this.invalidate_render();
    }

    pub fn measure_override_impl(this: &Rc<dyn TView>, _w: Option<i16>, _h: Option<i16>) -> Vector {
        let this: Rc<dyn TStaticText> = dyn_cast_rc(this.clone()).unwrap();
        this.static_text().data.borrow().text_size
    }

    pub fn arrange_override_impl(this: &Rc<dyn TView>, size: Vector) -> Vector {
        let this: Rc<dyn TStaticText> = dyn_cast_rc(this.clone()).unwrap();
        Vector { x: size.x, y: this.static_text().data.borrow().text_size.y }
    }

    pub fn render_impl(this: &Rc<dyn TView>, rp: &mut RenderPort) {
        let this: Rc<dyn TStaticText> = dyn_cast_rc(this.clone()).unwrap();
        rp.text(Point { x: 0, y: 0 }, (Fg::Red, Bg::Blue), &this.static_text().data.borrow().text);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename="StaticText")]
pub struct StaticTextTemplate {
    #[serde(flatten)]
    pub view: ViewTemplate,
    pub text: String,
    pub text_align: Option<HAlign>,
}

#[typetag::serde]
impl Template for StaticTextTemplate {
    fn create_instance(&self) -> Rc<dyn TObj> {
        let obj = StaticText::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn TObj>) {
        self.view.apply(instance);
        let obj: Rc<dyn TStaticText> = dyn_cast_rc(instance.clone()).unwrap();
        obj.set_text(Rc::new(self.text.clone()));
        obj.set_text_align(self.text_align);
    }
}
