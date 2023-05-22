extern crate exec;
extern crate x11;

use std::env;
use std::process;
use std::ptr;

const KEYID_LSUPER: u32 = 130;

fn usage(prog: &String) -> () {
    println!("usage: {prog} [cmd...]");
    println!("Launch command if only Left Super is pressed then released.");
}

fn x11_open_display() -> Option<*mut x11::xlib::Display> {
    unsafe {
        let display = x11::xlib::XOpenDisplay(ptr::null());
        if display == ptr::null_mut() {
            return None;
        }
        return Some(display);
    }
}

fn x11_close_display(display: *mut x11::xlib::Display) {
    unsafe {
        x11::xlib::XCloseDisplay(display);
    }
}

fn x11_check_lsuper() -> Option<bool> {
    match x11_open_display() {
        Some(display) => {
            loop {
                let mut keymap: [i8; 32] = [0; 32];
                unsafe {
                    x11::xlib::XQueryKeymap(display, keymap.as_mut_ptr());
                }
                let (count, keyid, _) : (u32, u32, bool) = keymap.iter().fold(
                    (0, 0, false), |x, y| match x {
                        (c, k, s) => (c + y.count_ones(),
                                      if s {k} else {k + y.leading_zeros()},
                                      if s {s} else {y.leading_zeros() < 8}),
                });
                if count == 0 {
                    x11_close_display(display);
                    return Some(true);
                }
                if count > 1 || keyid != KEYID_LSUPER {
                    x11_close_display(display);
                    return Some(false);
                }
            }
        },
        None => {
            println!("Error: can't open display");
            return None;
        }

    }
}

fn single_release_launch(cmd: &String, argv: &[String]) -> () {
    match x11_check_lsuper() {
        Some(ret) => {
            if ret == true {
                println!("Error: {}", exec::Command::new(cmd).args(argv).exec());
            }
        },
        None => (),
    }
}

fn main() -> () {
    let args: Vec<String> = env::args().collect();
    match &args[..] {
        [prog] => usage(prog),
        [_, cmd] => single_release_launch(cmd, &[]),
        [_, cmd, argv @ ..] => single_release_launch(cmd, argv),
        _ => (),
    }
    process::exit(1);
}
