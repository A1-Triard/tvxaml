use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use std::cell::RefCell;
use crate::base::option_addr_eq;
use crate::template::{Template, NameResolver};

import! { pub decorator:
    use [view crate::view];
}

#[class_unsafe(inherits_View)]
pub struct Decorator {
    child: RefCell<Option<Rc<dyn IsView>>>,
    #[non_virt]
    child: fn() -> Option<Rc<dyn IsView>>,
    #[non_virt]
    set_child: fn(value: Option<Rc<dyn IsView>>),
    #[over]
    visual_children_count: (),
    #[over]
    visual_child: (),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl Decorator {
    pub fn new() -> Rc<dyn IsDecorator> {
        let res: Rc<dyn IsDecorator> = Rc::new(unsafe { Self::new_raw(DECORATOR_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Decorator {
            view: unsafe { View::new_raw(vtable) },
            child: RefCell::new(None),
        }
    }

    pub fn child_impl(this: &Rc<dyn IsDecorator>) -> Option<Rc<dyn IsView>> {
        this.decorator().child.borrow().clone()
    }

    pub fn set_child_impl(this: &Rc<dyn IsDecorator>, value: Option<Rc<dyn IsView>>) {
        if option_addr_eq(
            this.decorator().child.borrow().as_ref().map(Rc::as_ptr),
            value.as_ref().map(Rc::as_ptr)
        ) {
            return;
        }
        let child = this.decorator().child.borrow().clone();
        if let Some(child) = child {
            this.remove_visual_child(&child);
            child._set_visual_parent(None);
            child._set_layout_parent(None);
        }
        *this.decorator().child.borrow_mut() = value.clone();
        let this: Rc<dyn IsView> = this.clone();
        if let Some(child) = value {
            child._set_layout_parent(Some(&this));
            child._set_visual_parent(Some(&this));
            this.add_visual_child(&child);
        }
        this.invalidate_measure();
    }

    pub fn visual_children_count_impl(this: &Rc<dyn IsView>) -> usize {
        let this: Rc<dyn IsDecorator> = dyn_cast_rc(this.clone()).unwrap();
        if this.decorator().child.borrow().is_some() { 1 } else { 0 }
    }

    pub fn visual_child_impl(this: &Rc<dyn IsView>, index: usize) -> Rc<dyn IsView> {
        let this: Rc<dyn IsDecorator> = dyn_cast_rc(this.clone()).unwrap();
        assert_eq!(index, 0);
        this.decorator().child.borrow().clone().unwrap()
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, w: Option<i16>, h: Option<i16>) -> Vector {
        let this: Rc<dyn IsDecorator> = dyn_cast_rc(this.clone()).unwrap();
        if let Some(child) = this.decorator().child.borrow().clone() {
            child.measure(w, h);
            child.desired_size()
        } else {
            Vector::null()
        }
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsDecorator> = dyn_cast_rc(this.clone()).unwrap();
        if let Some(child) = this.decorator().child.borrow().clone() {
            child.arrange(bounds);
            child.render_bounds().size
        } else {
            Vector::null()
        }
    }
}

#[macro_export]
macro_rules! decorator_template {
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
                pub child: Option<Box<dyn $crate::template::Template>>,
                $($(
                    $(#[$field_attr])*
                    pub $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! decorator_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::view_apply_template!($this, $instance, $names);
        {
            use $crate::decorator::DecoratorExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::decorator::IsDecorator>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.child.as_ref().map(|x|
                obj.set_child(Some($crate::dynamic_cast_dyn_cast_rc(x.load_content($names)).unwrap()))
            );
        }
    };
}

decorator_template! {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    #[serde(rename="Decorator@Child")]
    pub struct DecoratorTemplate in template { }
}

#[typetag::serde(name="Decorator")]
impl Template for DecoratorTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        Decorator::new()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        decorator_apply_template!(this, instance, names);
    }
}
