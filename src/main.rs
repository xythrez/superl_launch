extern crate exec;
extern crate x11;

use std::env;
use std::process;
use std::ptr;
use std::thread;
use std::time;

const KEYID_LSUPER: u32 = 130;
const MOUSE_MASK_LSUPER: u32 = 64;

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
    let sleep_time = time::Duration::from_millis(10);
    match x11_open_display() {
        Some(display) => {
            loop {
                let mut root_return: u64 = 0;
                let mut child_return: u64 = 0;
                let mut win_x: i32 = 0;
                let mut win_y: i32 = 0;
                let mut root_x: i32 = 0;
                let mut root_y: i32 = 0;
                let mut mask_return: u32 = 0;
                let mut keymap: [i8; 32] = [0; 32];
                unsafe {
                    let win: u64 = x11::xlib::XDefaultRootWindow(display);
                    x11::xlib::XQueryPointer(display, win,
                                             &mut root_return,
                                             &mut child_return,
                                             &mut root_x,
                                             &mut root_y,
                                             &mut win_x,
                                             &mut win_y,
                                             &mut mask_return);
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
                if count > 1 || keyid != KEYID_LSUPER
                    || mask_return != MOUSE_MASK_LSUPER {
                        x11_close_display(display);
                        return Some(false);
                }
                thread::sleep(sleep_time);
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
