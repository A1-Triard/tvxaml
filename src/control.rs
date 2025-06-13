use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::base::{Fg, Bg};
use crate::static_text::StaticTextTemplate;
use crate::template::NameResolver;

import! { pub control:
    use [view crate::view];
    use crate::template::{Template, Names};
}

struct ControlData {
    child: Option<(Rc<dyn IsView>, Names)>,
    template: Option<Box<dyn Template>>,
}

#[class_unsafe(inherits_View)]
pub struct Control {
    data: RefCell<ControlData>,
    #[over]
    visual_children_count: (),
    #[over]
    visual_child: (),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
    #[virt]
    template: fn() -> Box<dyn Template>,
    #[over]
    _init: (),
    #[non_virt]
    update: fn(),
    #[virt]
    update_override: fn(template: &Names),
    #[over]
    _attach_to_app: (),
    #[over]
    _detach_from_app: (),
}

impl Control {
    pub fn new() -> Rc<dyn IsControl> {
        let res: Rc<dyn IsControl> = Rc::new(unsafe { Self::new_raw(CONTROL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Control {
            view: unsafe { View::new_raw(vtable) },
            data: RefCell::new(ControlData {
                child: None,
                template: None,
            }),
        }
    }

    pub fn _init_impl(this: &Rc<dyn IsView>) {
        View::_init_impl(this);
        let this: Rc<dyn IsControl> = dyn_cast_rc(this.clone()).unwrap();
        let template = this.template();
        this.control().data.borrow_mut().template = Some(template);
    }

    pub fn template_impl(_this: &Rc<dyn IsControl>) -> Box<dyn Template> {
        Box::new(StaticTextTemplate {
            text: Some("Control".to_string()),
            color: Some((Fg::Red, Bg::Green)),
            .. Default::default()
        })
    }

    pub fn update_impl(this: &Rc<dyn IsControl>) {
        if this.app().is_some() {
            let child = this.control().data.borrow().child.as_ref().unwrap().1.clone();
            this.update_override(&child);
        }
    }

    pub fn update_override_impl(_this: &Rc<dyn IsControl>, _template: &Names) { }

    pub fn _attach_to_app_impl(this: &Rc<dyn IsView>, value: &Rc<dyn IsApp>) {
        let child = {
            let this: Rc<dyn IsControl> = dyn_cast_rc(this.clone()).unwrap();
            let (child, names) = {
                let mut data = this.control().data.borrow_mut();
                let (child, names) = data.template.as_ref().unwrap().load_root();
                let child: Rc<dyn IsView> = dyn_cast_rc(child).expect("View");
                data.child = Some((child.clone(), names.clone()));
                (child, names)
            };
            this.update_override(&names);
            child
        };
        View::_attach_to_app_impl(this, value);
        child._set_layout_parent(Some(&this));
        child._set_visual_parent(Some(&this));
        this.add_visual_child(&child);
        this.invalidate_measure();
    }

    pub fn _detach_from_app_impl(this: &Rc<dyn IsView>) {
        let child = {
            let this: Rc<dyn IsControl> = dyn_cast_rc(this.clone()).unwrap();
            this.control().data.borrow_mut().child.take().unwrap().0
        };
        this.remove_visual_child(&child);
        child._set_visual_parent(None);
        child._set_layout_parent(None);
        View::_detach_from_app_impl(this);
    }

    pub fn visual_children_count_impl(this: &Rc<dyn IsView>) -> usize {
        if this.app().is_some() { 1 } else { 0 }
    }

    pub fn visual_child_impl(this: &Rc<dyn IsView>, index: usize) -> Rc<dyn IsView> {
        let this: Rc<dyn IsControl> = dyn_cast_rc(this.clone()).unwrap();
        assert_eq!(index, 0);
        debug_assert!(this.app().is_some());
        this.control().data.borrow().child.as_ref().unwrap().0.clone()
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsControl> = dyn_cast_rc(this.clone()).unwrap();
        let child = this.control().data.borrow().child.as_ref().map(|x| x.0.clone());
        if let Some(child) = child {
            child.measure(w, h);
            child.desired_size()
        } else {
            Vector::null()
        }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsControl> = dyn_cast_rc(this.clone()).unwrap();
        let child = this.control().data.borrow().child.as_ref().map(|x| x.0.clone());
        if let Some(child) = child {
            child.arrange(bounds);
            child.render_bounds().size
        } else {
            Vector::null()
        }
    }
}

#[macro_export]
macro_rules! control_template {
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

                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! control_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
    };
}

control_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Control")]
    pub struct ControlTemplate in template { }
}

#[typetag::serde(name="Control")]
impl Template for ControlTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Control::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        control_apply_template!(this, instance, names);
    }
}
