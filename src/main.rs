// #![allow(non_snake_case)]
#![allow(unused_variables)]
// extern crate rodio;
extern crate sciter;
//


use std::fs::File;
// use std::{thread, time};
use sciter::Element;
use self::sciter::dom::event::*;
use self::sciter::dom::HELEMENT;
use std::io::BufReader;
use rodio::Sink;
use walkdir::WalkDir;
use sublime_fuzzy::{best_match, format_simple};

struct Song {
    path: String,
}

struct Player {
    device: rodio::Device,
    sink: Sink,
    current_song_index: usize,
    library: Vec<Song>,
    ui_ready: bool,
    top_match_index: usize
}

impl Player {
    pub fn new() -> Self {
        let device = rodio::default_output_device().unwrap();
        let sink = Sink::new(&device);
        Player {
            sink,
            device,
            current_song_index: 0,
            library: Vec::new(),
            ui_ready: false,
            top_match_index: 0
        }
    }

    pub fn load_top_match(&mut self) {
        // Ok song probably needs a copy constructor?
        self.load_song(&Song { path: self.library[self.top_match_index].path.to_string() });
        // Can i make something like this work??
        // self.load_song(self.library[self.top_match_index]);
    }

    // Sink::stop() seems to stop the sink forever for some reason.
    // So to stop a playing sound we have to destroy the sink and
    // make a new one. (?)
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
                        // self.current_song = Some(song);
                        self.reset_sink();
                        self.sink.append(source);
                        return true;
                    },
                    Err(_) => return false,
                },
            Err(_) => return false,
        }
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn load_folder(&mut self, dir : &str) {
        for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            // println!("{}", entry.path().display());
            let path = entry.path().to_str();
            match path {
                Some(p) => {
                    let song = Song { path: p.to_string() };
                    self.library.push(song);
                },
                None => continue,
            }
        }
    }
}


fn main() {
    let mut player = Player::new();
    player.load_folder("test_songs");
    let html = include_bytes!("minimal.html");
    let mut frame = sciter::Window::new();
    frame.event_handler(player);
    frame.load_html(html, Some("example://minimal.html"));
    frame.run_app();
    println!("hi");
}

impl sciter::EventHandler for Player{

    fn on_event(&mut self, root: HELEMENT, source: HELEMENT,
                target: HELEMENT, code: BEHAVIOR_EVENTS,
                phase: PHASE_MASK, reason: EventReason) -> bool {
        if phase != PHASE_MASK::SINKING { return false; }
        if code == BEHAVIOR_EVENTS::DOCUMENT_READY {
            self.ui_ready = true;
        }
        if !self.ui_ready {
            return false;
        }

        let root = Element::from(root).root();
        let source = Element::from(source);
        let mut last_event_element = root.find_first("#last-event").unwrap()
                        .expect("div#last-event not found!");
        let id = source.get_attribute("id").unwrap_or_else(|| "_".to_string());
        if id != "last-event" {
            let result = last_event_element.set_text(format!("\nEVENT: {:?} ({})", code, source).as_str());
        }
        match code {
            BEHAVIOR_EVENTS::BUTTON_CLICK => return on_click(self, root, source),
            BEHAVIOR_EVENTS::EDIT_VALUE_CHANGED => return on_input(self, root, source),
            _ => (),
        }

        false
    }
}

fn on_input(player: &mut Player, root: Element, source: Element) -> bool {
    if let Some(id) = source.get_attribute("id") {
        match id.as_str() {
            "user-input" => {
                let mut matches_el = root.find_first("#matches").unwrap()
                    .expect("div#matches element not found");
                let user_input = root.find_first("#user-input").unwrap()
                    .expect("div#user-input not found!");
                // for some reason this will take a br raw string
                // but not " " (?) TODO find out why.
                let result = matches_el.set_html(br#" "#, None).unwrap();
                let mut i = 0;
                let mut top_score = 0;
                let mut top_index = 0;
                for song in &player.library {
                    let search_box = best_match(user_input.get_text().as_str(), song.path.as_str());
                    match search_box {
                        Some(search) => {
                        if search.score() >= top_score {
                            top_score = search.score();
                            top_index = i;
                        }
                        let formatted_match = format_simple(&search, &song.path, "<span style=\"color:red\">", "</span>");
                        let mut el = Element::create("li").unwrap();
                        let result = matches_el.append(&el);
                        let result = el.set_html(formatted_match.as_str().as_bytes(), None);
                        let result = el.append(&Element::with_text("span", format!(" [score: {}]", search.score()).as_str()).unwrap());
                        let result = el.append(&Element::with_text("span", format!(" [index: {}]", i).as_str()).unwrap());
                        let result = matches_el.append(&Element::create("br").unwrap());
                        }
                        None => {
                            let mut el = Element::create("li").unwrap();
                            let result = el.set_text(song.path.as_str());
                        },
                    }
                    i += 1;
                }
                player.top_match_index = top_index;
                println!("Top Match Score: {}", top_score);
                println!("Top Match Index: {}", player.top_match_index);
                return true;
            }
            _ => return false
        }
    }
    false
}

fn on_click(player: &mut Player, root: Element, source: Element) -> bool {
    // let root = Element::from(root).root();
    let user_input = root.find_first("#user-input").unwrap()
                    .expect("div#user-input not found!");

    if let Some(id) = source.get_attribute("id") {
        match id.as_str() {
            "load-button" => {
                player.load_top_match();
                return true;
            },
            "play-button" => {
                player.play();
                return true;
            },
            "pause-button" => {
                player.pause();
                return true;
            }
            _ => return false,
        }
    }
    false
}