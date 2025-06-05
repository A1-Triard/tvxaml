use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::cell::RefCell;
use crate::template::{Template, Names};

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
        Rc::new(unsafe { Self::new_raw(DECORATOR_VTABLE.as_ptr()) })
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
        let child = this.decorator().child.borrow().clone();
        if let Some(child) = child {
            this.remove_visual_child(&child);
            child.set_visual_parent(None);
            child.set_layout_parent(None);
        }
        *this.decorator().child.borrow_mut() = value.clone();
        let this: Rc<dyn IsView> = dyn_cast_rc(this.clone()).unwrap();
        if let Some(child) = value {
            child.set_layout_parent(Some(&this));
            child.set_visual_parent(Some(&this));
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

#[derive(Serialize, Deserialize)]
#[serde(rename="Decorator")]
pub struct DecoratorTemplate {
    #[serde(flatten)]
    pub view: ViewTemplate,
    pub child: Option<Box<dyn Template>>,
}

#[typetag::serde]
impl Template for DecoratorTemplate {
    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = Decorator::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        self.view.apply(instance, names);
        let obj: Rc<dyn IsDecorator> = dyn_cast_rc(instance.clone()).unwrap();
        obj.set_child(self.child.as_ref().map(|x| dyn_cast_rc(x.load_content(names)).unwrap()));
    }
}
