use basic_oop::{class_unsafe, import, Vtable};
use std::cell::RefCell;
use std::mem::replace;
use std::ptr::addr_eq;
use std::rc::{self};
use timer_no_std::{MonoClock, MonoTime};
use crate::arena::{Handle, Registry};
use crate::base::{Vector, Point, Screen, Event, Key};
use crate::render_port::RenderPort;
use crate::view::{View, ViewExt, SecondaryFocusKeys};

fn option_addr_eq<T, U>(p: Option<*const T>, q: Option<*const U>) -> bool where T: ?Sized, U: ?Sized {
    if p.is_none() && q.is_none() { return true; }
    let Some(p) = p else { return false; };
    let Some(q) = q else { return false; };
    addr_eq(p, q)
}

import! { pub app:
    use [obj basic_oop::obj];
    use std::rc::Rc;
    use crate::base::Rect;
    use crate::base::Error as tvxaml_base_Error;
    use crate::view::IsView;
}

const FPS: u16 = 40;

struct TimerData {
    start: MonoTime,
    span_ms: u16,
    alarm: Box<dyn FnOnce()>,
}

#[derive(Debug)]
pub struct Timer(Handle);

impl Timer {
    pub fn new(
        app: &Rc<dyn IsApp>,
        span_ms: u16,
        alarm: Box<dyn FnOnce()>
    ) -> Self {
        let mut app = app.app().data.borrow_mut();
        let start = app.clock.as_ref().expect("app is not running").time();
        app.timers.insert(move |handle| (TimerData {
            start,
            span_ms,
            alarm
        }, Timer(handle)))
    }

    pub fn drop_timer(self, app: &Rc<dyn IsApp>) {
        app.app().data.borrow_mut().timers.remove(self.0);
    }
}

struct AppData {
    root: Option<Rc<dyn IsView>>,
    app_rect: Rect,
    changing_focus: bool,
    primary_focus: rc::Weak<dyn IsView>,
    secondary_focus: rc::Weak<dyn IsView>,
    cursor: Option<Point>,
    exit_code: Option<u8>,
    invalidated_rect: Rect,
    pre_process: Vec<rc::Weak<dyn IsView>>,
    post_process: Vec<rc::Weak<dyn IsView>>,
    timers: Registry<TimerData>,
    clock: Option<MonoClock>,
}

#[class_unsafe(inherits_Obj)]
pub struct App {
    data: RefCell<AppData>,
    screen: RefCell<Box<dyn Screen>>,
    #[non_virt]
    run: fn(
        clock: &mut Option<MonoClock>,
        root: &Rc<dyn IsView>,
        init: Option<&mut dyn FnMut()>,
    ) -> Result<u8, tvxaml_base_Error>,
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
    #[non_virt]
    focus_next: fn(primary_focus: bool),
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
                root: None,
                app_rect,
                changing_focus: false,
                primary_focus: <rc::Weak::<View>>::new(),
                secondary_focus: <rc::Weak::<View>>::new(),
                cursor: None,
                exit_code: None,
                invalidated_rect: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                pre_process: Vec::new(),
                post_process: Vec::new(),
                clock: None,
                timers: Registry::new(),
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
        clock: &mut Option<MonoClock>,
        root: &Rc<dyn IsView>,
        init: Option<&mut dyn FnMut()>
    ) -> Result<u8, tvxaml_base_Error> {
        assert!(root.app().is_none(), "root already attached to another app");
        let old_root = this.app().data.borrow_mut().root.replace(root.clone());
        if let Some(old_root) = old_root {
            this.app().data.borrow_mut().root = Some(old_root);
            panic!("app is already running");
        }
        this.app().data.borrow_mut().clock = Some(clock.take().expect("no clock"));
        root._attach_to_app(this);
        init.map(|x| x());
        let mut time = this.app().data.borrow().clock.as_ref().unwrap().time();
        let res = loop {
            if let Some(exit_code) = this.app().data.borrow_mut().exit_code.take() {
                break Ok(exit_code);
            }
            loop {
                let alarm = {
                    let mut data = this.app().data.borrow_mut();
                    let time = data.clock.as_ref().unwrap().time();
                    let timer = data.timers.items().iter()
                        .find(|(_, data)| time.delta_ms_u16(data.start).unwrap_or(u16::MAX) >= data.span_ms)
                        .map(|(handle, _)| handle);
                    timer.map(|x| data.timers.remove(x).alarm)
                };
                if let Some(alarm) = alarm {
                    alarm();
                } else {
                    break;
                }
            }
            let has_timers = !this.app().data.borrow().timers.items().is_empty();
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
            Self::render(root, &mut rp);
            let cursor = rp.cursor;
            this.app().data.borrow_mut().cursor = cursor;
            match screen.update(cursor, !has_timers) {
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
                        match key {
                            Key::Tab => this.focus_next(true),
                            Key::Left | Key::Right | Key::Up | Key::Down => {
                                let primary_focus = if let Some(sfr) = root._secondary_focus_root() {
                                    match (key, sfr.secondary_focus_keys()) {
                                        (Key::Left, SecondaryFocusKeys::LeftRight) => false,
                                        (Key::Right, SecondaryFocusKeys::LeftRight) => false,
                                        (Key::Up, SecondaryFocusKeys::LeftRight) => true,
                                        (Key::Down, SecondaryFocusKeys::LeftRight) => true,
                                        (Key::Left, SecondaryFocusKeys::UpDown) => true,
                                        (Key::Right, SecondaryFocusKeys::UpDown) => true,
                                        (Key::Up, SecondaryFocusKeys::UpDown) => false,
                                        (Key::Down, SecondaryFocusKeys::UpDown) => false,
                                        _ => panic!(),
                                    }
                                } else {
                                    true
                                };
                                this.focus_next(primary_focus);
                            },
                            _ => { },
                        }
                    }
                },
                _ => { },
            }
            let data = this.app().data.borrow();
            let clock = data.clock.as_ref().unwrap();
            let ms = time.split_ms_u16(clock).unwrap_or(u16::MAX);
            if has_timers {
                assert!(FPS != 0 && u16::MAX / FPS > 8);
                clock.sleep_ms_u16((1000 / FPS).saturating_sub(ms));
            }
        };
        this.focus(None, true);
        this.focus(None, false);
        root._detach_from_app();
        let mut data = this.app().data.borrow_mut();
        *clock = Some(data.clock.take().unwrap());
        data.root = None;
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

    pub fn focus_next_impl(this: &Rc<dyn IsApp>, primary_focus: bool) {
        let sfr = {
            let data = this.app().data.borrow();
            let root = data.root.as_ref().expect("app is not running");
            root._secondary_focus_root()
        };
        let focused = if let Some(focused) = this.focused(primary_focus) {
            focused
        } else {
            let mut root = this.app().data.borrow().root.clone().unwrap();
            if !primary_focus {
                let Some(sfr) = sfr.clone() else { return; };
                root = sfr;
            }
            if !root.is_enabled() { return; }
            if root.allow_focus() {
                this.focus(Some(&root), primary_focus);
                return;
            }
            root
        };
        if primary_focus {
            if let Some(sfr) = sfr.as_ref() {
                if sfr.is_visual_ancestor_of(focused.clone()) { return; }
            }
        } else {
            let Some(sfr) = sfr.as_ref() else { return; };
            if !sfr.is_visual_ancestor_of(focused.clone()) { return; }
        }
        let mut focus = focused.clone();
        loop {
            if 
                   focus.visual_children_count() != 0
                && (!primary_focus || !option_addr_eq(Some(Rc::as_ptr(&focus)), sfr.as_ref().map(Rc::as_ptr)))
            {
                focus = focus.visual_child(0);
            } else {
                loop {
                    if
                           !primary_focus
                        && option_addr_eq(Some(Rc::as_ptr(&focus)), sfr.as_ref().map(Rc::as_ptr))
                    {
                        break;
                    }
                    let Some(parent) = focus.visual_parent() else { break; };
                    let children_count = parent.visual_children_count();
                    debug_assert!(children_count != 0);
                    let index = (0 .. children_count).into_iter()
                        .find(|&i| addr_eq(Rc::as_ptr(&parent.visual_child(i)), Rc::as_ptr(&focus)))
                        .unwrap();
                    if index == children_count - 1 {
                        focus = parent;
                    } else {
                        focus = parent.visual_child(index + 1);
                        break;
                    }
                }
            }
            if addr_eq(Rc::as_ptr(&focus), Rc::as_ptr(&focused)) { return; }
            if primary_focus && option_addr_eq(Some(Rc::as_ptr(&focus)), sfr.as_ref().map(Rc::as_ptr)) {
                continue;
            }
            if focus.allow_focus() && focus.is_enabled() {
                this.focus(Some(&focus), primary_focus);
                return;
            }
        }
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
