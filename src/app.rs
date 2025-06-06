use basic_oop::{class_unsafe, import, Vtable};
use int_vec_2d::{Vector, Point};
use std::cell::{Cell, RefCell};
use std::mem::replace;
use std::ptr::addr_eq;
use std::rc::{self};
use tvxaml_screen_base::Screen;
use crate::render_port::RenderPort;
use crate::view::{View, ViewExt};

fn option_addr_eq<T, U>(p: Option<*const T>, q: Option<*const U>) -> bool where T: ?Sized, U: ?Sized {
    if p.is_none() && q.is_none() { return true; }
    let Some(p) = p else { return false; };
    let Some(q) = q else { return false; };
    addr_eq(p, q)
}

import! { pub app:
    use [obj basic_oop::obj];
    use int_vec_2d::Rect;
    use std::rc::Rc;
    use tvxaml_screen_base::Error as tvxaml_screen_base_Error;
    use crate::view::IsView;
}

struct Focus {
    changing: bool,
    primary: rc::Weak<dyn IsView>,
    secondary: rc::Weak<dyn IsView>,
}

#[class_unsafe(inherits_Obj)]
pub struct App {
    screen: RefCell<Box<dyn Screen>>,
    cursor: Cell<Option<Point>>,
    exit_code: Cell<Option<u8>>,
    root: Rc<dyn IsView>,
    invalidated_rect: Cell<Rect>,
    focus: RefCell<Focus>,
    #[non_virt]
    root: fn() -> Rc<dyn IsView>,
    #[non_virt]
    run: fn() -> Result<u8, tvxaml_screen_base_Error>,
    #[non_virt]
    exit: fn(exit_code: u8),
    #[non_virt]
    quit: fn(),
    #[non_virt]
    invalidate_render: fn(rect: Rect),
    #[non_virt]
    focus: fn(view: Option<&Rc<dyn IsView>>, primary_focus: bool),
    #[non_virt]
    focused: fn(primary_focus: bool) -> Option<Rc<dyn IsView>>,
}

impl App {
    pub fn new(screen: Box<dyn Screen>, root: Rc<dyn IsView>) -> Rc<dyn IsApp> {
        Rc::new(unsafe { Self::new_raw(screen, root, APP_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(screen: Box<dyn Screen>, root: Rc<dyn IsView>, vtable: Vtable) -> Self {
        App {
            obj: unsafe { Obj::new_raw(vtable) },
            screen: RefCell::new(screen),
            cursor: Cell::new(None),
            exit_code: Cell::new(None),
            root,
            invalidated_rect: Cell::new(Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }),
            focus: RefCell::new(Focus {
                changing: false,
                primary: <rc::Weak::<View>>::new(),
                secondary: <rc::Weak::<View>>::new(),
            }),
        }
    }

    pub fn root_impl(this: &Rc<dyn IsApp>) -> Rc<dyn IsView> {
        this.app().root.clone()
    }

    fn render(view: &Rc<dyn IsView>, rp: &mut RenderPort) {
        if rp.invalidated_rect.intersect(rp.bounds).is_empty() {
            return;
        }
        view.render(rp);
        let base_offset = rp.offset;
        let base_bounds = rp.bounds;
        for i in 0 .. view.visual_children_count() {
            let child = view.visual_child(i);
            let bounds = child.margin().shrink_rect(child.render_bounds()).offset(base_offset);
            rp.bounds = bounds.intersect(base_bounds);
            rp.offset = Vector { x: bounds.l(), y: bounds.t() };
            Self::render(&child, rp);
        }
    }

    pub fn run_impl(this: &Rc<dyn IsApp>) -> Result<u8, tvxaml_screen_base_Error> {
        this.root().set_app(Some(this));
        let res = loop {
            if let Some(exit_code) = this.app().exit_code.get() {
                break Ok(exit_code);
            }
            let screen_size = this.app().screen.borrow().size();
            this.app().root.measure(Some(screen_size.x), Some(screen_size.y));
            this.app().root.arrange(Rect { tl: Point { x: 0, y: 0 }, size: screen_size });
            let bounds = this.app().root.margin().shrink_rect(this.app().root.render_bounds());
            let mut screen = this.app().screen.borrow_mut();
            let mut rp = RenderPort {
                screen: screen.as_mut(),
                invalidated_rect: this.app().invalidated_rect.get(),
                bounds: bounds.intersect(Rect { tl: Point { x: 0, y: 0 }, size: screen_size }),
                offset: Vector { x: bounds.l(), y: bounds.t() },
                cursor: this.app().cursor.get(),
            };
            Self::render(&this.app().root, &mut rp);
            let cursor = rp.cursor;
            this.app().cursor.set(cursor);
            if let Err(e) = screen.update(cursor, true) {
                break Err(e);
            }
        };
        this.root().set_app(None);
        res
    }

    pub fn exit_impl(this: &Rc<dyn IsApp>, exit_code: u8) {
        this.app().exit_code.set(Some(exit_code));
    }

    pub fn quit_impl(this: &Rc<dyn IsApp>) {
        this.exit(0);
    }

    pub fn invalidate_render_impl(this: &Rc<dyn IsApp>, rect: Rect) {
        let app_rect = Rect { tl: Point { x: 0, y: 0 }, size: this.app().screen.borrow().size() };
        let union = this.app().invalidated_rect.get().union_intersect(rect, app_rect);
        this.app().invalidated_rect.set(union);
    }

    pub fn focused_impl(this: &Rc<dyn IsApp>, primary_focus: bool) -> Option<Rc<dyn IsView>> {
        let focus = this.app().focus.borrow();
        if primary_focus {
            focus.primary.upgrade()
        } else {
            focus.secondary.upgrade()
        }
    }

    pub fn focus_impl(this: &Rc<dyn IsApp>, view: Option<&Rc<dyn IsView>>, primary_focus: bool) {
        view.map(|x| assert!(option_addr_eq(x.root().app().map(|x| Rc::as_ptr(&x)), Some(Rc::as_ptr(this)))));
        let prev = {
            let mut focus = this.app().focus.borrow_mut();
            assert!(!focus.changing);
            focus.changing = true;
            if primary_focus {
                replace(&mut focus.primary, <rc::Weak::<View>>::new())
            } else {
                replace(&mut focus.secondary, <rc::Weak::<View>>::new())
            }.upgrade()
        };
        if let Some(prev) = prev {
            prev._set_is_focused(primary_focus, false);
        }
        {
            let mut focus = this.app().focus.borrow_mut();
            if primary_focus {
                focus.primary = view.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade);
            } else {
                focus.secondary = view.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade);
            };
        }
        if let Some(next) = view {
            next._set_is_focused(primary_focus, true);
        }
        this.app().focus.borrow_mut().changing = false;
    }
}
