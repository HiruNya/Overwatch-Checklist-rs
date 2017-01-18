#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate yaml_rust;
extern crate toml;

//for main loop
use std::collections::{HashMap, BTreeMap};
use conrod::backend::piston::{OpenGL, window, WindowEvents};
use conrod::backend::piston::event::UpdateEvent;
use conrod::{UiBuilder};
use std::path::PathBuf;

//for widgets
use conrod::widget::{Widget, Canvas, List, Button, Matrix, RoundedRectangle, Text};
use conrod::{Positionable, Colorable, Sizeable, Labelable};
use conrod::color::{Color as Colour, DARK_YELLOW, WHITE, DARK_GREY, YELLOW, PURPLE};
const CYAN: Colour = Colour::Rgba(0 as f32, 255 as f32, 255 as f32, 1.0);
const DARK_CYAN: Colour = Colour::Rgba(0 as f32, 125 as f32, 255 as f32, 0.1);
const MAGENTA: Colour = Colour::Rgba(255 as f32, 0 as f32, 255 as f32, 1.0);
const DARK_MAGENTA: Colour = Colour::Rgba(255 as f32, 0 as f32, 255 as f32, 0.1);

widget_ids! {
    struct Ids {
        canvas,
        list,
        tab_buttons,
        button_matrix,
        file_nav,
        x_button,
        error_box,
        error_text,
        error_button,
    }
}

struct AppData {
    data: HashMap<String, Vec<CosmeticItem>>,
    current_tab: String,
    view_type: u8,
    event_file: PathBuf,
    file_list: Vec<PathBuf>,
    data_changed: bool,
    errors: Vec<String>,
}

impl AppData {
    fn new() -> AppData {
        use std::fs::File;
        use std::io::Read;
        use yaml_rust::YamlLoader;
        let mut path = match find_folder::Search::KidsThenParents(3, 5)
            .for_folder("events") {
            Ok(e) => e,
            Err(e) => panic!("Error: {}", e),
        };
        let event_dir = path.clone();

        let config = ConfigData::load_toml(event_dir.clone());

        let mut hash_data: HashMap<String, Vec<CosmeticItem>> = HashMap::new();
        let mut event_file_path = PathBuf::new();

        if config.default_path == PathBuf::new() {
            let cos_types = vec!(
                "Skins",
                "Emotes",
                "Sprays",
                "Voice Lines",
                "Victory Poses",
                "Player Icons",
                "Highlight Intros",
            );
            for i in cos_types {
                hash_data.insert(i.to_string(), Vec::new());
            }
        }
        else {
            path.push(config.default_path.as_path());
            event_file_path = path.clone();
            let mut file = match File::open(path) {
                Ok(e) => e,
                Err(e) => panic!("Error: {}", e),
            };
            let mut document = String::new();
            match file.read_to_string(&mut document) {
                Ok(_) => {},
                _ => panic!("An error occurred opening an event file"),
            }
            let yaml_data = match YamlLoader::load_from_str(document.as_str()) {
                Ok(e) => e,
                Err(e) => panic!("Error reading the YAML file: {}", e),
            };

            let cosmetic_types = match yaml_data[0].as_hash() {
                Some(e) => e.clone(),
                None => panic!("Error in unwrapping hash value from yaml data"),
            };
            drop(yaml_data);
            for (i, j) in cosmetic_types {
                let mut vec_data = Vec::new();
                let def_vec = Vec::new();
                for k in j.as_vec().unwrap_or(&def_vec) {
                    let bool_val = match k["Obtained"].as_i64().unwrap_or(0) {
                        1 => true,
                        _ => false,
                    };
                    let item = CosmeticItem {
                        name: k["Name"].as_str().unwrap_or("ERROR").to_string(),
                        rarity: k["Rarity"].as_i64().unwrap_or(0).clone() as u8,
                        obtained: bool_val,
                    };
                    vec_data.push(item)
                }
                hash_data.insert(i.as_str().unwrap_or("ERROR").to_string(), vec_data);
            }
        }
        AppData {
            data: hash_data,
            current_tab: "Skins".to_string(),
            view_type: 0,
            file_list: generate_file_list(event_dir).unwrap_or_else(|| Vec::new()),
            event_file: event_file_path,
            data_changed: false,
            errors: Vec::new(),
        }
    }
    fn refresh_data(&mut self) {
        use std::fs::File;
        use std::io::Read;
        use yaml_rust::YamlLoader;
        let mut file = match File::open(&self.event_file) {
            Ok(e) => e,
            Err(e) => {self.send_error(format!("Error: {}", e)); return {}},
        };
        let mut document = String::new();
        match file.read_to_string(&mut document) {
            Ok(_) => {},
            _ => {self.send_error(format!("An error occurred opening an event file")); return {}},
        }
        let yaml_data = match YamlLoader::load_from_str(document.as_str()) {
            Ok(e) => e,
            Err(e) => {self.send_error(format!("Error reading the YAML file: {}", e)); return {}},
        };

        let mut hash_data: HashMap<String, Vec<CosmeticItem>> = HashMap::new();
        let cosmetic_types = match yaml_data[0].as_hash() {
            Some(e) => e.clone(),
            None => {self.send_error(format!("Error in unwrapping hash value from yaml data")); return {}},
        };
        drop(yaml_data);
        for (i, j) in cosmetic_types {
            let mut vec_data = Vec::new();
            let def_vec = Vec::new();
            for k in j.as_vec().unwrap_or(&def_vec) {
                let bool_val = match k["Obtained"].as_i64().unwrap_or(0) {
                    1 => true,
                    _ => false,
                };
                let item = CosmeticItem {
                    name: k["Name"].as_str().unwrap_or("ERROR").to_string(),
                    rarity: k["Rarity"].as_i64().unwrap_or(0).clone() as u8,
                    obtained: bool_val,
                };
                vec_data.push(item)
            }
            hash_data.insert(i.as_str().unwrap_or("ERROR").to_string(), vec_data);
        }
        self.data = hash_data;
    }
    fn save_data_to_file(&mut self) {
        use std::fs::File;
        use std::io::Write;
        use yaml_rust::emitter;
        use yaml_rust::Yaml;

        let mut hash_data = BTreeMap::new();
        for (key, value) in self.data.iter() {
            let mut vec = Vec::new();
            for item in value {
                let name = Yaml::String(item.name.clone());
                let rarity = Yaml::Integer(item.rarity as i64);
                let obtained = Yaml::Integer(if item.obtained {1} else {0});
                let mut hash = BTreeMap::new();
                hash.insert(Yaml::String("Name".to_string()), name);
                hash.insert(Yaml::String("Rarity".to_string()), rarity);
                hash.insert(Yaml::String("Obtained".to_string()), obtained);
                vec.push(Yaml::Hash(hash));
            }
            hash_data.insert(Yaml::String(key.to_string()), Yaml::Array(vec));
        }
        let yaml_data = Yaml::Hash(hash_data);

        let mut file_string = String::new();
        {
            let mut emit = emitter::YamlEmitter::new(&mut file_string);
            match emit.dump(&yaml_data) {
                Ok(_) => {},
                Err(e) => {self.send_error(format!("Error Saving: {:?}", e)); return {}},
            };
        }
        let mut file = match File::create(&self.event_file) {
            Ok(e) => e,
            Err(e) => {self.send_error(format!("Error Saving: {:?}", e)); return {}},
        };
        match file.write_all(file_string.as_bytes()) {
            Ok(_) => {},
            Err(e) => {self.send_error(format!("Error Saving: {:?}", e)); return {}},
        };
    }
    fn reset_obtained_data(&mut self) {
        for (_, value) in self.data.iter_mut() {
            for item in value {
                item.obtained = false
            }
        }
    }
    fn send_error(&mut self, text: String) {
        self.errors.insert(0, text)
    }
}

fn set_ui(ref mut ui: conrod::UiCell, ids: &mut Ids, data: &mut AppData) {
    //Add the canvas
    Canvas::new().set(ids.canvas, ui);

    // Add Cosmetic Tabs in the form of buttons
    {
        let cos_types = vec!(
        "Skins",
        "Emotes",
        "Sprays",
        "Voice Lines",
        "Victory Poses",
        "Player Icons",
        "Highlight Intros",
        );
        let mut cosmetic_tab_matrix = Matrix::new(cos_types.len(), 1)
            .top_left_of(ids.canvas)
            .w_h(ui.win_w, (ui.win_h/8.0))
            .set(ids.tab_buttons, ui);
        for i in cos_types {
            for _ in cosmetic_tab_matrix.next(ui).unwrap().set(
                Button::new()
                    .color(if i == data.current_tab {DARK_GREY} else {WHITE})
                    .label(i),
                    ui
            ) {
                data.current_tab = i.to_string();
                if data.view_type != 0 {
                    data.view_type = 0;
                }
            }
        };
    }

    // Make the list
    match data.view_type {
        1 => {
            let file_list = data.file_list.clone();
            let (mut list, scroll) = List::new(file_list.len(), 30.0)
                .middle_of(ids.canvas)
                .w_h(ui.win_w, (6.0*ui.win_h)/8.0)
                .set(ids.file_nav, ui);
            match scroll {
                Some(e) => e.set(ui),
                None => {},
            }
            for file_item in file_list {
                match list.next(ui) {
                    Some(element_) => {
                        let str_ = file_item.file_name().unwrap().to_str().unwrap();
                        for _ in element_.set(
                            Button::new()
                                .label(str_)
                                .color(if file_item == data.event_file {DARK_MAGENTA} else {WHITE}),
                            ui
                        ) {
                            if file_item != data.event_file {
                                data.event_file = file_item.clone();
                                data.refresh_data();
                            }
                        };
                    },
                    None => break,
                }
            }
            for _ in Button::new()
                .w_h(30.0, 30.0)
                .color(Colour::Rgba(255.0, 0.0, 0.0, 1.0))
                .top_right_of(ids.file_nav)
                .set(ids.x_button, ui) {
                data.view_type = 0;
            }

        },
        _ => {
            let mut list = data.data.get_mut(data.current_tab.as_str()).unwrap();
            let (mut items, scrollbar) = List::new(list.len(), 30.0)
                .w_h(ui.win_w, ((6.0 * ui.win_h) / 8.0))
                .middle_of(ids.canvas)
                .scrollbar_on_top()
                .scrollbar_color(Colour::Rgba(255.0, 0.0, 0.0, 255.0))
                .scrollbar_width(20.0)
                .instantiate_all_items()
                .set(ids.list, ui);
            match scrollbar {
                // Unwrap the scrollbar
                Some(e) => e.set(ui),
                None => {},
            }
            // Create an element for each item in the list
            for ref mut item_in_list in list.iter_mut() {
                let colour = match item_in_list.obtained {
                    false => match item_in_list.rarity {
                        1 => DARK_CYAN,
                        2 => DARK_MAGENTA,
                        3 => DARK_YELLOW,
                        _ => DARK_GREY,
                    },
                    true => match item_in_list.rarity {
                        1 => CYAN,
                        2 => MAGENTA,
                        3 => YELLOW,
                        _ => WHITE,
                    }
                };
                let item = Button::new()
                    .color(colour)
                    .label(item_in_list.name.as_str());
                match items.next(ui) {
                    Some(e) => for _ in e.set(item, ui) {
                        item_in_list.obtained = match item_in_list.obtained {
                            false => true,
                            true => false,
                        };
                        if !data.data_changed {
                            data.data_changed = true
                        }
                    },
                    None => {},
                };
            }
        },
    }

    {
        let num_errors = data.errors.len();
        if num_errors != 0 {
            RoundedRectangle::fill_with([ui.win_w / 2.0, 5.0 * ui.win_h / 8.0], 5.0, PURPLE)
                .middle_of(ids.canvas)
                .set(ids.error_box, ui);
            Text::new(data.errors[num_errors - 1].as_str())
                .padded_wh_of(ids.error_box, 40.0)
                .middle_of(ids.error_box)
                .align_text_middle()
                .set(ids.error_text, ui);
            for _ in Button::new()
                .label("Ok")
                .w_h(200.0, 30.0)
                .mid_bottom_with_margin_on(ids.error_box, 40.0)
                .set(ids.error_button, ui) {
                data.errors.pop();
            };
        }
    }

    // Buttons
    {
        let mut button_matrix = Matrix::new(3, 1)
            .w_h(ui.win_w, ui.win_h / 8.0)
            .bottom_left_of(ids.canvas)
            .set(ids.button_matrix, ui);
        for _ in button_matrix.next(ui).unwrap()
            .set(Button::new()
                     .label("Load Data")
                     .color(WHITE),
                 ui) {
            data.view_type = 1;
        }
        for _ in button_matrix.next(ui).unwrap()
            .set(Button::new()
                     .label("Save Data")
                     .color(if data.data_changed {WHITE} else {DARK_GREY}),
                 ui) {
            if data.data_changed {
                data.save_data_to_file();
                data.data_changed = false;
            }
        };
        for _ in button_matrix.next(ui).unwrap()
            .set(Button::new()
                     .label("Reset Data")
                     .color(WHITE),
                 ui) {
            data.reset_obtained_data();
        }
    }
}

fn main() {
    const WIDTH: u32 = 900;
    const HEIGHT: u32 = 400;
    let opengl = OpenGL::V3_2;
    let mut window: window::Window = window::WindowSettings::new("Overwatch Checklist", [WIDTH, HEIGHT])
        .opengl(opengl).exit_on_esc(true).vsync(true).build().expect("Making the window failed");
    let mut events: WindowEvents = WindowEvents::new();
    let mut ui = UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets").expect("Loading the assets failed");
    let font_path = assets.join("koverwatch.ttf");
    ui.fonts.insert_from_file(font_path).expect("Loading the font failed");
    let mut text_texture_cache = window::GlyphCache::new(&mut window, WIDTH, HEIGHT);
    let image_map = conrod::image::Map::new();
    let ids = &mut Ids::new(ui.widget_id_generator());

    let mut data = AppData::new();

    while let Some(event) = window.next_event(&mut events) {
        if let Some(e) = window::convert_event(event.clone(), &window) {
            ui.handle_event(e)
        }
        event.update(|_|
            set_ui(ui.set_widgets(), ids, &mut data)
        );
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                window::draw(c, g, primitives,
                &mut text_texture_cache,
                &image_map,
                texture_from_image);
            }
        });
    }
}

fn generate_file_list(dir: PathBuf) -> Option<Vec<PathBuf>>{
    let mut dir_contents = match dir.read_dir() {
                Ok(e) => e,
                Err(_) => return None,
            };
    let mut file_list = Vec::new();
    loop {
        match dir_contents.next() {
            Some(dir_entry) => match dir_entry {
                Ok(entry_) => file_list.push(entry_.path()),
                Err(_) => return None,
            },
            None => break,
        }
    }
    Some(file_list)
}

/*fn string_to_colour(mut source: String) -> Option<Colour>{
    if source.len() != 6 {
        return None;
    }
    let mut colour_vals = vec!(0 as f32, 0 as f32, 0 as f32);
    for i in 0..3 {
        let unit = match source.pop() {
            Some(char) => match char {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                'A' | 'a' => 10,
                'B' | 'b' => 11,
                'C' | 'c' => 12,
                'D' | 'd' => 13,
                'E' | 'e' => 14,
                'F' | 'f' => 15,
                _ => return None,
            },
            None => return None,
        };
        let tens = match source.pop() {
            Some(char) => match char {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                'A' | 'a' => 10,
                'B' | 'b' => 11,
                'C' | 'c' => 12,
                'D' | 'd' => 13,
                'E' | 'e' => 14,
                'F' | 'f' => 15,
                _ => return None,
            },
            None => return None,
        };
        colour_vals[i] = ((16 * tens) + unit) as f32;
    }
    Some(Colour::Rgba(colour_vals[2], colour_vals[1], colour_vals[0], 1.0))
}*/

/*fn colour_to_string(source: &Colour) -> String {
    let colours = vec!(source.red() as u8, source.green() as u8, source.blue() as u8);
    let mut hex_colours = String::new();
    for i in colours {
        hex_colours.push_str(format!("{:X}", i).as_str())
    }
    hex_colours
}*/

struct ConfigData {
    default_path: PathBuf,
}

impl ConfigData {
    fn load_toml(event_dir: PathBuf) -> ConfigData {
        use std::env::current_dir;
        use std::fs::File;
        use std::io::Read;
        use toml::Parser;
        use toml::Value;

        let mut config_path = match current_dir() {
            Ok(e) => e,
            Err(e) => panic!("Error getting current directory: {}", e),
        };
        config_path.push("config.toml");
        if !config_path.as_path().is_file() {
            panic!("Config file is missing")
        };

        let mut file = match File::open(config_path) {
            Ok(e) => e,
            Err(e) => panic!("Error opening config file: {:?}", e),
        };
        let mut toml_data = String::new();
        match file.read_to_string(&mut toml_data) {
            Ok(e) => e,
            Err(e) => panic!("Error reading the config file: {}", e),
        };

        let data = match Parser::new(toml_data.as_str()).parse() {
            Some(e) => e,
            None => panic!("Error parsing config file!"),
        };

        let general = match data["General"].clone() {
            Value::Table(table) => table,
            _ => panic!("Error in using \"General\" table"),
        };

        let mut default_path = PathBuf::from(match general["default_event"] {
            Value::String(ref e) => event_dir.join(PathBuf::from(e.clone())),
            _ => panic!("Could not decode \"default_event\""),
        });
        if !default_path.is_file() {
            default_path = PathBuf::new();
        }

        ConfigData {
            default_path: default_path,
        }
    }
}

struct CosmeticItem {
    name: String,
    rarity: u8,
    obtained: bool,
}