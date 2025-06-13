use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use iter_identify_first_last::IteratorIdentifyFirstLastExt;
use crate::template::{Template, NameResolver};
use crate::view_vec::ViewVecExt;

import! { pub adorners_panel:
    use [panel crate::panel];
}

#[class_unsafe(inherits_Panel)]
pub struct AdornersPanel {
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl AdornersPanel {
    pub fn new() -> Rc<dyn IsAdornersPanel> {
        let res: Rc<dyn IsAdornersPanel> = Rc::new(unsafe { Self::new_raw(ADORNERS_PANEL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        AdornersPanel {
            panel: unsafe { Panel::new_raw(vtable) },
        }
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        let mut size = Vector::null();
        for (is_first, child) in this.children().iter().identify_first() {
            if is_first {
                child.measure(w, h);
                size = child.desired_size();
            } else {
                child.measure(Some(size.x), Some(size.y));
            }
        }
        size
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        let mut child_bounds = bounds;
        for (is_first, child) in this.children().iter().identify_first() {
            if is_first {
                child.arrange(bounds);
                child_bounds = child.render_bounds();
            } else {
                child.arrange(child_bounds);
            }
        }
        child_bounds.size
    }
}

#[macro_export]
macro_rules! adorners_panel_template {
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
macro_rules! adorners_panel_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::panel_apply_template!($this, $instance, $names);
    };
}

adorners_panel_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="AdornersPanel@Children")]
    pub struct AdornersPanelTemplate in adorners_panel_template { }
}

#[typetag::serde(name="AdornersPanel")]
impl Template for AdornersPanelTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        AdornersPanel::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        adorners_panel_apply_template!(this, instance, names);
    }
}
