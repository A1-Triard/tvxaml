use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use crate::template::{Template, Names};

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
    init: (),
    #[non_virt]
    children: fn() -> Rc<dyn IsViewVec>,
    #[over]
    visual_children_count: (),
    #[over]
    visual_child: (),
}

impl Panel {
    pub fn new() -> Rc<dyn IsPanel> {
        Rc::new(unsafe { Self::new_raw(PANEL_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        Panel {
            view: unsafe { View::new_raw(vtable) },
            children: dyn_cast_rc(PanelChildrenVec::new()).unwrap()
        }
    }

    pub fn init_impl(this: &Rc<dyn IsView>) {
        View::init_impl(this);
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

#[derive(Serialize, Deserialize)]
#[serde(rename="Panel")]
pub struct PanelTemplate {
    #[serde(flatten)]
    pub view: ViewTemplate,
    pub children: Vec<Box<dyn Template>>,
}

#[typetag::serde]
impl Template for PanelTemplate {
    fn is_name_scope(&self) -> bool {
        self.view.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.view.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = Panel::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        self.view.apply(instance, names);
        let obj: Rc<dyn IsPanel> = dyn_cast_rc(instance.clone()).unwrap();
        for child in &self.children {
            obj.children().push(dyn_cast_rc(child.load_content(names)).unwrap());
        }
    }
}
