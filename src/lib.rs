use std::collections::HashMap;
#[macro_use]
extern crate serde_derive;

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

#[cfg(test)]
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
}
