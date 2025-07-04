use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use crate::template::{Template, NameResolver};

import! { panel_children_vec:
    use [view_vec crate::view_vec];
}

#[class_unsafe(inherits_ViewVec)]
struct PanelChildrenVec {
    #[over]
    changed: (),
}

impl PanelChildrenVec {
    fn new() -> Rc<dyn IsPanelChildrenVec> {
        Rc::new(unsafe { Self::new_raw(PANEL_CHILDREN_VEC_VTABLE.as_ptr()) })
    }

    unsafe fn new_raw(vtable: Vtable) -> Self {
        PanelChildrenVec {
            view_vec: unsafe { ViewVec::new_raw(true, true, vtable) },
        }
    }

    fn changed_impl(this: &Rc<dyn IsViewVec>) {
        ViewVec::changed_impl(this);
        this.owner().map(|x| x.invalidate_measure());
    }
}

import! { pub panel:
    use [view crate::view];
    use crate::view_vec::IsViewVec;
}

#[class_unsafe(inherits_View)]
pub struct Panel {
    children: Rc<dyn IsViewVec>,
    #[over]
    _init: (),
    #[non_virt]
    children: fn() -> Rc<dyn IsViewVec>,
    #[over]
    visual_children_count: (),
    #[over]
    visual_child: (),
}

impl Panel {
    pub fn new() -> Rc<dyn IsPanel> {
        let res: Rc<dyn IsPanel> = Rc::new(unsafe { Self::new_raw(PANEL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Panel {
            view: unsafe { View::new_raw(vtable) },
            children: PanelChildrenVec::new()
        }
    }

    pub fn _init_impl(this: &Rc<dyn IsView>) {
        View::_init_impl(this);
        let panel: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        panel.panel().children.init(this);
    }

    pub fn children_impl(this: &Rc<dyn IsPanel>) -> Rc<dyn IsViewVec> {
        this.panel().children.clone()
    }

    pub fn visual_children_count_impl(this: &Rc<dyn IsView>) -> usize {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        this.panel().children.len()
    }

    pub fn visual_child_impl(this: &Rc<dyn IsView>, index: usize) -> Rc<dyn IsView> {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        this.panel().children.at(index)
    }
}

#[macro_export]
macro_rules! panel_template {
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
                #[serde(skip_serializing_if="Vec::is_empty")]
                pub children: Vec<Box<dyn $crate::template::Template>>,

                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! panel_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
        {
            use $crate::panel::PanelExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::panel::IsPanel>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            for child in &$this.children {
                obj.children().push($crate::dynamic_cast_dyn_cast_rc(child.load_content($names)).unwrap());
            }
        }
    };
}

panel_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Panel@Children")]
    pub struct PanelTemplate in template { }
}

#[typetag::serde(name="Panel")]
impl Template for PanelTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Panel::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        panel_apply_template!(this, instance, names);
    }
}
