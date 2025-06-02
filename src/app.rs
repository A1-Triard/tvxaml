use basic_oop::{class_unsafe, import, Vtable};
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
    #[non_virt]
    root: fn() -> Rc<dyn TView>,
    #[non_virt]
    run: fn() -> Result<u8, tvxaml_screen_base_Error>,
    #[non_virt]
    exit: fn(exit_code: u8),
    #[non_virt]
    quit: fn(),
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
}
