use rmap::automata::OutputId;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;

use std::{thread::sleep, time::Duration};

use winapi::{
    ctypes::*,
    shared::{minwindef::*, windef::*},
    um::winuser::*,
};

use crossbeam::channel;

use rmap::automata::{Automata, AutomataBuilder};

pub enum Commands {
    Bind { id: u32, tgt: String },
    Poll,
    PollAxis { id: u32 },
}

fn main() {
    let sdl_context = sdl2::init().unwrap();

    let gc = sdl_context.game_controller().unwrap();
    let stick = sdl_context.joystick().unwrap();
    let events = sdl_context.event().unwrap();
    let vid = sdl_context.video().unwrap();
    // let mouse = sdl_context.mouse().unwrap();
    // let keyboard =sdl_context.keyboard().unwrap();
    let mut epump = sdl_context.event_pump().unwrap();

    // let mut window = sdl2::video::WindowBuilder::new(&vid, "idk what im doing", 1, 1);
    //window.hidden();
    // let window = window.build().unwrap();

    let is_en = epump.is_event_enabled(sdl2::event::EventType::KeyDown);
    println!("key down enabled: {}", is_en);

    /*
    let code = 0x41;
    println!("sending A down in 1 sec");
    sleep(Duration::from_millis(1000));
    send_keybd_input(KEYEVENTF_SCANCODE, code);
    // sleep(Duration::from_millis(10));
    // send_keybd_input(KEYEVENTF_SCANCODE, code);
    // sleep(Duration::from_millis(10));
    // send_keybd_input(KEYEVENTF_SCANCODE, code);
    // sleep(Duration::from_millis(10));
    // send_keybd_input(KEYEVENTF_SCANCODE, code);

    println!("sending A up in 1 sec");
    sleep(Duration::from_millis(1000));
    send_keybd_input(KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP, code);
    */

    let (msg_tx, msg_rx) = channel::unbounded::<Commands>();

    //dbg!(epump.keyboard_state().is_scancode_pressed(Scancode::A));
    for e in epump.poll_iter() {
        println!("{:?}", e);
    }

    let mut autos: Vec<Automata> = Vec::new();

    let automata = {
        use sdl2::controller::Button;
        let mut builder = AutomataBuilder::default();

        let (in_a_down, in_a_up) = builder.new_input(Button::A);
        let (in_r_down, in_r_up) = builder.new_input(Button::RightShoulder);

        let out_a = builder.new_output();
        let out_ra = builder.new_output();

        let s_0 = builder.new_state();
        let s_r = builder.new_state();
        let s_0a = builder.new_state();
        let s_ra = builder.new_state();

        s_0.silent(in_r_down, s_r.id);
        s_0.transition(in_a_down, s_0a.id, out_a.id);

        s_r.silent(in_r_up, s_0.id);
        s_0a.silent(in_a_up, s_0.id);

        s_r.transition(in_a_down, s_ra.id, out_ra.id);
        s_ra.silent(in_a_up, s_r.id);

        Automata::from_builder(builder)
    };

    autos.push(automata);

    let pad = stick.open(0).unwrap();
    let pad0 = gc.open(0).unwrap();
    dbg!(pad.num_axes());
    let num_axes = pad.num_axes();
    loop {
        epump.pump_events();

        let mut print = false;

        while let Ok(msg) = msg_rx.try_recv() {
            match msg {
                Commands::Bind { id, tgt } => {
                    //
                    println!("binding {} to {}", id, tgt);
                }
                Commands::PollAxis { id } => {
                    if let Ok(val) = pad.axis(id) {
                        println!("axis {} - {}", id, val);
                    }
                }
                Commands::Poll => {
                    print = true;
                }
            }
            //
        }

        let mut exit = false;
        //for ai in 0..num_axes {
        //    println!("{:?}", pad.axis(ai));
        //}
        for e in epump.poll_iter() {
            match &e {
                Event::Quit { .. } => {
                    exit = true;
                }
                Event::ControllerAxisMotion {
                    timestamp,
                    which,
                    axis,
                    value,
                } => {
                    //
                    // println!("{:?}", e);
                }
                Event::ControllerButtonDown {
                    timestamp,
                    which,
                    button,
                } => {
                    println!("button down!");
                    for auto in autos.iter_mut() {
                        let input_id = auto.map_input(*button, true);
                        println!("mapped input: {:?}", input_id);
                        let output = auto.step(*button, true);

                        match output {
                            Some(OutputId(0)) => {
                                println!("Just A down");
                            }
                            Some(OutputId(1)) => {
                                println!("Just A up");
                            }
                            Some(OutputId(2)) => {
                                println!("L + A down");
                            }
                            Some(OutputId(3)) => {
                                println!("L + A up");
                            }
                            _ =>(),

                        }
                        println!("binding output: {:?}", output)
                    }
                }
                Event::ControllerButtonUp {
                    timestamp,
                    which,
                    button,
                } => {
                    println!("button up!");
                    for auto in autos.iter_mut() {
                        let input_id = auto.map_input(*button, false);
                        println!("mapped input: {:?}", input_id);
                        let output = auto.step(*button, false);
                        println!("binding output: {:?}", output)
                    }
                }
                Event::ControllerDeviceAdded { timestamp, which } => {
                    //
                    println!("{:?}", e);
                }
                Event::ControllerDeviceRemoved { timestamp, which } => {
                    //
                    println!("{:?}", e);
                }
                Event::ControllerDeviceRemapped { timestamp, which } => {
                    //
                    println!("{:?}", e);
                }
                Event::TextEditing { timestamp, window_id, text, start, length } => {
                    //
                    println!("{:?}", e);
                }
                Event::TextInput { timestamp, window_id, text } => {
                    //
                    println!("{:?}", e);
                }
                /*
                Event::Quit { timestamp } => todo!(),
                Event::AppTerminating { timestamp } => todo!(),
                Event::AppLowMemory { timestamp } => todo!(),
                Event::AppWillEnterBackground { timestamp } => todo!(),
                Event::AppDidEnterBackground { timestamp } => todo!(),
                Event::AppWillEnterForeground { timestamp } => todo!(),
                Event::AppDidEnterForeground { timestamp } => todo!(),
                Event::Window { timestamp, window_id, win_event } => todo!(),
                Event::FingerDown { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => todo!(),
                Event::FingerUp { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => todo!(),
                Event::FingerMotion { timestamp, touch_id, finger_id, x, y, dx, dy, pressure } => todo!(),
                Event::DollarGesture { timestamp, touch_id, gesture_id, num_fingers, error, x, y } => todo!(),
                Event::DollarRecord { timestamp, touch_id, gesture_id, num_fingers, error, x, y } => todo!(),
                Event::MultiGesture { timestamp, touch_id, d_theta, d_dist, x, y, num_fingers } => todo!(),
                Event::ClipboardUpdate { timestamp } => todo!(),
                Event::DropFile { timestamp, window_id, filename } => todo!(),
                Event::DropText { timestamp, window_id, filename } => todo!(),
                Event::DropBegin { timestamp, window_id } => todo!(),
                Event::DropComplete { timestamp, window_id } => todo!(),
                Event::AudioDeviceAdded { timestamp, which, iscapture } => todo!(),
                Event::AudioDeviceRemoved { timestamp, which, iscapture } => todo!(),
                Event::RenderTargetsReset { timestamp } => todo!(),
                Event::RenderDeviceReset { timestamp } => todo!(),
                Event::User { timestamp, window_id, type_, code, data1, data2 } => todo!(),
                Event::Unknown { timestamp, type_ } => todo!(),
                */
                _ => (),
                //
            }
            if print {
                println!("{:?}", e);
            }
        }

        if exit {
            break;
        }

        sleep(Duration::from_millis(16));
    }

    // println!("joysticks: {:?}", gc.num_joysticks());

    // cont.update();

    // println!("here we go again");
}


fn send_mouse_input(flags: u32, data: u32, dx: i32, dy: i32) {
    let mut input = INPUT {
        type_: INPUT_MOUSE,
        u: unsafe {
            std::mem::transmute_copy(&MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            })
        },
    };
    unsafe { SendInput(1, &mut input as LPINPUT, std::mem::size_of::<INPUT>() as c_int) };
}

fn send_keybd_input(flags: u32, key_code: u64) {
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe {
            std::mem::transmute_copy(&KEYBDINPUT {
                wVk: 0,
                wScan: MapVirtualKeyW(u64::from(key_code) as u32, 0) as u16,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            })
        },
    };
    unsafe { SendInput(1, &mut input as LPINPUT, std::mem::size_of::<INPUT>() as c_int) };
}
