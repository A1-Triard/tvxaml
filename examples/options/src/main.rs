#![feature(macro_metavar_expr_concat)]

use dynamic_cast::dyn_cast_rc;
use std::process::ExitCode;
use std::rc::{self, Rc};
use timer_no_std::MonoClock;
use tvxaml::base::Key;
use tvxaml::app::{App, AppExt};
use tvxaml::check_box::{IsCheckBox, CheckBoxExt};
use tvxaml::template::Template;
use tvxaml::view::{IsView, ViewExt};
use tvxaml::xaml::{self};

fn main() -> ExitCode {
    ExitCode::from(start_and_print_err())
}

fn start_and_print_err() -> u8 {
    match start() {
        Err(e) => {
           eprintln!("{e}");
            1
        },
        Ok(exit_code) => exit_code
    }
}

fn start() -> Result<u8, tvxaml::base::Error> {
    let mut clock = Some(unsafe { MonoClock::new() });
    let screen = unsafe { tvxaml_screen_ncurses::init(None, None) }?;
    let xaml = include_str!("ui.xaml");
    let ui: Box<dyn Template> = xaml::from_str(xaml).unwrap();
    let (root, names) = ui.load_root();
    let app = App::new(screen);
    let root: Rc<dyn IsView> = dyn_cast_rc(root).unwrap();
    {
        let app = Rc::downgrade(&app);
        root.handle_key(Some(Box::new(move |key, _| {
            if key == Key::Escape {
                app.upgrade().unwrap().quit();
                return true;
            }
            false
        })));
    }
    let rbcb: Rc<dyn IsCheckBox> = dyn_cast_rc(names.find("rbcb").unwrap().clone()).unwrap();
    let rb: rc::Weak<dyn IsView> = Rc::downgrade(&dyn_cast_rc(names.find("rb").unwrap().clone()).unwrap());
    let rbcb_ref = Rc::downgrade(&rbcb);
    rbcb.handle_toggle(Some(Box::new(move || {
        let rb = rb.upgrade().unwrap();
        let rbcb = rbcb_ref.upgrade().unwrap();
        rb.set_is_enabled(rbcb.is_checked());
    })));
    app.run(&mut clock, &root, Some(&mut || { app.focus_next(true); app.focus_next(false); }))
}
