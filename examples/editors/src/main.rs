#![feature(macro_metavar_expr_concat)]

use dynamic_cast::dyn_cast_rc;
use std::process::ExitCode;
use std::rc::{self, Rc};
use timer_no_std::MonoClock;
use tvxaml::base::Key;
use tvxaml::app::{App, AppExt};
use tvxaml::check_box::{IsCheckBox, CheckBoxExt};
use tvxaml::input_line::{IsInputLine, InputLineExt};
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
    let edit1: Rc<dyn IsInputLine> = dyn_cast_rc(names.find("edit1").unwrap().clone()).unwrap();
    let edit2: Rc<dyn IsInputLine> = dyn_cast_rc(names.find("edit2").unwrap().clone()).unwrap();
    edit1.text().set("Abcd".to_string());
    edit2.text().set("Abcd".to_string());
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
    app.run(&mut clock, &root, Some(&mut || { app.focus_next(true); }))
}
