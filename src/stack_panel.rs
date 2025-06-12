use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::Cell;
use crate::template::{Template, NameResolver};
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
        let res: Rc<dyn IsStackPanel> = Rc::new(unsafe { Self::new_raw(STACK_PANEL_VTABLE.as_ptr()) });
        res._init();
        res
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
        let old = this.stack_panel().vertical.replace(value);
        if old == value { return; }
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

#[macro_export]
macro_rules! stack_panel_template {
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
        $crate::panel_template! {
            $(#[$attr])*
            $vis struct $name in $mod {
                $(use $path as $import;)*

                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub vertical: Option<bool>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! stack_panel_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::panel_apply_template!($this, $instance, $names);
        {
            use $crate::stack_panel::StackPanelExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::stack_panel::IsStackPanel>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.vertical.map(|x| obj.set_vertical(x));
        }
    };
}

stack_panel_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="StackPanel@Children")]
    pub struct StackPanelTemplate in template { }
}

#[typetag::serde(name="StackPanel")]
impl Template for StackPanelTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        StackPanel::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        stack_panel_apply_template!(this, instance, names);
    }
}
