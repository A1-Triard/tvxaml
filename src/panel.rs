use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;

import! { pub panel:
    use [view crate::view];
}

#[class_unsafe(inherits_ViewVec)]
struct PanelChildrenVec {
    #[over]
    changed: (),
}

impl PanelChildrenVec {
    fn new() -> Rc<dyn TPanelChildrenVec> {
        Rc::new(unsafe { Self::new_raw(PANEL_CHILDREN_VEC_VTABLE.as_ptr()) })
    }

    unsafe fn new_raw(vtable: Vtable) -> Self {
        PanelChildrenVec {
            view_vec: unsafe { ViewVec::new_raw(vtable) },
        }
    }

    fn changed_impl(this: &Rc<dyn TViewVec>) {
        ViewVec::changed_impl(this);
        this.owner().map(|x| x.invalidate_measure());
    }
}

#[class_unsafe(inherits_View)]
pub struct Panel {
    children: Rc<dyn TViewVec>,
    #[over]
    init: (),
    #[non_virt]
    children: fn() -> Rc<dyn TViewVec>,
}

impl Panel {
    pub fn new() -> Rc<dyn TPanel> {
        Rc::new(unsafe { Self::new_raw(PANEL_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Panel {
            view: unsafe { View::new_raw(vtable) },
            children: dyn_cast_rc(PanelChildrenVec::new()).unwrap()
        }
    }

    pub fn init_impl(this: &Rc<dyn TView>) {
        View::init_impl(this);
        let panel: Rc<dyn TPanel> = dyn_cast_rc(this.clone()).unwrap();
        panel.panel().children.init(this);
    }

    pub fn children_impl(this: &Rc<dyn TPanel>) -> Rc<dyn TViewVec> {
        this.panel().children.clone()
    }
}
