use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::cell::Cell;
use crate::template::{Template, Names};
use crate::view_vec::ViewVecExt;

import! { pub stack_panel:
    use [panel crate::panel];
}

#[class_unsafe(inherits_Panel)]
pub struct StackPanel {
    vertical: Cell<bool>,
    #[non_virt]
    vertical: fn() -> bool,
    #[non_virt]
    set_vertical: fn(value: bool),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl StackPanel {
    pub fn new() -> Rc<dyn IsStackPanel> {
        Rc::new(unsafe { Self::new_raw(STACK_PANEL_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        StackPanel {
            panel: unsafe { Panel::new_raw(vtable) },
            vertical: Cell::new(true),
        }
    }

    pub fn vertical_impl(this: &Rc<dyn IsStackPanel>) -> bool {
        this.stack_panel().vertical.get()
    }

    pub fn set_vertical_impl(this: &Rc<dyn IsStackPanel>, value: bool) {
        this.stack_panel().vertical.set(value);
        this.invalidate_measure();
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsStackPanel> = dyn_cast_rc(this.clone()).unwrap();
        if this.stack_panel().vertical.get() {
            let mut size = Vector::null();
            for child in this.children().iter() {
                child.measure(w, None);
                let desired_size = child.desired_size();
                size += Vector { x: 0, y: desired_size.y };
                size = size.max(Vector { x: desired_size.x, y: 0 });
            }
            size
        } else {
            let mut size = Vector::null();
            for child in this.children().iter() {
                child.measure(None, h);
                let desired_size = child.desired_size();
                size += Vector { x: desired_size.x, y: 0 };
                size = size.max(Vector { x: 0, y: desired_size.y });
            }
            size
        }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsStackPanel> = dyn_cast_rc(this.clone()).unwrap();
        if this.stack_panel().vertical.get() {
            let mut pos = bounds.tl;
            let mut size = Vector::null();
            for child in this.children().iter() {
                let child_size = Vector { x: bounds.w(), y: child.desired_size().y };
                child.arrange(Rect { tl: pos, size: child_size });
                pos = pos.offset(Vector { x: 0, y: child_size.y });
                size += Vector { x: 0, y: child_size.y };
                size = size.max(Vector { x: child_size.x, y: 0 });
            }
            size
        } else {
            let mut pos = bounds.tl;
            let mut size = Vector::null();
            for child in this.children().iter() {
                let child_size = Vector { x: child.desired_size().x, y: bounds.h() };
                child.arrange(Rect { tl: pos, size: child_size });
                pos = pos.offset(Vector { x: child_size.x, y: 0 });
                size += Vector { x: child_size.x, y: 0 };
                size = size.max(Vector { x: 0, y: child_size.y });
            }
            size
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename="StackPanel")]
pub struct StackPanelTemplate {
    #[serde(flatten)]
    pub panel: PanelTemplate,
    pub vertical: bool,
}

#[typetag::serde]
impl Template for StackPanelTemplate {
    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = StackPanel::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        self.panel.apply(instance, names);
        let obj: Rc<dyn IsStackPanel> = dyn_cast_rc(instance.clone()).unwrap();
        obj.set_vertical(self.vertical);
    }
}
