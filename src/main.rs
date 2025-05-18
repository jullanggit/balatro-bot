use std::{
    env,
    marker::PhantomData,
    net::{SocketAddr, UdpSocket},
    process::{Child, Command, Stdio},
    thread::sleep,
    time::Duration,
};

use serde::{Deserialize, Deserializer};
use serde_repr::Deserialize_repr;

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

        loop {
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

#[derive(Deserialize, Debug)]
struct Ante {
    blinds: Blind,
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
    #[serde(deserialize_with = "from_str")]
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
fn from_str<'de, D>(deserializer: D) -> Result<Value, D::Error>
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

    sleep(Duration::from_secs(5));

    bot.run();
}
