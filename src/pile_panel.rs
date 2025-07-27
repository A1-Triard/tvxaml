use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use crate::template::{Template, NameResolver};
use crate::view_vec::ViewVecExt;

import! { pub pile_panel:
    use [panel crate::panel];
}

#[class_unsafe(inherits_Panel)]
pub struct PilePanel {
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl PilePanel {
    pub fn new() -> Rc<dyn IsPilePanel> {
        let res: Rc<dyn IsPilePanel> = Rc::new(unsafe { Self::new_raw(PILE_PANEL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        PilePanel {
            panel: unsafe { Panel::new_raw(vtable) },
        }
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        let mut size = Vector::null();
        for child in this.children().iter() {
            child.measure(w, h);
            size = size.max(child.desired_size());
        }
        size
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        let mut size = Vector::null();
        for child in this.children().iter() {
            child.arrange(bounds);
            size = size.max(child.render_bounds().size);
        }
        size
    }
}

#[macro_export]
macro_rules! pile_panel_template {
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

                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! pile_panel_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::panel_apply_template!($this, $instance, $names);
    };
}

pile_panel_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
    #[serde(rename="PilePanel@Children")]
    pub struct PilePanelTemplate in pile_panel_template { }
}

#[typetag::serde(name="PilePanel")]
impl Template for PilePanelTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        PilePanel::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        pile_panel_apply_template!(this, instance, names);
    }
}
