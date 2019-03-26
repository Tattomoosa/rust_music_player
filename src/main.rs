// #![allow(non_snake_case)]
#![allow(unused_variables)]
// extern crate rodio;
extern crate sciter;
//


use std::fs::File;
use std::{thread, time};
use sciter::{Element, Value};
use self::sciter::dom::event::*;
use self::sciter::dom::HELEMENT;
use std::io::BufReader;
use rodio::Sink;
//
struct PlayButton;

struct Song {
    path: String,
}

struct Player {
    device: rodio::Device,
    sink: Sink,
    current_song: Option<Song>,
    library: Option<Box<Song>>
}

impl Player {
    pub fn new() -> Self {
        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);
        Player {
            sink,
            device,
            current_song: None,
            library: None
        }
    }

    // Sink::stop() seems to stop the sink forever for some reason.
    // So to stop a playing sound we have to destroy the sink and
    // make a new one.
    fn reset_sink(&mut self) {
        if !self.sink.empty() {
            self.sink = Sink::new(&self.device);
        }
    }

    // TODO return proper error
    pub fn load_song(&mut self, song: &Song) -> bool {
        match File::open(&song.path) {
            Ok(file) =>
                match rodio::Decoder::new(BufReader::new(file)) {
                    Ok(source) => {
                        self.reset_sink();
                        self.sink.append(source);
                        return true;
                    },
                    Err(_) => return false,
                },
            Err(_) => return false,
        }
        /*
        let file = File::open(&song.path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        self.reset_sink();
        self.sink.append(source);
        true
        */
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn play(&self) {
        self.sink.play();
    }
}


fn main() {
    let mut player = Player::new();
    // let song = Song { path: "test.mp3".to_string() };
    // player.load_song(&song);
    // player.play();
    // thread::sleep(time::Duration::from_millis(3000));
    // player.pause();
    // thread::sleep(time::Duration::from_millis(1000));
    // player.load_song(&song);
    // player.play();
    // thread::sleep(time::Duration::from_millis(3000));
    let html = include_bytes!("minimal.html");
    // let mut frame = sciter::Window::new();
    let mut frame = sciter::Window::new();
    frame.event_handler(player);
    frame.load_html(html, Some("example://minimal.html"));
    frame.run_app();
    println!("hi");
}

impl sciter::EventHandler for Player {
    fn on_event(&mut self, root: HELEMENT, source: HELEMENT,
                target: HELEMENT, code: BEHAVIOR_EVENTS,
                phase: PHASE_MASK, reason: EventReason) -> bool {

        if phase != PHASE_MASK::SINKING { return false; }

        let root = Element::from(root).root();
        let source = Element::from(source);

        println!("\nEVENT: {:?} ({})", code, source);

        if code == BEHAVIOR_EVENTS::BUTTON_CLICK {
            // let root = Element::from(root).root();
            let last_event_element = root.find_first("#last-event").unwrap()
                          .expect("div#last-event not found!");
            let song_path = root.find_first("#user-input").unwrap()
                          .expect("div#user-input not found!");


            if let Some(id) = source.get_attribute("id") {
                match id.as_str() {
                    "load-button" => {
                        self.load_song(&Song { path: song_path.get_text() });
                        return true;
                    },
                    "play-button" => {
                        self.play();
                        return true;
                    },
                    "pause-button" => {
                        self.pause();
                        return true;
                    }
                    _ => panic!("ID '{}' not recognized!", id),
                }
            };
        };

        false
    }
}

/*
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
*/
