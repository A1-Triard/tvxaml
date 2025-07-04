#![feature(macro_metavar_expr_concat)]

use dynamic_cast::dyn_cast_rc;
use std::process::ExitCode;
use std::rc::Rc;
use timer_no_std::MonoClock;
use tvxaml::base::{Key, Vector};
use tvxaml::app::{App, AppExt};
use tvxaml::canvas::{IsCanvasLayout, CanvasLayoutExt};
use tvxaml::template::Template;
use tvxaml::view::{IsView, ViewExt};
use tvxaml::xaml::{self};

mod floating_frame;

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
    let frame_layout: Rc<dyn IsCanvasLayout> = dyn_cast_rc(names.find("FrameLayout").unwrap().clone()).unwrap();
    let app = App::new(screen);
    let root: Rc<dyn IsView> = dyn_cast_rc(root).unwrap();
    {
        let app = Rc::downgrade(&app);
        let frame_layout = Rc::downgrade(&frame_layout);
        root.handle_key(Some(Box::new(move |key, _| {
            if key == Key::Escape {
                app.upgrade().unwrap().quit();
                return true;
            }
            let offset = match key {
                Key::Left | Key::Char('h') => Some(Vector { x: -2, y: 0 }),
                Key::Right | Key::Char('l') => Some(Vector { x: 2, y: 0 }),
                Key::Up | Key::Char('k') => Some(Vector { x: 0, y: -1 }),
                Key::Down | Key::Char('j') => Some(Vector { x: 0, y: 1 }),
                _ => None
            };
            if let Some(offset) = offset {
                let frame_layout = frame_layout.upgrade().unwrap();
                frame_layout.set_tl(frame_layout.tl().offset(offset));
                true
            } else {
                false
            }
        })));
    }
    app.run(&mut clock, &root, Some(&mut || app.focus(Some(&root), Some(true))))
}
