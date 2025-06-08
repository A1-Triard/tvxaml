use basic_oop::{class_unsafe, import, Vtable};
use dynamic_cast::dyn_cast_rc;
use iter_identify_first_last::IteratorIdentifyFirstLastExt;
use serde::{Serialize, Deserialize};
use std::cell::Cell;
use crate::template::{Template, NameResolver};
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
    dock: Cell<Dock>,
    #[non_virt]
    dock: fn() -> Dock,
    #[non_virt]
    set_dock: fn(value: Dock),
}

impl DockLayout {
    pub fn new() -> Rc<dyn IsDockLayout> {
        Rc::new(unsafe { Self::new_raw(DOCK_LAYOUT_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        DockLayout {
            layout: unsafe { Layout::new_raw(vtable) },
            dock: Cell::new(Dock::Left),
        }
    }

    pub fn dock_impl(this: &Rc<dyn IsDockLayout>) -> Dock {
        this.dock_layout().dock.get()
    }

    pub fn set_dock_impl(this: &Rc<dyn IsDockLayout>, value: Dock) {
        this.dock_layout().dock.set(value);
        this.owner().and_then(|x| x.layout_parent()).map(|x| x.invalidate_measure());
    }
}

#[macro_export]
macro_rules! dock_layout_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::layout_template! {
            $(#[$attr])*
            $vis struct $name {
                #[serde(default)]
                #[serde(skip_serializing_if="Option::is_none")]
                pub dock: Option<Dock>,
                $($(
                    $(#[$field_attr])*
                    $field_vis $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! dock_layout_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::layout_apply_template!($this, $instance, $names);
        {
            use $crate::dock_panel::DockLayoutExt;

            let obj: $crate::alloc_rc_Rc<dyn $crate::dock_panel::IsDockLayout>
                = $crate::dynamic_cast_dyn_cast_rc($instance.clone()).unwrap();
            $this.dock.map(|x| obj.set_dock(x));
        }
    };
}

dock_layout_template! {
    #[derive(Serialize, Deserialize, Default)]
    #[serde(rename="DockLayout@Dock")]
    pub struct DockLayoutTemplate { }
}

#[typetag::serde(name="DockLayout")]
impl Template for DockLayoutTemplate {
    fn create_instance(&self) -> Rc<dyn IsObj> {
        let obj = DockLayout::new();
        dyn_cast_rc(obj).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        dock_layout_apply_template!(this, instance, names);
    }
}

import! { pub dock_panel:
    use [panel crate::panel];
}

#[class_unsafe(inherits_Panel)]
pub struct DockPanel {
    last_child_fill: Cell<bool>,
    #[non_virt]
    last_child_fill: fn() -> bool,
    #[non_virt]
    set_last_child_fill: fn(value: bool),
    #[over]
    measure_override: (),
    #[over]
    arrange_override: (),
}

impl DockPanel {
    pub fn new() -> Rc<dyn IsDockPanel> {
        let res: Rc<dyn IsDockPanel> = Rc::new(unsafe { Self::new_raw(DOCK_PANEL_VTABLE.as_ptr()) });
        res._init();
        res
    }

    pub unsafe fn new_raw(vtable: Vtable) -> Self {
        DockPanel {
            panel: unsafe { Panel::new_raw(vtable) },
            last_child_fill: Cell::new(true),
        }
    }

    pub fn last_child_fill_impl(this: &Rc<dyn IsDockPanel>) -> bool {
        this.dock_panel().last_child_fill.get()
    }

    pub fn set_last_child_fill_impl(this: &Rc<dyn IsDockPanel>, value: bool) {
        this.dock_panel().last_child_fill.set(value);
    }

    pub fn measure_override_impl(this: &Rc<dyn IsView>, mut w: Option<i16>, mut h: Option<i16>) -> Vector {
        let this: Rc<dyn IsDockPanel> = dyn_cast_rc(this.clone()).unwrap();
        let last_child_fill = this.last_child_fill();
        let mut size = Vector::null();
        let mut docked = Thickness::all(0);
        for (is_last, child) in this.children().iter().identify_last() {
            if is_last && last_child_fill {
                child.measure(w, h);
                size = size.max(child.desired_size());
                continue;
            }
            let dock_layout: Option<Rc<dyn IsDockLayout>> = dyn_cast_rc(child.layout());
            let dock = dock_layout.map_or(Dock::Left, |x| x.dock());
            match dock {
                Dock::Left => {
                    child.measure(None, h);
                    if let Some(w) = w.as_mut() {
                        *w = (*w as u16).saturating_sub(child.desired_size().x as u16) as i16;
                    }
                    size = size.max(Vector { x: 0, y: child.desired_size().y });
                    let docked_child = Thickness::new(i32::from(child.desired_size().x), 0, 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Dock::Top => {
                    child.measure(w, None);
                    if let Some(h) = h.as_mut() {
                        *h = (*h as u16).saturating_sub(child.desired_size().y as u16) as i16;
                    }
                    size = size.max(Vector { x: child.desired_size().x, y: 0 });
                    let docked_child = Thickness::new(0, i32::from(child.desired_size().y), 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Dock::Right => {
                    child.measure(None, h);
                    if let Some(w) = w.as_mut() {
                        *w = (*w as u16).saturating_sub(child.desired_size().x as u16) as i16;
                    }
                    size = size.max(Vector { x: 0, y: child.desired_size().y });
                    let docked_child = Thickness::new(0, 0, i32::from(child.desired_size().x), 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Dock::Bottom => {
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
        docked.expand_rect_size(size)
    }

    pub fn arrange_override_impl(this: &Rc<dyn IsView>, bounds: Rect) -> Vector {
        let this: Rc<dyn IsDockPanel> = dyn_cast_rc(this.clone()).unwrap();
        let last_child_fill = this.last_child_fill();
        let mut size = Vector::null();
        let mut docked = Thickness::all(0);
        for (is_last, child) in this.children().iter().identify_last() {
            let bounds = docked.shrink_rect(bounds);
            if is_last && last_child_fill {
                child.arrange(bounds);
                size = size.max(child.render_bounds().size);
                continue;
            }
            let dock_layout: Option<Rc<dyn IsDockLayout>> = dyn_cast_rc(child.layout());
            let dock = dock_layout.map_or(Dock::Left, |x| x.dock());
            match dock {
                Dock::Left => {
                    child.arrange(
                        Rect { tl: bounds.tl, size: Vector { x: child.desired_size().x, y: bounds.h() } },
                    );
                    size = size.max(Vector { x: 0, y: child.desired_size().y });
                    let docked_child = Thickness::new(i32::from(child.desired_size().x), 0, 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Dock::Top => {
                    child.arrange(
                        Rect { tl: bounds.tl, size: Vector { x: bounds.w(), y: child.desired_size().y } },
                    );
                    size = size.max(Vector { x: child.desired_size().x, y: 0 });
                    let docked_child = Thickness::new(0, i32::from(child.desired_size().y), 0, 0);
                    docked += docked_child;
                    size = docked_child.shrink_rect_size(size);
                },
                Dock::Right => {
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
                Dock::Bottom => {
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
        docked.expand_rect_size(size)
    }
}

#[macro_export]
macro_rules! dock_panel_template {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident {
            $($(
                $(#[$field_attr:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $crate::panel_template! {
            $(#[$attr])*
            $vis struct $name {
                $($(
                    $(#[$field_attr])*
                    $field_vis $field_name : $field_ty
                ),+)?
            }
        }
    };
}

#[macro_export]
macro_rules! dock_panel_apply_template {
    ($this:ident, $instance:ident, $names:ident) => {
        $crate::panel_apply_template!($this, $instance, $names);
    };
}

dock_panel_template! {
    #[derive(Serialize, Deserialize, Default)]
    #[serde(rename="DockPanel@Children")]
    pub struct DockPanelTemplate { }
}

#[typetag::serde(name="DockPanel")]
impl Template for DockPanelTemplate {
    fn is_name_scope(&self) -> bool {
        self.is_name_scope
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn create_instance(&self) -> Rc<dyn IsObj> {
        dyn_cast_rc(DockPanel::new()).unwrap()
    }

    fn apply(&self, instance: &Rc<dyn IsObj>, names: &mut NameResolver) {
        let this = self;
        dock_panel_apply_template!(this, instance, names);
    }
}
