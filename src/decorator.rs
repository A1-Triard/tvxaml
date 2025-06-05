use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use crate::template::Template;

import! { pub decorator:
    use [view crate::view];
}

#[class_unsafe(inherits_View)]
pub struct Decorator {
    child: RefCell<Rc<dyn IsView>>,
    #[non_virt]
    child: fn() -> Rc<dyn IsView>,
    #[non_virt]
    set_child: fn(value: Rc<dyn IsView>),
    #[over]
    visual_children_count: (),
    #[over]
    visual_child: (),
}

impl Decorator {
    pub fn new() -> Rc<dyn IsDecorator> {
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
        let panel: Rc<dyn TPanel> = dyn_cast_rc(this.clone()).unwrap();
        panel.panel().children.init(this);
    }

    pub fn children_impl(this: &Rc<dyn TPanel>) -> Rc<dyn IsViewVec> {
        this.panel().children.clone()
    }

    pub fn visual_children_count_impl(this: &Rc<dyn IsView>) -> usize {
        let this: Rc<dyn TPanel> = dyn_cast_rc(this.clone()).unwrap();
        this.panel().children.len()
    }

    pub fn visual_child_impl(this: &Rc<dyn IsView>, index: usize) -> Rc<dyn IsView> {
        let this: Rc<dyn TPanel> = dyn_cast_rc(this.clone()).unwrap();
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
    fn create_instance(&self) -> Rc<dyn TObj> {
        let obj = Panel::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn TObj>) {
        self.view.apply(instance);
        let obj: Rc<dyn TPanel> = dyn_cast_rc(instance.clone()).unwrap();
        for child in &self.children {
            obj.children().push(dyn_cast_rc(child.load_content()).unwrap());
        }
    }
}
