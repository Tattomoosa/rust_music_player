#![allow(non_snake_case)]
#![allow(unused_variables)]
// extern crate rodio;
extern crate sciter;
//


use std::fs::File;
use std::{thread, time};
use sciter::{Element, Value};
use self::sciter::dom::event::*;
use self::sciter::dom::HELEMENT;
// use self::sciter::value::Value;
// use std::io::BufReader;
// use rodio::Sink;
//
struct PlayButton;

fn main() {
    /*
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    let file = File::open("test.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);
    // rodio::play_raw(&device, source.convert_samples());
    loop {
        thread::sleep(time::Duration::from_millis(1000));
        println!("PAUSE!");
        Sink::pause(&sink);
        thread::sleep(time::Duration::from_millis(1000));
        println!("PLAY!");
        Sink::play(&sink);
    }
    */
    let html = include_bytes!("minimal.html");
    let mut frame = sciter::Window::new();
    frame.event_handler(PlayButton);
    frame.load_html(html, Some("example://minimal.html"));
    frame.run_app();
    println!("hi");
}

impl sciter::EventHandler for PlayButton {
    fn on_event(&mut self, root: HELEMENT, source: HELEMENT,
                target: HELEMENT, code: BEHAVIOR_EVENTS,
                phase: PHASE_MASK, reason: EventReason) -> bool {

        if code == BEHAVIOR_EVENTS::BUTTON_CLICK {
            let root = Element::from(root).root();
            let message = root.find_first("#song-title").unwrap()
                          .expect("div#song-title not found!");
            let source = Element::from(source);

            println!("root: {:?}, message: {:?}, source: {:?}",
                     root, message, source);

            if let Some(id) = source.get_attribute("id") {
                if id == "send" {
                    source.send_event(BEHAVIOR_EVENTS::CHANGE, None,
                                      Some(message.as_ptr()))
                                      .expect("Failed to send event");
                    return true;
                } else if id == "fire" {
                    let data = Value::from("Rusty param");
                    source.fire_event(BEHAVIOR_EVENTS::CHANGE, None,
                                      Some(message.as_ptr()), false,
                                      Some(data)).expect("Failed to 
                                      fire event");
                    return true;
                }
            };
        };

        false
    }
}
