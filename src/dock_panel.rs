use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use serde::{Serialize, Deserialize};
use std::cell::Cell;
use crate::template::{Template, Names};
use crate::view_vec::ViewVecExt;

import! { pub dock_layout:
    use [layout crate::view];
    use int_vec_2d::Point;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Dock { Left, Top, Right, Bottom }

#[class_unsafe(inherits_Layout)]
pub struct DockLayout {
    dock: Cell<Option<Dock>>,
    #[non_virt]
    dock: fn() -> Option<Dock>,
    #[non_virt]
    set_dock: fn(value: Option<Dock>),
}

impl DockLayout {
    pub fn new() -> Rc<dyn IsDockLayout> {
        Rc::new(unsafe { Self::new_raw(DOCK_LAYOUT_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        DockLayout {
            layout: unsafe { Layout::new_raw(vtable) },
            dock: Cell::new(None),
        }
    }

    pub fn dock_impl(this: &Rc<dyn IsDockLayout>) -> Option<Dock> {
        this.dock_layout().dock.get()
    }

    pub fn set_dock_impl(this: &Rc<dyn IsDockLayout>, value: Option<Dock>) {
        this.dock_layout().dock.set(value);
        this.owner().and_then(|x| x.layout_parent()).map(|x| x.invalidate_measure());
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename="DockLayout")]
pub struct DockLayoutTemplate {
    #[serde(flatten)]
    pub layout: LayoutTemplate,
    pub dock: Option<Dock>,
}

#[typetag::serde]
impl Template for DockLayoutTemplate {
    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = DockLayout::new();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        self.layout.apply(instance, names);
        let obj: Rc<dyn IsDockLayout> = dyn_cast_rc(instance.clone()).unwrap();
        obj.set_dock(self.dock);
    }
}

import! { pub dock_panel:
    use [panel crate::panel];
}

#[class_unsafe(inherits_Panel)]
pub struct DockPanel {
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl DockPanel {
    pub fn new() -> Rc<dyn IsDockPanel> {
        Rc::new(unsafe { Self::new_raw(DOCK_PANEL_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        DockPanel {
            panel: unsafe { Panel::new_raw(vtable) },
        }
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, mut w: Option<i16>, mut h: Option<i16>) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        let mut size = Vector::null();
        let mut docked = Thickness::all(0);
        for child in this.children().iter() {
            let dock_layout: Option<Rc<dyn IsDockLayout>> = dyn_cast_rc(child.layout());
            let dock = dock_layout.and_then(|x| x.dock());
            match dock {
                None => { },
                Some(Dock::Left) => {
                    child.measure(None, h);
                    if let Some(w) = w.as_mut() {
                        *w = (*w as u16).saturating_sub(child.desired_size().x as u16) as i16;
                    }
                    size = size.max(Vector { x: 0, y: child.desired_size().y });
                    let docked_child = Thickness::new(i32::from(child.desired_size().x), 0, 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Some(Dock::Top) => {
                    child.measure(w, None);
                    if let Some(h) = h.as_mut() {
                        *h = (*h as u16).saturating_sub(child.desired_size().y as u16) as i16;
                    }
                    size = size.max(Vector { x: child.desired_size().x, y: 0 });
                    let docked_child = Thickness::new(0, i32::from(child.desired_size().y), 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Some(Dock::Right) => {
                    child.measure(None, h);
                    if let Some(w) = w.as_mut() {
                        *w = (*w as u16).saturating_sub(child.desired_size().x as u16) as i16;
                    }
                    size = size.max(Vector { x: 0, y: child.desired_size().y });
                    let docked_child = Thickness::new(0, 0, i32::from(child.desired_size().x), 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Some(Dock::Bottom) => {
                    child.measure(w, None);
                    if let Some(h) = h.as_mut() {
                        *h = (*h as u16).saturating_sub(child.desired_size().y as u16) as i16;
                    }
                    size = size.max(Vector { x: child.desired_size().x, y: 0 });
                    let docked_child = Thickness::new(0, 0, 0, i32::from(child.desired_size().y));
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
            }
        }
        for child in this.children().iter() {
            let dock_layout: Option<Rc<dyn IsDockLayout>> = dyn_cast_rc(child.layout());
            let dock = dock_layout.and_then(|x| x.dock());
            if dock.is_none() {
                child.measure(w, h);
                size = size.max(child.desired_size());
            }
        }
        docked.expand_rect_size(size)
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsPanel> = dyn_cast_rc(this.clone()).unwrap();
        let mut size = Vector::null();
        let mut docked = Thickness::all(0);
        for child in this.children().iter() {
            let bounds = docked.shrink_rect(bounds);
            let dock_layout: Option<Rc<dyn IsDockLayout>> = dyn_cast_rc(child.layout());
            let dock = dock_layout.and_then(|x| x.dock());
            match dock {
                None => { },
                Some(Dock::Left) => {
                    child.arrange(
                        Rect { tl: bounds.tl, size: Vector { x: child.desired_size().x, y: bounds.h() } },
                    );
                    size = size.max(Vector { x: 0, y: child.desired_size().y });
                    let docked_child = Thickness::new(i32::from(child.desired_size().x), 0, 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Some(Dock::Top) => {
                    child.arrange(
                        Rect { tl: bounds.tl, size: Vector { x: bounds.w(), y: child.desired_size().y } },
                    );
                    size = size.max(Vector { x: child.desired_size().x, y: 0 });
                    let docked_child = Thickness::new(0, i32::from(child.desired_size().y), 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Some(Dock::Right) => {
                    child.arrange(
                        Rect::from_tl_br(
                            Point {
                                x: bounds.r().wrapping_sub(child.desired_size().x),
                                y: bounds.t()
                            },
                            bounds.br()
                        ),
                    );
                    size = size.max(Vector { x: 0, y: child.desired_size().y });
                    let docked_child = Thickness::new(0, 0, i32::from(child.desired_size().x), 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Some(Dock::Bottom) => {
                    child.arrange(
                        Rect::from_tl_br(
                            Point {
                                x: bounds.l(),
                                y: bounds.b().wrapping_sub(child.desired_size().y)
                            },
                            bounds.br()
                        ),
                    );
                    size = size.max(Vector { x: child.desired_size().x, y: 0 });
                    let docked_child = Thickness::new(0, 0, 0, i32::from(child.desired_size().y));
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
            }
        }
        let bounds = docked.shrink_rect(bounds);
        for child in this.children().iter() {
            let dock_layout: Option<Rc<dyn IsDockLayout>> = dyn_cast_rc(child.layout());
            let dock = dock_layout.and_then(|x| x.dock());
            if dock.is_none() {
                child.arrange(bounds);
                size = size.max(child.render_bounds().size);
            }
        }
        docked.expand_rect_size(size)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename="DockPanel")]
pub struct DockPanelTemplate {
    #[serde(flatten)]
    pub panel: PanelTemplate,
}

#[typetag::serde]
impl Template for DockPanelTemplate {
    fn is_name_scope(&self) -> bool {
        self.panel.view.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.panel.view.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = DockPanel::new();
        obj.init();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut Names) {
        self.panel.apply(instance, names);
    }
}
