#![allow(dead_code)]
use std::error::Error;
use mlua::Lua;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use midir::MidiOutputConnection;
use mlua::Result as LuaResult;
mod ascii;
mod midi;
mod clock;
mod interpreter;
mod config;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", ascii::BANNER);
    let cfg: config::EremitConfig = confy::load("eremit", None)?;
    // Communicating to the clock
    let (sender_to_clock, receiver_for_clock) = mpsc::channel::<clock::ClockControlMessage>();
    // Receiving data from the clock
    let (sender_from_clock, receiver_for_main) = mpsc::channel::<clock::ClockControlMessage>();
    let receiver_for_main = Arc::new(Mutex::new(receiver_for_main));
    let clock = Arc::new(Mutex::new(clock::Clock::new(receiver_for_clock, sender_from_clock)));
    let clock_clone = clock.clone();
    thread::spawn(move || {
        let _ = clock_clone.lock().unwrap().run();
    });
    let mut conn_out: Arc<Mutex<Option<MidiOutputConnection>>> = Arc::new(Mutex::new(None));
    conn_out = midi::setup_midi_connection(conn_out);
    let mut interpreter = interpreter::Interpreter::new();
    let _ = interpreter.register_function("report", {
        let cloned_sender = sender_to_clock.clone();
            move |_lua: &Lua, _args: ()| -> LuaResult<()> {
                cloned_sender.send(clock::ClockControlMessage {
                    name: "report".to_string(),
                    args: vec![],
                }).unwrap();
            Ok(())
        }
    });
    let _ = interpreter.register_function("get_tempo", {
        let cloned_sender = sender_to_clock.clone();
        let cloned_receiver = receiver_for_main.clone();
        move |_lua: &Lua, _args: ()| -> LuaResult<f32> {
            cloned_sender.send(clock::ClockControlMessage {
                name: "get_tempo".to_string(),
                args: vec![],
            }).unwrap();
            let recv = cloned_receiver.lock().unwrap().recv().unwrap();
            match recv.name.as_str() {
                "get_tempo" => {
                    Ok(recv.args[0].parse::<f32>().unwrap())
                },
                _ => {
                    println!("Unknown command: {}", recv.name);
                    Ok(0 as f32)
                }
            }
        }
    });
    let _ = interpreter.register_function("get_phase", {
        let cloned_sender = sender_to_clock.clone();
        let cloned_receiver = receiver_for_main.clone();
        move |_lua: &Lua, _args: ()| -> LuaResult<f64> {
            cloned_sender.send(clock::ClockControlMessage {
                name: "get_phase".to_string(),
                args: vec![],
            }).unwrap();
            let recv = cloned_receiver.lock().unwrap().recv().unwrap();
            match recv.name.as_str() {
                "get_phase" => {
                    Ok(recv.args[0].parse::<f64>().unwrap())
                },
                _ => {
                    println!("Unknown command: {}", recv.name);
                    Ok(0 as f64)
                }
            }
        }
    });
    let _ = interpreter.register_function("set_tempo", {
        let cloned_sender = sender_to_clock.clone();
        move |_lua: &Lua, _args: (f64,)| -> LuaResult<()> {
            cloned_sender.send(clock::ClockControlMessage {
                name: "set_tempo".to_string(),
                args: vec![_args.0.to_string()],
            }).unwrap();
            Ok(())
        }
    });
    let _ = interpreter.register_function("play", {
        let cloned_sender = sender_to_clock.clone();
        move |_lua: &Lua, _args: ()| -> LuaResult<()> {
            cloned_sender.send(clock::ClockControlMessage {
                name: "play".to_string(),
                args: vec![],
            }).unwrap();
            Ok(())
        }
    });
    let _ = interpreter.register_function("sync", {
        let cloned_sender = sender_to_clock.clone();
        move |_lua: &Lua, _args: ()| -> LuaResult<()> {
            cloned_sender.send(clock::ClockControlMessage {
                name: "sync".to_string(),
                args: vec![],
            }).unwrap();
            Ok(())
        }
    });
    let _ = interpreter.register_function("peers", {
        let cloned_sender = sender_to_clock.clone();
        let cloned_receiver = receiver_for_main.clone();
        move |_lua: &Lua, _args: ()| -> LuaResult<i32> {
            cloned_sender.send(clock::ClockControlMessage {
                name: "peers".to_string(),
                args: vec![],
            }).unwrap();
            let recv = cloned_receiver.lock().unwrap().recv().unwrap();
            match recv.name.as_str() {
                "peers" => {
                    Ok(recv.args[0].parse::<i32>().unwrap())
                },
                _ => {
                    println!("Unknown command: {}", recv.name);
                    Ok(0)
                }
            }
        }
    });
    let _ = interpreter.run();
    println!("{}", ascii::GOODBYE);
    Ok(())
}