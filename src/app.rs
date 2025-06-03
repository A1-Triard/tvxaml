use basic_oop::{class_unsafe, import, Vtable};
use int_vec_2d::{Rect, Vector, Point};
use std::cell::{Cell, RefCell};
use tvxaml_screen_base::Screen;

import! { pub app:
    use [obj basic_oop::obj];
    use std::rc::Rc;
    use tvxaml_screen_base::Error as tvxaml_screen_base_Error;
    use crate::view::TView;
}

#[class_unsafe(inherits_Obj)]
pub struct App {
    screen: RefCell<Box<dyn Screen>>,
    exit_code: Cell<Option<u8>>,
    root: Rc<dyn TView>,
    invalidated_rect: Cell<Rect>,
    #[non_virt]
    root: fn() -> Rc<dyn TView>,
    #[non_virt]
    run: fn() -> Result<u8, tvxaml_screen_base_Error>,
    #[non_virt]
    exit: fn(exit_code: u8),
    #[non_virt]
    quit: fn(),
    #[non_virt]
    invalidate_render: fn(rect: Rect),
}

impl App {
    pub fn new(screen: Box<dyn Screen>, root: Rc<dyn TView>) -> Rc<dyn TApp> {
        Rc::new(unsafe { Self::new_raw(screen, root, APP_VTABLE.as_ptr()) })
    }

    pub unsafe fn new_raw(screen: Box<dyn Screen>, root: Rc<dyn TView>, vtable: Vtable) -> Self {
        App {
            obj: unsafe { Obj::new_raw(vtable) },
            screen: RefCell::new(screen),
            exit_code: Cell::new(None),
            root,
            invalidated_rect: Cell::new(Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }),
        }
    }

    pub fn root_impl(this: &Rc<dyn TApp>) -> Rc<dyn TView> {
        this.app().root.clone()
    }

    pub fn run_impl(this: &Rc<dyn TApp>) -> Result<u8, tvxaml_screen_base_Error> {
        loop {
            if let Some(exit_code) = this.app().exit_code.get() {
                break Ok(exit_code);
            }
            if let Err(e) = this.app().screen.borrow_mut().update(None, true) {
                break Err(e);
            }
        }
    }

    pub fn exit_impl(this: &Rc<dyn TApp>, exit_code: u8) {
        this.app().exit_code.set(Some(exit_code));
    }

    pub fn quit_impl(this: &Rc<dyn TApp>) {
        this.exit(0);
    }

    pub fn invalidate_render_impl(this: &Rc<dyn TApp>, rect: Rect) {
        let union = this.app().invalidated_rect.get().union(rect);
        let app_rect = Rect { tl: Point { x: 0, y: 0 }, size: this.app().screen.borrow().size() };
        let invalidated_rect = union.map_or(app_rect, |x| x.either(
            |band| band.either(|h| app_rect.intersect_h_band(h), |v| app_rect.intersect_v_band(v)),
            |x| app_rect.intersect(x)
        ));
        this.app().invalidated_rect.set(invalidated_rect);
    }
}
