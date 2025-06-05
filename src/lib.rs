#![feature(generic_const_exprs)]
#![feature(variant_count)]

use std::{
    array,
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    mem::{self, variant_count},
};

use mlua::{DeserializeOptions, Lua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Deserializer};
use serde_repr::Deserialize_repr;
use strum::VariantArray;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash, VariantArray)]
enum CardSuit {
    Diamonds,
    Spades,
    Clubs,
    Hearts,
}
impl AsIndex for CardSuit {
    fn as_index(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash, VariantArray)]
#[repr(u8)]
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
impl AsIndex for CardValue {
    fn as_index(&self) -> u8 {
        *self as u8
    }
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

type EmptyTable = [(); 0];

#[derive(Debug, Deserialize)]
#[expect(non_snake_case)]
struct Game {
    STOP_USE: i32,
    bankrupt_at: u64,
    banned_keys: EmptyTable,
    base_reroll_cost: u64,
    bosses_used: BossesUsed,
    #[serde(deserialize_with = "enum_array")]
    cards_played: [CardPlayed; variant_count::<CardValue>()],
    chips: u64,
    consumeable_buffer: i32,
    // TODO: consumeable_usage: EmptyTable,
    consumeable_usage_total: Option<ConsumeableUsageTotal>,
    current_round: CurrentRound,
    disabled_ranks: EmptyTable,
    disabled_suits: EmptyTable,
    discount_percent: u8,
    dollars: u64,
    ecto_minus: u8,
    edition_rate: u8,
    first_shop_buffoon: bool,
    #[serde(deserialize_with = "enum_array_default")]
    hand_usage: [HandUsage; variant_count::<Hand>()],
    #[serde(deserialize_with = "enum_array")]
    hands: [HandData; variant_count::<Hand>()],
    hands_played: u16,
    inflation: i32,
    interest_amount: u32,
    interest_cap: u32,
    joker_buffer: u32,
    joker_rate: u32,
    joker_usage: EmptyTable,
    last_hand_played: Hand,
    legendary_mod: u32,
    max_jokers: u32,
    modifiers: EmptyTable,
    orbital_choices: OrbitalChoices,
    pack_size: u32,
    perishable_rounds: u32,
    planet_rate: u32,
    playing_card_rate: u32,
    pool_flags: EmptyTable,
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
    tags: EmptyTable,
    unused_discards: u16,
    used_vouchers: EmptyTable,
    used_jokers: HashMap<String, bool>, // TODO: add joker enum
    won: bool,
}

#[derive(Debug, Deserialize)]
struct BossesUsed {
    bl_arm: u8,
    bl_club: u8,
    bl_eye: u8,
    bl_final_acorn: u8,
    bl_final_bell: u8,
    bl_final_heart: u8,
    bl_final_leaf: u8,
    bl_final_vessel: u8,
    bl_fish: u8,
    bl_flint: u8,
    bl_goad: u8,
    bl_head: u8,
    bl_hook: u8,
    bl_house: u8,
    bl_manacle: u8,
    bl_mark: u8,
    bl_mouth: u8,
    bl_needle: u8,
    bl_ox: u8,
    bl_pillar: u8,
    bl_plant: u8,
    bl_psychic: u8,
    bl_serpent: u8,
    bl_tooth: u8,
    bl_wall: u8,
    bl_water: u8,
    bl_wheel: u8,
    bl_window: u8,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(transparent)]
struct Played(bool);
impl Indexer for Played {
    type Indexer = CardSuit;
}

#[derive(Debug, Deserialize, Clone, Default)]
struct CardPlayed {
    #[serde(deserialize_with = "enum_array_default")]
    suits: [Played; variant_count::<CardSuit>()],
    total: u64,
}
impl Indexer for CardPlayed {
    type Indexer = CardValue;
}

#[derive(Debug, Deserialize)]
struct ConsumeableUsageTotal {
    all: u16,
    planet: u16,
    spectral: u16,
    tarot: u16,
    tarot_planet: u16,
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
    Upcoming,
    Select,
}

#[derive(Debug, Deserialize)]
struct RoundBonus {
    discards: u16,
    next_hands: u16,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct HandUsage {
    count: u16,
}
impl Indexer for HandUsage {
    type Indexer = Hand;
}

#[derive(Debug, Deserialize, Clone, Default)]
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
    type Indexer;
}
impl Indexer for HandData {
    type Indexer = Hand;
}

fn enum_array<'de, D, T>(
    deserializer: D,
) -> Result<[T; mem::variant_count::<T::Indexer>()], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Clone + Indexer + Debug + Default,
    T::Indexer: Deserialize<'de> + AsIndex + Hash + Eq + VariantArray + Clone + Debug,
{
    enum_array_inner(deserializer, false)
}

/// Uses Default::default() for nonexistent variants
fn enum_array_default<'de, D, T>(
    deserializer: D,
) -> Result<[T; mem::variant_count::<T::Indexer>()], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Clone + Indexer + Debug + Default,
    T::Indexer: Deserialize<'de> + AsIndex + Hash + Eq + VariantArray + Clone + Debug,
{
    enum_array_inner(deserializer, true)
}

/// Parses into an array indexed by Indexer discriminant.
fn enum_array_inner<'de, D, T>(
    deserializer: D,
    default: bool,
) -> Result<[T; mem::variant_count::<T::Indexer>()], D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Clone + Indexer + Debug + Default,
    T::Indexer: Deserialize<'de> + AsIndex + Hash + Eq + VariantArray + Clone + Debug,
{
    let unwrap_fn = if default {
        Option::unwrap_or_default
    } else {
        Option::unwrap
    };
    let temp: HashMap<T::Indexer, T> = HashMap::deserialize(deserializer)?;
    let mut variants = array::from_fn(|i| T::Indexer::VARIANTS[i].clone());
    variants.sort_unstable_by_key(AsIndex::as_index);
    Ok(variants.map(|variant| unwrap_fn(temp.get(&variant).cloned())))
}

#[derive(Debug, Deserialize)]
struct OrbitalChoices {
    Small: Hand,
    Big: Hand,
    Boss: Hand,
}

#[derive(Debug, Deserialize)]
struct IdolCard {
    rank: CardValue,
    suit: CardSuit,
}

#[derive(Debug, Deserialize)]
struct MailCard {
    rank: CardValue,
}

#[derive(Debug, Deserialize)]
struct CurrentRound {
    ancient_card: OnlyCardSuit,
    cards_flipped: u8,
    castle_card: OnlyCardSuit,
    // should only error if current hand isnt initialised
    #[serde(deserialize_with = "none_on_error")]
    current_hand: Option<CurrentHand>,
    discards_left: u8,
    discards_used: u8,
    dollars: u64,
    free_rerolls: u8,
    hands_left: u8,
    hands_played: u8,
    idol_card: IdolCard,
    jokers_purchased: u8,
    mail_card: MailCard,
    most_played_poker_hand: Hand,
    reroll_cost: u64,
    reroll_cost_increase: u64,
    round_dollars: u64,
    used_packs: EmptyTable,
}

/// Returns none on deserialization error
fn none_on_error<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(T::deserialize(deserializer).ok())
}

#[derive(Debug, Deserialize)]
struct OnlyCardSuit {
    suit: CardSuit,
}

type HandLevel = String; // TODO: actually do this
#[derive(Debug, Deserialize)]
struct CurrentHand {
    chip_total: u64,
    chips: u64,
    hand_level: HandLevel,
    handname: Hand,
    mult: u64,
}

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
    // For some reason not present in G.GAME.hands
    // #[serde(rename = "Royal Flush")]
    // RoyalFlush,
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
        let g: Table = globals.get("G")?;
        let game: Value = g.get("GAME")?;
        let options = DeserializeOptions::new()
            .deny_unsupported_types(false)
            .deny_recursive_tables(false);
        let r: mlua::Result<Game> = lua.from_value_with(dbg!(game), options); // TODO: deserialize G into State
        dbg!(r);
        Ok(())
    })?;

    table.set("print_game", print_game)?;

    Ok(table)
}
