use basic_oop::{class_unsafe, import, Vtable};
use std::cell::RefCell;
use std::mem::replace;
use std::ptr::addr_eq;
use std::rc::{self};
use tvxaml_screen_base::{Vector, Point, Screen, Event};
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
    use std::rc::Rc;
    use tvxaml_screen_base::Rect;
    use tvxaml_screen_base::Error as tvxaml_screen_base_Error;
    use crate::view::IsView;
}

struct AppData {
    app_rect: Rect,
    changing_focus: bool,
    primary_focus: rc::Weak<dyn IsView>,
    secondary_focus: rc::Weak<dyn IsView>,
    cursor: Option<Point>,
    exit_code: Option<u8>,
    invalidated_rect: Rect,
    pre_process: Vec<rc::Weak<dyn IsView>>,
    post_process: Vec<rc::Weak<dyn IsView>>,
}

#[class_unsafe(inherits_Obj)]
pub struct App {
    data: RefCell<AppData>,
    screen: RefCell<Box<dyn Screen>>,
    #[non_virt]
    run: fn(root: Rc<dyn IsView>, init: Option<&mut dyn FnMut()>) -> Result<u8, tvxaml_screen_base_Error>,
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
    #[non_virt]
    _add_pre_process: fn(view: &Rc<dyn IsView>),
    #[non_virt]
    _remove_pre_process: fn(view: &Rc<dyn IsView>),
    #[non_virt]
    _add_post_process: fn(view: &Rc<dyn IsView>),
    #[non_virt]
    _remove_post_process: fn(view: &Rc<dyn IsView>),
}

impl App {
    pub fn new(screen: Box<dyn Screen>) -> Rc<dyn IsApp> {
        Rc::new(unsafe { Self::new_raw(screen, APP_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(screen: Box<dyn Screen>, vtable: Vtable) -> Self {
        let app_rect = Rect { tl: Point { x: 0, y: 0 }, size: screen.size() };
        App {
            obj: unsafe { Obj::new_raw(vtable) },
            screen: RefCell::new(screen),
            data: RefCell::new(AppData {
                app_rect,
                changing_focus: false,
                primary_focus: <rc::Weak::<View>>::new(),
                secondary_focus: <rc::Weak::<View>>::new(),
                cursor: None,
                exit_code: None,
                invalidated_rect: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                pre_process: Vec::new(),
                post_process: Vec::new(),
            }),
        }
    }

    pub fn into_inner(self) -> Box<dyn Screen> {
        self.screen.into_inner()
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

    pub fn run_impl(
        this: &Rc<dyn IsApp>,
        root: Rc<dyn IsView>,
        init: Option<&mut dyn FnMut()>
    ) -> Result<u8, tvxaml_screen_base_Error> {
        assert!(root.app().is_none(), "root already attached to another app");
        root._attach_to_app(this);
        init.map(|x| x());
        let res = loop {
            if let Some(exit_code) = this.app().data.borrow_mut().exit_code.take() {
                break Ok(exit_code);
            }
            let screen_size = this.app().screen.borrow().size();
            root.measure(Some(screen_size.x), Some(screen_size.y));
            root.arrange(Rect { tl: Point { x: 0, y: 0 }, size: screen_size });
            let bounds = root.margin().shrink_rect(root.render_bounds());
            let mut screen = this.app().screen.borrow_mut();
            let (cursor, invalidated_rect) = {
                let mut data = this.app().data.borrow_mut();
                (
                    data.cursor,
                    replace(&mut data.invalidated_rect, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() })
                )
            };
            let mut rp = RenderPort {
                screen: screen.as_mut(),
                invalidated_rect,
                bounds: bounds.intersect(Rect { tl: Point { x: 0, y: 0 }, size: screen_size }),
                offset: Vector { x: bounds.l(), y: bounds.t() },
                cursor,
            };
            Self::render(&root, &mut rp);
            let cursor = rp.cursor;
            this.app().data.borrow_mut().cursor = cursor;
            match screen.update(cursor, true) {
                Err(e) => break Err(e),
                Ok(Some(Event::Resize)) => {
                    let app_rect = Rect { tl: Point { x: 0, y: 0 }, size: this.app().screen.borrow().size() };
                    this.app().data.borrow_mut().app_rect = app_rect;
                },
                Ok(Some(Event::Key(n, key))) => {
                    'c: for _ in 0 .. n.get() {
                        for pre_process in this.app().data.borrow().pre_process.clone() {
                            let pre_process = pre_process.upgrade().unwrap();
                            if pre_process.is_enabled() && pre_process.pre_process_key(key) { continue 'c; }
                        }
                        if let Some(focused) = this.focused(true) {
                            if focused.is_enabled() && focused._raise_key(key) { continue; }
                        }
                        if let Some(focused) = this.focused(false) {
                            if focused.is_enabled() && focused._raise_key(key) { continue; }
                        }
                        for post_process in this.app().data.borrow().post_process.clone() {
                            let post_process = post_process.upgrade().unwrap();
                            if post_process.is_enabled() && post_process.post_process_key(key) { continue 'c; }
                        }
                    }
                },
                _ => { },
            }
        };
        this.focus(None, true);
        this.focus(None, false);
        root._detach_from_app();
        res
    }

    pub fn exit_impl(this: &Rc<dyn IsApp>, exit_code: u8) {
        this.app().data.borrow_mut().exit_code = Some(exit_code);
    }

    pub fn quit_impl(this: &Rc<dyn IsApp>) {
        this.exit(0);
    }

    pub fn invalidate_render_impl(this: &Rc<dyn IsApp>, rect: Rect) {
        let mut data = this.app().data.borrow_mut();
        let union = data.invalidated_rect.union_intersect(rect, data.app_rect);
        data.invalidated_rect = union;
    }

    pub fn focused_impl(this: &Rc<dyn IsApp>, primary_focus: bool) -> Option<Rc<dyn IsView>> {
        let data = this.app().data.borrow();
        if primary_focus {
            data.primary_focus.upgrade()
        } else {
            data.secondary_focus.upgrade()
        }
    }

    pub fn focus_impl(this: &Rc<dyn IsApp>, view: Option<&Rc<dyn IsView>>, primary_focus: bool) {
        view.map(|x| assert!(option_addr_eq(x.app().map(|x| Rc::as_ptr(&x)), Some(Rc::as_ptr(this)))));
        let prev = {
            let mut data = this.app().data.borrow_mut();
            assert!(!data.changing_focus);
            data.changing_focus = true;
            if primary_focus {
                replace(&mut data.primary_focus, <rc::Weak::<View>>::new())
            } else {
                replace(&mut data.secondary_focus, <rc::Weak::<View>>::new())
            }.upgrade()
        };
        if let Some(prev) = prev {
            prev._set_is_focused(primary_focus, false);
        }
        {
            let mut data = this.app().data.borrow_mut();
            if primary_focus {
                data.primary_focus = view.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade);
            } else {
                data.secondary_focus = view.map_or_else(|| <rc::Weak::<View>>::new(), Rc::downgrade);
            };
        }
        if let Some(next) = view {
            next._set_is_focused(primary_focus, true);
        }
        this.app().data.borrow_mut().changing_focus = false;
    }

    pub fn _add_pre_process_impl(this: &Rc<dyn IsApp>, view: &Rc<dyn IsView>) {
        this.app().data.borrow_mut().pre_process.push(Rc::downgrade(view));
    }

    pub fn _remove_pre_process_impl(this: &Rc<dyn IsApp>, view: &Rc<dyn IsView>) {
        let mut data = this.app().data.borrow_mut();
        let view_as_ptr = Rc::as_ptr(view);
        let index = data.pre_process.iter().position(|x| {
            let Some(x) = x.upgrade() else { return false; };
            addr_eq(Rc::as_ptr(&x), view_as_ptr)
        }).unwrap();
        data.pre_process.swap_remove(index);
    }

    pub fn _add_post_process_impl(this: &Rc<dyn IsApp>, view: &Rc<dyn IsView>) {
        this.app().data.borrow_mut().post_process.push(Rc::downgrade(view));
    }

    pub fn _remove_post_process_impl(this: &Rc<dyn IsApp>, view: &Rc<dyn IsView>) {
        let mut data = this.app().data.borrow_mut();
        let view_as_ptr = Rc::as_ptr(view);
        let index = data.post_process.iter().position(|x| {
            let Some(x) = x.upgrade() else { return false; };
            addr_eq(Rc::as_ptr(&x), view_as_ptr)
        }).unwrap();
        data.post_process.swap_remove(index);
    }
}
