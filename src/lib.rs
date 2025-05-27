#![feature(generic_const_exprs)]
#![feature(variant_count)]

use std::{array, collections::HashMap, hash::Hash, mem};

use mlua::{DeserializeOptions, Lua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Deserializer};
use serde_repr::Deserialize_repr;
use strum::VariantArray;

#[derive(Debug, Deserialize)]
enum CardSuit {
    Diamonds,
    Spades,
    Clubs,
    Hearts,
}

#[derive(Debug, Deserialize)]
enum CardValue {
    Ace,
    #[serde(rename = "2")]
    Two,
    #[serde(rename = "3")]
    Three,
    #[serde(rename = "4")]
    Four,
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "6")]
    Six,
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "8")]
    Eight,
    #[serde(rename = "9")]
    Nine,
    #[serde(rename = "10")]
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Debug, Deserialize)]
struct State {
    __orig: Orig,
}

#[derive(Debug, Deserialize)]
#[expect(non_snake_case)]
struct Orig {
    ACC: i32,
    GAME: Game,
    MAJORS: u8,
    STAGE: Stage,
    STATE: GameState,
}
#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum GameState {
    SelectingHand = 1,
    HandPlayed,
    DrawToHand,
    GameOver,
    Shop,
    PlayTarot,
    BlindSelect,
    RoundEval,
    TarotPack,
    PlanetPack,
    Menu,
    Tutorial,
    Splash,
    Sandbox,
    SpectralPack,
    DemoCta,
    StandardPack,
    BuffoonPack,
    NewRound,
}
#[derive(Debug, Deserialize)]
enum Stage {
    MainMenu = 1,
    Run,
    Sandbox,
}

type EmptyBraces = [(); 0];

#[derive(Debug, Deserialize)]
#[expect(non_snake_case)]
struct Game {
    STOP_USE: i32,
    bankrupt_at: u64,
    banned_keys: EmptyBraces,
    base_reroll_cost: u64,
    // [(suits, total)]
    cards_played: Vec<([bool; 4], u32)>, // sorted after CardValue & CardColor discriminant
    chips: u64,
    common_mod: i32,
    consumeable_buffer: i32,
    consumeable_usage: EmptyBraces,
    current_round: CurrentRound,
    disabled_ranks: EmptyBraces,
    disabled_suits: EmptyBraces,
    discount_percent: u8,
    dollars: u64,
    ecto_minus: u8,
    edition_rate: u8,
    hand_usage: EmptyBraces,
    #[serde(deserialize_with = "enum_array")]
    hands: [HandData; 13], // [; num hands]
    hands_played: u32,
    inflation: i32,
    interest_amount: u32,
    interest_cap: u32,
    joker_buffer: u32,
    joker_rate: u32,
    joker_usage: EmptyBraces,
    legendary_mod: u32,
    max_jokers: u32,
    modifiers: EmptyBraces,
    pack_size: u32,
    perishable_rounds: u32,
    planet_rate: u32,
    playing_card_rate: u32,
    pool_flags: EmptyBraces,
    round: u16,
    round_bonus: RoundBonus,
    round_resets: RoundResets,
    round_scores: RoundScores,
    shop: Shop,
    skips: u16,
    spectral_rate: u8,
    stake: u16,
    tarot_rate: u8,
    tag_tally: i32,
    tags: EmptyBraces,
    unused_discards: u16,
    used_vouchers: EmptyBraces,
    used_jokers: HashMap<String, bool>,
    won: bool,
}
#[derive(Debug, Deserialize)]
struct Shop {
    joker_max: u8,
}
#[derive(Debug, Deserialize)]
struct RoundScores {
    #[serde(deserialize_with = "amt")]
    cards_discarded: u16,
    #[serde(deserialize_with = "amt")]
    cards_played: u16,
    #[serde(deserialize_with = "amt")]
    cards_purchased: u16,
    #[serde(deserialize_with = "amt")]
    furthest_ante: u16,
    #[serde(deserialize_with = "amt")]
    furthest_round: u16,
    #[serde(deserialize_with = "amt")]
    hand: u64,
    #[serde(deserialize_with = "amt")]
    poker_hand: u64,
    #[serde(deserialize_with = "amt")]
    times_rerolled: u16,
}
fn amt<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    #[derive(Deserialize)]
    struct Temp<T> {
        amt: T,
    }

    Ok(Temp::<T>::deserialize(deserializer)?.amt)
}

#[derive(Debug, Deserialize)]
struct RoundResets {
    ante: u16,
    blind_ante: u16,
    blind_states: BlindStates,
    boss_rerolled: bool,
    discards: u16,
    free_rerolls: u16,
    hands: u16,
    reroll_cost: u16,
}
#[derive(Debug, Deserialize)]
struct BlindStates {
    Big: BlindState,
    Boss: BlindState,
    Small: BlindState,
}
#[derive(Debug, Deserialize)]
enum BlindState {
    UpComing,
    Select,
}

#[derive(Debug, Deserialize)]
struct RoundBonus {
    discards: u16,
    next_hands: u16,
}

#[derive(Debug, Deserialize, Clone)]
struct HandData {
    chips: u64,
    l_chips: u64,
    l_mult: u64,
    level: u64,
    mult: u64,
    played: u64,
    played_this_round: u64,
    s_chips: u32,
    s_mult: u32,
}
trait Indexer {
    type INDEXER;
}
impl Indexer for HandData {
    type INDEXER = Hand;
}
fn enum_array<'de, D, T>(
    deserializer: D,
) -> Result<[T; mem::variant_count::<T::INDEXER>()], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Clone + Indexer,
    T::INDEXER: Deserialize<'de> + AsIndex + Hash + Eq + VariantArray + Clone,
{
    let temp: HashMap<T::INDEXER, T> = HashMap::deserialize(deserializer)?;
    let mut variants = array::from_fn(|i| T::INDEXER::VARIANTS[i].clone());
    variants.sort_unstable_by_key(AsIndex::as_index);
    Ok(variants.map(|variant| temp.get(&variant).cloned().unwrap()))
}

#[derive(Debug, Deserialize)]
struct IdolCard {
    rank: CardValue,
    suit: CardSuit,
}

#[derive(Debug, Deserialize)]
struct CurrentRound {
    ancient_card: CardSuit,
    cards_flipped: u8,
    castle_card: CardSuit,
    current_hand: CurrentHand,
    discards_left: u8,
    discards_used: u8,
    dollars: u64,
    free_rerolls: u8,
    hands_left: u8,
    hands_played: u8,
    idol_card: IdolCard,
    jokers_purchased: u8,
    mail_card: CardValue,
    most_played_poker_hand: Hand,
    reroll_cost: u64,
    reroll_cost_increase: u64,
    round_dollars: u64,
    used_packs: [(); 1],
}

#[derive(Debug, Deserialize)]
struct CurrentHand {
    chip_total: u64,
    chips: u64,
    hand_level: HandLevel,
    handname: Hand,
    mult: u64,
}

#[derive(Debug, Deserialize)]
struct HandLevel {}
#[derive(Debug, Deserialize, VariantArray, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(u8)]
enum Hand {
    #[serde(rename = "High Card")]
    HighCard,
    Pair,
    #[serde(rename = "Two Pair")]
    TwoPair,
    #[serde(rename = "Three of a Kind")]
    ThreeOfAKind,
    Straight,
    Flush,
    #[serde(rename = "Full House")]
    FullHouse,
    #[serde(rename = "Four of a Kind")]
    FourOfAKind,
    #[serde(rename = "Straight Flush")]
    StraightFlush,
    #[serde(rename = "Royal Flush")]
    RoyalFlush,
    #[serde(rename = "Five of a Kind")]
    FiveOfAKind,
    #[serde(rename = "Flush House")]
    FlushHouse,
    #[serde(rename = "Flush Five")]
    FlushFive,
}
trait AsIndex {
    fn as_index(&self) -> u8;
}
impl AsIndex for Hand {
    fn as_index(&self) -> u8 {
        *self as u8
    }
}

#[mlua::lua_module]
fn print_game(lua: &Lua) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    let print_game = lua.create_function(|lua, ()| {
        let globals = lua.globals();
        let g: Value = globals.get("G")?;
        let options = DeserializeOptions::new()
            .deny_unsupported_types(false)
            .deny_recursive_tables(false);
        let r: mlua::Result<State> = lua.from_value_with(g, options);
        // let game: Table = g.get("GAME")?;
        dbg!(r);
        Ok(())
    })?;

    table.set("print_game", print_game)?;

    Ok(table)
}
