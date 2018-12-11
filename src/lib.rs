#[macro_use]
extern crate serde_derive;
extern crate directories;
extern crate regex;
extern crate reqwest;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::{create_dir, read_dir, File};
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT_SET: u8 = 2;

#[derive(Serialize, Deserialize, Debug)]
struct ExpirationWrapper {
    expire_time: u64,
    card_set_json: CardSetJson,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CardSetJson {
    pub card_set: CardSet,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CardSet {
    pub version: u32,
    pub set_info: SetInfo,
    pub card_list: Vec<Card>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SetInfo {
    pub set_id: u32,
    pub pack_item_def: u32,
    pub name: TranslatedText,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslatedText {
    #[serde(default)]
    pub english: String,
    #[serde(default)]
    pub german: String,
    #[serde(default)]
    pub french: String,
    #[serde(default)]
    pub italian: String,
    #[serde(default)]
    pub koreana: String,
    #[serde(default)]
    pub spanish: String,
    #[serde(default)]
    pub schinese: String,
    #[serde(default)]
    pub tchinese: String,
    #[serde(default)]
    pub russian: String,
    #[serde(default)]
    pub thai: String,
    #[serde(default)]
    pub japanese: String,
    #[serde(default)]
    pub portuguese: String,
    #[serde(default)]
    pub polish: String,
    #[serde(default)]
    pub danish: String,
    #[serde(default)]
    pub dutch: String,
    #[serde(default)]
    pub finnish: String,
    #[serde(default)]
    pub norwegian: String,
    #[serde(default)]
    pub swedish: String,
    #[serde(default)]
    pub hungarian: String,
    #[serde(default)]
    pub czech: String,
    #[serde(default)]
    pub romanian: String,
    #[serde(default)]
    pub turkish: String,
    #[serde(default)]
    pub brazilian: String,
    #[serde(default)]
    pub bulgarian: String,
    #[serde(default)]
    pub greek: String,
    #[serde(default)]
    pub ukrainian: String,
    #[serde(default)]
    pub latam: String,
    #[serde(default)]
    pub vietnamese: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Card {
    pub card_id: u32,
    pub base_card_id: u32,
    pub card_type: String,
    #[serde(default)]
    pub sub_type: String,
    pub card_name: TranslatedText,
    pub card_text: TranslatedText,
    pub mini_image: Image,
    pub large_image: Image,
    pub ingame_image: Image,
    #[serde(default)]
    pub illustrator: String,
    #[serde(default)]
    pub is_red: bool,
    #[serde(default)]
    pub is_green: bool,
    #[serde(default)]
    pub is_blue: bool,
    #[serde(default)]
    pub is_black: bool,
    #[serde(default)]
    pub gold_cost: u32,
    #[serde(default)]
    pub mana_cost: u32,
    #[serde(default)]
    pub attack: u32,
    #[serde(default)]
    pub hit_points: u32,
    pub references: Vec<Reference>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct HeroCard {
    pub card: Card,
    pub turn: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CardCard {
    pub card: Card,
    pub count: u32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Deck {
    pub name: String,
    pub heroes: Vec<HeroCard>,
    pub cards: Vec<CardCard>,
}

impl Deck {
    pub fn new() -> Self {
        Self {
            name: String::from(""),
            heroes: Vec::new(),
            cards: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    #[serde(default)]
    pub default: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reference {
    #[serde(default)]
    pub card_id: u32,
    #[serde(default)]
    pub ref_type: String,
    #[serde(default)]
    pub count: u32,
}

#[derive(Serialize, Deserialize)]
struct JsonRef {
    cdn_root: String,
    url: String,
    expire_time: u64,
}

/// This function will search the user's local cache for
/// the card set data, if not found or out of date, will
/// fetch updates from Valve's API and update the cached files.
/// Once that process is complete, it will return a Vec of [CardSets](struct.CardSet.html).
pub fn get_all_card_sets() -> Result<Vec<CardSet>, String> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let proj_dir = directories::ProjectDirs::from("", "", "artifact_lib").unwrap();
    let cache_dir = proj_dir.cache_dir();
    let dir = match read_dir(cache_dir) {
        Ok(d) => d,
        Err(_) => match create_dir(cache_dir) {
            Ok(_) => read_dir(cache_dir).unwrap(),
            Err(e) => {
                return Err(format!(
                    "Error reading or creating directory: {:?}, {}",
                    cache_dir, e
                ))
            }
        },
    };

    let mut fetch_sets = Vec::new();
    // assume all are missing
    for i in 0..CURRENT_SET {
        fetch_sets.push(i);
    }
    println!("Attempting to fetch card sets from cache");
    let mut card_sets: Vec<CardSet> = Vec::new();
    for path in dir {
        let file: ExpirationWrapper = match serde_json::from_reader(
            File::open(path.unwrap().path()).expect("something broke reading cache file"),
        ) {
            Ok(r) => r,
            Err(e) => {
                return Err(format!(
                    "failed to coerce cache file to ExpirationWrapper: {}",
                    e
                ))
            }
        };

        let id = file.card_set_json.card_set.set_info.set_id;
        if std::time::Duration::new(file.expire_time, 0) > time {
            println!("card set {} is up to date", id);
            let rem = fetch_sets.iter().position(|x| *x == id as u8).unwrap();
            fetch_sets.remove(rem);
            card_sets.push(file.card_set_json.card_set);
        } else {
            println!("card set {} is expired", id);
        }
    }

    let mut card_sets_wrapped: Vec<ExpirationWrapper> = Vec::new();
    for set in fetch_sets {
        println!("Fetching card set {} from API", set);
        let set = match fetch_card_set(set) {
            Ok(r) => r,
            Err(e) => return Err(e),
        };
        card_sets_wrapped.push(set);
    }

    for wrapper in card_sets_wrapped {
        let id = wrapper.card_set_json.card_set.set_info.set_id;
        let f = format!("card_set_{}.json", id);
        let path = cache_dir.join(f);
        let file = File::create(path).unwrap();
        let _ = serde_json::to_writer(file, &wrapper);
        card_sets.push(wrapper.card_set_json.card_set);
    }

    Ok(card_sets)
}

fn fetch_card_set(set: u8) -> Result<ExpirationWrapper, String> {
    let valve_api_path = format!("https://playartifact.com/cardset/{}", set);

    let redir: JsonRef = match reqwest::get(valve_api_path.as_str()) {
        Ok(mut r) => match r.json() {
            Ok(j) => j,
            Err(e) => return Err(format!("Error coercing response to JsonRef: {}", e)),
        },
        Err(e) => return Err(format!("Error reaching Valve redirect endpoint: {}", e)),
    };

    let card_set_url = format!("{}{}", redir.cdn_root, redir.url);

    let card_set_json: CardSetJson = match reqwest::get(card_set_url.as_str()) {
        Ok(mut r) => match r.json() {
            Ok(j) => j,
            Err(e) => return Err(format!("Error coercing response to CardSetJson: {}", e)),
        },
        Err(e) => return Err(format!("Error reaching Valve card set endpoint: {}", e)),
    };

    Ok(ExpirationWrapper {
        card_set_json,
        expire_time: redir.expire_time,
    })
}

pub fn map_ids_to_cards(sets: Vec<crate::CardSet>) -> HashMap<u32, crate::Card> {
    let mut map = HashMap::new();
    for set in sets {
        for card in set.card_list {
            map.insert(card.card_id, card);
        }
    }
    map
}

pub fn map_names_to_cards(sets: Vec<crate::CardSet>) -> HashMap<String, crate::Card> {
    let mut map = HashMap::new();
    for set in sets {
        for card in set.card_list {
            map.insert(card.card_name.english.clone(), card);
        }
    }
    map
}

mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn fetch_cards() {
        let sets = crate::get_all_card_sets().unwrap();
        for set in sets {
            println!("{:?}", set.set_info.name.english);
        }
    }
}
