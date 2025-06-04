use basic_oop::{class_unsafe, import, Vtable};
use int_vec_2d::{Vector, Point};
use std::cell::{Cell, RefCell};
use tvxaml_screen_base::Screen;
use crate::render_port::RenderPort;
use crate::view::ViewExt;

import! { pub app:
    use [obj basic_oop::obj];
    use int_vec_2d::Rect;
    use std::rc::Rc;
    use tvxaml_screen_base::Error as tvxaml_screen_base_Error;
    use crate::view::TView;
}

#[class_unsafe(inherits_Obj)]
pub struct App {
    screen: RefCell<Box<dyn Screen>>,
    cursor: Cell<Option<Point>>,
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
            cursor: Cell::new(None),
            exit_code: Cell::new(None),
            root,
            invalidated_rect: Cell::new(Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }),
        }
    }

    pub fn root_impl(this: &Rc<dyn TApp>) -> Rc<dyn TView> {
        this.app().root.clone()
    }

    fn render(view: &Rc<dyn TView>, rp: &mut RenderPort) {
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

    pub fn run_impl(this: &Rc<dyn TApp>) -> Result<u8, tvxaml_screen_base_Error> {
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

    pub fn exit_impl(this: &Rc<dyn TApp>, exit_code: u8) {
        this.app().exit_code.set(Some(exit_code));
    }

    pub fn quit_impl(this: &Rc<dyn TApp>) {
        this.exit(0);
    }

    pub fn invalidate_render_impl(this: &Rc<dyn TApp>, rect: Rect) {
        let app_rect = Rect { tl: Point { x: 0, y: 0 }, size: this.app().screen.borrow().size() };
        let union = this.app().invalidated_rect.get().union_intersect(rect, app_rect);
        this.app().invalidated_rect.set(union);
    }
}
