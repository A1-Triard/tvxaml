use basic_oop::{class_unsafe, import, Vtable};

import! { pub app:
    use [obj basic_oop::obj];
    use std::rc::Rc;
    use crate::view::TView;
}

#[class_unsafe(inherits_Obj)]
pub struct App {
    root: Rc<dyn TView>,
    #[non_virt]
    root: fn() -> Rc<dyn TView>,
}

impl App {
    pub fn new(root: Rc<dyn TView>) -> Rc<dyn TApp> {
        Rc::new(unsafe { Self::new_raw(root, APP_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(root: Rc<dyn TView>, vtable: Vtable) -> Self {
        App {
            obj: unsafe { Obj::new_raw(vtable) },
            root,
        }
    }

    pub fn root_impl(this: &Rc<dyn TApp>) -> Rc<dyn TView> {
        this.app().root.clone()
    }
}
