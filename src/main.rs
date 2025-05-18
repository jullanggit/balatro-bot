#![feature(iter_intersperse)]

use std::{
    collections::HashMap,
    env,
    io::stdin,
    marker::PhantomData,
    net::{SocketAddr, UdpSocket},
    process::{Child, Command, Stdio},
    thread::sleep,
    time::Duration,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::Deserialize_repr;
use strum::IntoStaticStr;

/// 1-based indices
type Indices = Vec<u8>;
#[derive(Debug, Clone, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
// Vec<u8>: 1-based indices
pub enum BotAction {
    SelectBlind,
    SkipBlind,
    PlayHand(Indices),
    DiscardHand(Indices),
    EndShop,
    RerollShop,
    BuyCard(Indices),
    BuyVoucher(Indices),
    BuyBooster(Indices),
    SelectBoosterCard(Indices),
    SkipBoosterPack,
    SellJoker(Indices),
    UseConsumable(Indices),
    SellConsumable(Indices),
    RearrangeJokers,
    RearrangeConsumables,
    RearrangeHand,
    Pass,
    StartRun,
}
impl Serialize for BotAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use BotAction::*;

        let name: &'static str = self.into();
        if let PlayHand(cards)
        | DiscardHand(cards)
        | BuyCard(cards)
        | BuyVoucher(cards)
        | BuyBooster(cards)
        | SelectBoosterCard(cards)
        | SellJoker(cards)
        | UseConsumable(cards)
        | SellConsumable(cards) = self
        {
            // Format the inner vector as a comma-separated string
            let joined = cards
                .iter()
                .map(|c| c.to_string())
                .intersperse(String::from(","))
                .collect::<String>();
            serializer.serialize_str(&format!("{name}|{joined}"))
        } else {
            serializer.serialize_str(name)
        }
    }
}

pub struct Bot {
    socket: UdpSocket,
    balatro_instance: Option<Child>,
    addr: SocketAddr,
}
impl Bot {
    fn start_balatro(&mut self) {
        let home = env::var("HOME").unwrap();
        self.balatro_instance = Some(
            Command::new(format!(
                "{home}/.games/Balatro windows 2/Balatro/run_lovely_linux.sh"
            ))
            .arg(self.addr.port().to_string())
            .stdout(Stdio::null())
            .spawn()
            .expect("Failed to start Balatro"),
        );
    }
    fn send_command(&self, command: &str) -> std::io::Result<usize> {
        self.socket.send_to(command.as_bytes(), self.addr)
    }
    pub fn run(&mut self) {
        let mut buffer = [0u8; 65536];
        let mut cli_buffer = String::new();

        loop {
            // cli commands
            {
                cli_buffer.clear();
                stdin().read_line(&mut cli_buffer).unwrap();
                // if the buffer isnt only a newline
                if cli_buffer.len() > 1 {
                    self.send_command(&cli_buffer).unwrap();
                }
            }

            self.send_command("HELLO").unwrap();

            match self.socket.recv_from(&mut buffer) {
                Ok((size, _)) => {
                    let msg = String::from_utf8_lossy(&buffer[..size]);
                    match serde_json_path_to_error::from_str::<GameData>(dbg!(&msg)) {
                        Ok(game_data) => {
                            dbg!(game_data);
                        }
                        Err(e) => {
                            eprintln!("Error: {e}");
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        eprintln!("Socket error: {}", e);
                    }
                }
            }
            sleep(Duration::from_secs(1));
        }
    }
}

type Idk = PhantomData<bool>;
#[derive(Deserialize, Debug)]
struct GameData {
    shop: Vec<Idk>,
    discount_percent: u8,
    interest_cap: u64,
    inflation: u64,
    num_hands_played: u64,
    tags: Vec<Idk>,
    #[serde(rename = "waitingFor")]
    waiting_for: WaitingFor, // "start_run"
    #[serde(rename = "waitingForAction")]
    waiting_for_action: bool,
    max_jokers: u8,
    ante: Ante,
    handscores: Vec<Idk>,
    bankrupt_at: u8, // 0
    current_round: CurrentRound,
    hand: Vec<Card>,
    consumables: Vec<Idk>,
    deckback: Vec<Idk>,
    deck: Vec<Idk>,
    dollars: u64,
    round: u64,
    state: GameState,
    jokers: Vec<Idk>,
}

#[derive(Debug)]
struct Ante {
    blinds: Option<Blind>,
}
impl<'de> Deserialize<'de> for Ante {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            blinds: if let Ok(map) = HashMap::<String, Blind>::deserialize(deserializer) {
                let mut iter = map.into_iter();
                let (name, blind) = iter.next().unwrap();
                assert!(&name == "blinds");
                assert!(iter.next().is_none());
                Some(blind)
            } else {
                None
            },
        })
    }
}

#[derive(Deserialize, Debug)]
struct Blind {
    ondeck: OnDeck,
}
#[derive(Deserialize, Debug)]
enum OnDeck {
    Small,
    Big,
    Boss,
}

#[derive(Deserialize, Debug)]
struct CurrentRound {
    discards_left: u8,
}

// TODO: clean this up a bit
#[derive(Deserialize, Debug)]
struct Card {
    value: Value,
    suit: Suit,
    label: Label,
}

#[derive(Debug)]
enum Value {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use Value::*;
        use serde::de::Error;

        let s = String::deserialize(deserializer)?;

        Ok(match s.as_str() {
            "Ace" => Ace,
            "2" => Two,
            "3" => Three,
            "4" => Four,
            "5" => Five,
            "6" => Six,
            "7" => Seven,
            "8" => Eight,
            "9" => Nine,
            "10" => Ten,
            "Jack" => Jack,
            "Queen" => Queen,
            "King" => King,
            other => return Err(Error::custom(format!("Invalid value: {other}"))),
        })
    }
}

#[derive(Debug, Deserialize)]
enum Suit {
    Diamonds,
    Spades,
    Clubs,
    Hearts,
}

#[derive(Debug, Deserialize)]
enum Label {
    #[serde(rename = "Base Card")]
    BaseCard,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WaitingFor {
    StartRun,
    SkipOrSelectBlind,
    SelectCardsFromHand,
    SelectShopAction,
    SelectBoosterAction,
    SellJokers,
    RearrangeJokers,
    UseOrSellConsumables,
    RearrangeConsumables,
    RearrangeHand,
}

fn main() {
    let addr = format!("127.0.0.1:{}", "12345").parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(1000)))
        .unwrap();

    let mut bot = Bot {
        socket,
        balatro_instance: None,
        addr,
    };

    bot.start_balatro();

    bot.run();
}
