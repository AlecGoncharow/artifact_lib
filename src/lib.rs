#[macro_use]
extern crate serde_derive;
extern crate artifact_serde;
extern crate directories;
extern crate reqwest;
extern crate serde_json;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{create_dir, read_dir, File};
use std::time::{SystemTime, UNIX_EPOCH};
use artifact_serde::de::decode;
use directories::ProjectDirs;

const CURRENT_SET: u8 = 2;

/// Wraps the [CardSetJson](struct.CardSetJson.html) with the expiration time.
#[derive(Serialize, Deserialize, Debug)]
struct ExpirationWrapper {
    expire_time: u64,
    card_set_json: CardSetJson,
}
/// This is the top level of the response JSON provided by the Valve [API](https://github.com/ValveSoftware/ArtifactDeckCode#card-set-api)
#[derive(Serialize, Deserialize, Debug)]
pub struct CardSetJson {
    pub card_set: CardSet,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardSet {
    pub version: u32,
    pub set_info: SetInfo,
    pub card_list: Vec<Card>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetInfo {
    pub set_id: u32,
    pub pack_item_def: u32,
    pub name: TranslatedText,
}
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
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

impl PartialEq for TranslatedText {
    fn eq(&self, other: &TranslatedText) -> bool {
        self.english == other.english
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
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
    pub armor: i32,
    #[serde(default)]
    pub hit_points: u32,
    pub references: Vec<Reference>,
}

impl std::cmp::PartialEq for Card {
    fn eq(&self, other: &Card) -> bool {
        self.card_id == other.card_id
    }
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum CardColor {
    Red,
    Blue,
    Black,
    Green,
    Item,
}
impl std::fmt::Display for CardColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CardColor::Red => write!(f, "Red"),
            CardColor::Blue => write!(f, "Blue"),
            CardColor::Black => write!(f, "Black"),
            CardColor::Green => write!(f, "Green"),
            CardColor::Item => write!(f, "Item"),
        }
    }
}
impl Card {
    pub fn get_color(&self) -> CardColor {
        if self.is_red {
            CardColor::Red
        } else if self.is_blue {
            CardColor::Blue
        } else if self.is_black {
            CardColor::Black
        } else if self.is_blue {
            CardColor::Blue
        } else if self.is_green {
            CardColor::Green
        } else {
            CardColor::Item
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct HeroCard {
    pub card: Card,
    pub turn: u32,
    pub color: CardColor,
}
impl PartialEq for HeroCard {
    fn eq(&self, other: &HeroCard) -> bool {
        self.turn == other.turn
    }
}
impl Ord for HeroCard {
    fn cmp(&self, other: &HeroCard) -> Ordering {
        self.turn.cmp(&other.turn)
    }
}

impl PartialOrd for HeroCard {
    fn partial_cmp(&self, other: &HeroCard) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
// Red < Blue < Black < Green | mana < gold
#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct CardCard {
    pub card: Card,
    pub count: u32,
    pub color: CardColor,
}
impl PartialEq for CardCard {
    fn eq(&self, other: &CardCard) -> bool {
        self.card.card_id == other.card.card_id
    }
}

impl Ord for CardCard {
    fn cmp(&self, other: &CardCard) -> Ordering {
        match self.color {
            CardColor::Item => match other.color {
                CardColor::Item => self.card.gold_cost.cmp(&other.card.gold_cost),
                _ => Ordering::Greater,
            },
            _ => match other.color {
                CardColor::Item => Ordering::Less,
                _ => self.card.mana_cost.cmp(&other.card.mana_cost),
            },
        }
    }
}

impl PartialOrd for CardCard {
    fn partial_cmp(&self, other: &CardCard) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Artifact deck representation, typically derived from Artifact Deck Codes.
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

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Image {
    #[serde(default)]
    pub default: String,
}
impl PartialEq for Image {
    fn eq(&self, other: &Image) -> bool {
        self.default == other.default
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Reference {
    #[serde(default)]
    pub card_id: u32,
    #[serde(default)]
    pub ref_type: String,
    #[serde(default)]
    pub count: u32,
}
impl PartialEq for Reference {
    fn eq(&self, other: &Reference) -> bool {
        self.card_id == other.card_id
    }
}
#[derive(Debug, Clone)]
pub enum NamedCard {
    Single(Card),
    Multiple(Vec<Card>),
}

impl NamedCard {
    pub fn is_single(&self) -> bool {
        match self {
            NamedCard::Single(_) => true,
            NamedCard::Multiple(_) => false,
        }
    }

    pub fn into_vec(self) -> Vec<Card> {
        match self {
            NamedCard::Single(s) => vec![s.clone()],
            NamedCard::Multiple(m) => m.clone(),
        }
    }

    /// Due to the nature of the data provided by Valve, the active component
    /// of cards are sometimes called the same way as the card itself.
    /// This function takes this into account and unwraps then NamedCard into a card, where
    /// Multiple types are unwrapped into the card which contains the actual card and not the
    /// active component, assuming it can find the card matching this case.
    pub fn into_card(self) -> Option<Card> {
        match self {
            NamedCard::Single(s) => Some(s.clone()),
            NamedCard::Multiple(m) => {
                let mut ret: Option<Card> = None;
                for card in m {
                    if card.large_image.default != "" {
                        ret = Some(card.clone());
                    }
                }
                ret
            }
        }
    }
}
/// Helper struct that will store the [CardSets](struct.CardSet.html) and a couple
/// HashMaps for fast indexing
/// # Example Usage
/// ```
/// let my_artifact = artifact_lib::Artifact::new();
/// let named_card = match my_artifact.card_from_name("Storm Spirit") {
///     Some(c) => match c {
///         artifact_lib::NamedCard::Single(s) => s,
///         artifact_lib::NamedCard::Multiple(m) => m.first().unwrap(),
///     }
///     _ => panic!("this should not happen"),
/// };
/// let id_card = my_artifact.card_from_id(named_card.card_id).unwrap();
///
/// let my_adc = "ADCJWkTZX05uwGDCRV4XQGy3QGLmqUBg4GQJgGLGgO7AaABR3JlZW4vQmxhY2sgRXhhbXBsZQ__";
/// let my_deck = my_artifact.get_deck(my_adc);
/// ```
pub struct Artifact {
    pub card_sets: Vec<CardSet>,
    pub id_map: HashMap<u32, Card>,
    pub name_map: HashMap<String, NamedCard>,
}

impl Artifact {
    /// Creates a new Artifact object, prepopulated with all the
    /// card sets and a couple HashMaps that help with indexing
    /// into the card sets
    pub fn new() -> Self {
        let card_sets = get_all_card_sets().unwrap();
        let id_map = map_ids_to_cards(card_sets.clone());
        let name_map = map_names_to_cards(card_sets.clone());
        Self {
            card_sets,
            id_map,
            name_map,
        }
    }

    pub fn card_from_name(&self, name: &str) -> Option<&NamedCard> {
        self.card_from_name_string(&String::from(name))
    }

    pub fn card_from_name_string(&self, name: &String) -> Option<&NamedCard> {
        self.name_map.get(&name.to_lowercase())
    }

    pub fn card_from_id(&self, id: u32) -> Option<&Card> {
        self.id_map.get(&id)
    }

    /// Takes in an ADC and returns the corresponding Deck, including
    /// Hero reference cards.
    pub fn get_deck(&self, adc: &str) -> Result<Deck, String> {
        let mut decoded_deck = decode(adc).expect("failed to decode adc");
        let mut heroes = Vec::new();
        for hero in decoded_deck.heroes {
            let card = match self.id_map.get(&hero.id) {
                Some(c) => c.clone(),
                None => continue,
            };
            let color = card.get_color();
            let refer = card.references.clone();
            for r in refer {
                if r.ref_type == "includes" {
                    decoded_deck
                        .cards
                        .push(artifact_serde::de::DeserializedCard {
                            id: r.card_id,
                            count: r.count,
                        });
                }
            }

            heroes.push(HeroCard {
                card,
                turn: hero.turn,
                color: color,
            });
        }

        let mut cards = Vec::new();
        for ref_card in decoded_deck.cards {
            let card = match self.id_map.get(&ref_card.id) {
                Some(c) => c.clone(),
                None => continue,
            };
            let color = card.get_color();

            cards.push(CardCard {
                card,
                count: ref_card.count,
                color,
            });
        }

        Ok(Deck {
            heroes,
            cards,
            name: decoded_deck.name,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct JsonRef {
    cdn_root: String,
    url: String,
    expire_time: u64,
}

/// This function will search the user's local cache for
/// the card set data, if not found or out of date, will
/// fetch updates from Valve's API and create and update the cached files.
/// Once that process is complete, it will return a Vec of [CardSets](struct.CardSet.html).
pub fn get_all_card_sets() -> Result<Vec<CardSet>, String> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("failed to get unix timestamp");
    let proj_dir = ProjectDirs::from("", "", "artifact_lib")
        .expect("failed to build ProjectDirs");
    let cache_dir = proj_dir.cache_dir();
    let dir = match read_dir(cache_dir) {
        Ok(d) => d,
        Err(_) => match create_dir(cache_dir) {
            Ok(_) => read_dir(cache_dir).expect("failed to read cache directory"),
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
            File::open(path.expect("failed to open card set json in cache").path())
                .expect("failed to read cache file"),
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
            let rem = fetch_sets
                .iter()
                .position(|x| *x == id as u8)
                .expect("something terrible has happened, this is a bug");
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
        let file = File::create(path).expect("failed to create file in cache");
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

/// Returns a HashMap mapping cards' card_ids  to the respective card
pub fn map_ids_to_cards(sets: Vec<crate::CardSet>) -> HashMap<u32, crate::Card> {
    let mut map = HashMap::new();
    for set in sets {
        for card in set.card_list {
            map.insert(card.card_id, card);
        }
    }
    map
}

/// Returns a HashMap mapping cards' English names to the respective card
pub fn map_names_to_cards(sets: Vec<crate::CardSet>) -> HashMap<String, crate::NamedCard> {
    let mut map = HashMap::new();
    for set in sets {
        for card in set.card_list {
            let value = map.get(&card.card_name.english.to_lowercase());
            match value {
                Some(c) => match c {
                    NamedCard::Multiple(m) => {
                        let mut vec = m.clone();
                        vec.push(card.clone());
                        map.insert(
                            card.card_name.english.to_lowercase(),
                            NamedCard::Multiple(vec),
                        );
                    }
                    NamedCard::Single(s) => {
                        let mut vec: Vec<Card> = Vec::new();
                        vec.push(s.clone());
                        vec.push(card.clone());
                        map.insert(
                            card.card_name.english.to_lowercase(),
                            NamedCard::Multiple(vec),
                        );
                    }
                },
                None => {
                    map.insert(
                        card.card_name.english.to_lowercase(),
                        NamedCard::Single(card),
                    );
                }
            }
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
    fn test_artifact() {
        let my_artifact: super::Artifact = super::Artifact::new();
        let named_card: &crate::Card = match my_artifact.card_from_name("Storm Spirit") {
            Some(c) => match c {
                crate::NamedCard::Single(s) => s,
                crate::NamedCard::Multiple(m) => m.first().unwrap(),
            },
            None => panic!("could not get Storm Spirit"),
        };
        let id_card = my_artifact
            .card_from_id(named_card.card_id)
            .expect("could not get card from id");
        assert_eq!(named_card, id_card);

        let my_adc = "ADCJWkTZX05uwGDCRV4XQGy3QGLmqUBg4GQJgGLGgO7AaABR3JlZW4vQmxhY2sgRXhhbXBsZQ__";
        let _my_deck = my_artifact.get_deck(my_adc);
        //println!("{:?}", my_deck);
    }
}
