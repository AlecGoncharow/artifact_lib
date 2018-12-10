#[macro_use]
extern crate serde_derive;
extern crate directories;
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
/// Once that process is complete, it will return a Vec of [CardSet](struct.CardSet)s.
pub fn get_all_card_sets() -> Result<Vec<CardSet>, String> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let proj_dir = directories::ProjectDirs::from("", "", "artifact_lib").unwrap();
    let cache_dir = proj_dir.cache_dir();
    let dir = match read_dir(cache_dir) {
        Ok(d) => d,
        Err(_) => match create_dir(cache_dir) {
            Ok(_) => read_dir(cache_dir).unwrap(),
            Err(e) => panic!(
                "Error reading or creating directory: {:?}, {}",
                cache_dir, e
            ),
        },
    };

    println!("Attempting to fetch card sets from cache");
    // allow data fetching if dir is empty
    let mut fetch_data = true;
    for path in dir {
        fetch_data = false;
        let file: ExpirationWrapper = serde_json::from_reader(
            File::open(path.unwrap().path()).expect("something broke reading cache file"),
        )
        .unwrap();
        if std::time::Duration::new(file.expire_time, 0) < time {
            println!("A card set has expired, fetching new card sets");
            fetch_data = true;
            break;
        }
    }

    // if new or expired cards
    if fetch_data {
        println!("Attempting to fetch card sets from API");
        let mut card_sets_wrapped: Vec<ExpirationWrapper> = Vec::new();
        let valve_api_path = "https://playartifact.com/cardset/";
        for i in 0..CURRENT_SET {
            let url = format!("{}{}", valve_api_path, i);
            let redir: JsonRef = reqwest::get(url.as_str()).unwrap().json().unwrap();
            let card_set_url = format!("{}{}", redir.cdn_root, redir.url);

            let card_set: crate::CardSetJson =
                reqwest::get(card_set_url.as_str()).unwrap().json().unwrap();

            card_sets_wrapped.push(ExpirationWrapper {
                card_set_json: card_set,
                expire_time: redir.expire_time,
            });
        }

        for (i, wrapper) in card_sets_wrapped.iter().enumerate() {
            let f = format!("card_set_{}.json", i);
            let path = cache_dir.join(f);
            let file = File::create(path).unwrap();
            let _ = serde_json::to_writer(file, &wrapper);
        }
    }

    // finally fetch the card set from cache for real
    let dir = match read_dir(cache_dir) {
        Ok(d) => d,
        Err(_) => match create_dir(cache_dir) {
            Ok(_) => read_dir(cache_dir).unwrap(),
            Err(e) => panic!(
                "Error reading or creating directory: {:?}, {}",
                cache_dir, e
            ),
        },
    };

    let mut card_sets: Vec<CardSet> = Vec::new();
    for path in dir {
        let file: ExpirationWrapper = serde_json::from_reader(
            File::open(path.unwrap().path()).expect("something broke reading cache file"),
        )
        .unwrap();

        card_sets.push(file.card_set_json.card_set);
    }

    Ok(card_sets)
}

/// Takes in a vector of JSON formatted &str and attempts to coerce them into CardSetJson,
/// if successful, maps card_ids to Cards.\
/// The JSON should take the form mentioned
/// [here](https://github.com/ValveSoftware/ArtifactDeckCode)
/// ```ignore
///{
///  "card_set": {
///    "version": 1,
///  "set_info": {
///   "set_id": 0,
///    "pack_item_def": 0,
///     "name": {
///        "english": "Base Set"
///      }
///    },
///   "card_list": [{
///
///   "card_id": 4000,
///   "base_card_id": 4000,
///    "card_type": "Hero",
///   "card_name": {
///     "english": "Farvhan the Dreamer"
///  },
///   "card_text": {
///      "english": "Pack Leadership<BR>\nFarvhan the Dreamer's allied neighbors have +1 Armor."
///    },
///     "mini_image": {
///       "default": "<url to png>"
///     },
///    "large_image": {
///       "default": "<url to png>"
///      },
///     "ingame_image": {
///       "default": "<url to png>"
///    },
///    "is_green": true,
///    "attack": 4,
///    "hit_points": 10,
///      "references": [{
///      "card_id": 4002,
///        "ref_type": "includes",
///          "count": 3
///  },
///        {
///        "card_id": 4001,
///      "ref_type": "passive_ability"
///        }
///    ]
///
///
///    },
///    ..... more cards ....
///
///    ]
///  }
///}
///```
///
pub fn map_card_ids_to_cards_from_str(
    sets: Vec<&str>,
) -> Result<HashMap<u32, crate::Card>, String> {
    let mut d_sets = Vec::new();
    for set in sets {
        let s: crate::CardSetJson = match serde_json::from_str(set) {
            Ok(s) => s,
            Err(e) => {
                let error_string = format!("Invalid JSON input: {}", e);
                return Err(error_string);
            }
        };

        let d = s.card_set;
        d_sets.push(d);
    }

    Ok(set_up_deck_map(d_sets))
}

pub fn set_up_deck_map(sets: Vec<crate::CardSet>) -> HashMap<u32, crate::Card> {
    let mut map = HashMap::<u32, crate::Card>::new();
    for set in sets {
        for card in set.card_list {
            map.insert(card.card_id, card);
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
        let _sets = crate::get_all_card_sets();
    }
}
