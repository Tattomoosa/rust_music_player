// #![allow(non_snake_case)]
#![allow(unused_variables)]
// extern crate rodio;
extern crate sciter;
//


use std::fs::File;
use sciter::Element;
use self::sciter::dom::event::*;
use self::sciter::dom::HELEMENT;
use std::io::BufReader;
use rodio::Sink;
use walkdir::WalkDir;
use sublime_fuzzy::{best_match, format_simple};
// use inputbot::{*};
use inputbot;

struct Song {
    path: String,
}

struct Player {
    device: rodio::Device,
    sink: Sink,
    current_song_index: usize,
    library: Vec<Song>,
    is_ui_open: bool,
    is_ui_ready: bool,
    top_match_index: usize
}

struct UIMatch {
    match_type: UIMatchType,
    score: isize,
    text: String,
    index: usize
}

enum UIMatchType {
    SONG,
    OPERATION
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
            is_ui_open: false,
            is_ui_ready: false,
            top_match_index: 0
        }
    }

    pub fn get_matches(&self, query: &str) -> Vec<UIMatch> {
        let mut matches = Vec::new();
        let open_tag = "<span style=\"color:red\">";
        let close_tag = "</span>";
        let mut i = 0;
        for song in &self.library {
            let result = best_match(query, song.path.as_str());
            match result {
                Some(result) => matches.push(UIMatch {
                        match_type: UIMatchType::SONG,
                        score: result.score(),
                        index: i,
                        text: format!("[TYPE: SONG, SCORE: {}, INDEX: {}] {}",
                                      result.score(),
                                      i,
                                      format_simple(&result, &song.path, open_tag, close_tag)).to_string(),
                    }),
                None => ()
            }
            i += 1;
        }
        matches
    }

    pub fn load_top_match(&mut self) {
        self.load_song_from_library(self.top_match_index);
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
        return self.load_song_from_path(&song.path);
    }

    pub fn load_song_from_library(&mut self, index: usize) -> bool {
        // let pathstring = &self.library[index].path;
        // return self.load_song_from_path(pathstring.as_str());
        let pathstring = &self.library[index].path.clone();
        return self.load_song_from_path(pathstring.as_str());
    }

    pub fn load_song_from_path(&mut self, song_path: &str) -> bool {
        return match File::open(&song_path) {
            Ok(file) =>
                match rodio::Decoder::new(BufReader::new(file)) {
                    Ok(source) => {
                        // self.current_song = Some(song);
                        self.reset_sink();
                        self.sink.append(source);
                        true
                    },
                    Err(_) => false,
                },
            Err(_) => false,
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
            let path = entry.path().to_str();
            match path {
                Some(p) => {
                    if p.ends_with(".mp3") {
                        // let song = Song { path: p.to_string() };
                        self.library.push(Song {
                            path: p.to_string()
                        });
                    }
                },
                None => continue,
            }
        }
    }
}


fn main() {
    inputbot::KeybdKey::MKey.bind(|| {
        let mut player = Player::new();
        player.load_folder("test_songs");
        let html = include_bytes!("minimal.html");
        println!("yooooooo");
        let mut frame = sciter::Window::new();
        frame.event_handler(player);
        frame.load_html(html, Some("example://minimal.html"));
        frame.run_app();
    });
    inputbot::handle_input_events();
    println!("hi");
}

impl sciter::EventHandler for Player{

    fn on_event(&mut self, root: HELEMENT, source: HELEMENT,
                target: HELEMENT, code: BEHAVIOR_EVENTS,
                phase: PHASE_MASK, reason: EventReason) -> bool {
        if phase != PHASE_MASK::SINKING { return false; }

        if code == BEHAVIOR_EVENTS::DOCUMENT_READY {
            self.is_ui_ready = true;
        } else if !self.is_ui_ready {
            return false;
        }

        let root = Element::from(root).root();
        let source = Element::from(source);
        let mut last_event_element = root.find_first("#last-event").unwrap()
                        .expect("div#last-event not found!");
        let id = source.get_attribute("id").unwrap_or_else(|| "_".to_string());

        if id != "last-event" && id != "debug" {
            let result = last_event_element.set_text(format!("\nEVENT: {:?} ({})", code, source).as_str());
            println!("\nEVENT: {:?} ({})", code, source);
        }

        match code {
            BEHAVIOR_EVENTS::DOCUMENT_READY => return update_matches(self, root, source),
            BEHAVIOR_EVENTS::BUTTON_CLICK => return on_click(self, root, source),
            BEHAVIOR_EVENTS::EDIT_VALUE_CHANGED => return on_input(self, root, source),
            _ => false
        }
    }
}

fn update_matches(player: &mut Player, root: Element, source: Element) -> bool {
    let mut matches_el = root.find_first("#matches").unwrap()
        .expect("div#matches element not found");
    let user_input = root.find_first("#user-input").unwrap()
        .expect("div#user-input not found!");
    // for some reason this will take a br raw string
    // but not " " (?) TODO find out why.
    let result = matches_el.set_html(br#" "#, None).unwrap();
    let mut ui_matches = player.get_matches(user_input.get_text().as_str());
    ui_matches.sort_by(|b,a| a.score.cmp(&b.score));
    let mut top_score = 0;
    let mut top_index = 0;
    for m in ui_matches {
        let mut el = Element::create("li").unwrap();
        let result = matches_el.append(&el);
        let result = el.set_html(m.text.as_str().as_bytes(), None);
        if m.score >= top_score {
            top_score = m.score;
            top_index = m.index;
        }
    }
    player.top_match_index = top_index;
    true
    /*
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
            let mut formatted_match = String::from(format!(" [score: {}]", search.score()).as_str());
            formatted_match.push_str(format!(" [index: {}]", i).as_str());
            formatted_match.push_str(format_simple(&search, &song.path, "<span style=\"color:red\">", "</span>").as_str());
            let mut el = Element::create("li").unwrap();
            let result = matches_el.append(&el);
            let result = el.set_html(formatted_match.as_str().as_bytes(), None);
            }
            None => {
                let mut el = Element::create("li").unwrap();
                let result = matches_el.append(&el);
                let result = el.set_html(format!("<span style=\"color: lightgrey\">{}</span>", song.path).as_bytes(), None);
            },
        }
        i += 1;
    }
    player.top_match_index = top_index;
    println!("Top Match Score: {}", top_score);
    println!("Top Match Index: {}", player.top_match_index);
    return true;
    */
}

fn on_input(player: &mut Player, root: Element, source: Element) -> bool {
    if let Some(id) = source.get_attribute("id") {
        return match id.as_str() {
            "user-input" => update_matches(player, root, source),
            _ => false
        };
    }
    return false;
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
