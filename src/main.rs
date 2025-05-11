use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::process::{Child, Command};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameState {
    SelectingHand,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BotAction {
    SelectBlind,
    SkipBlind,
    PlayHand,
    DiscardHand,
    EndShop,
    RerollShop,
    BuyCard,
    BuyVoucher,
    BuyBooster,
    SelectBoosterCard,
    SkipBoosterPack,
    SellJoker,
    UseConsumable,
    SellConsumable,
    RearrangeJokers,
    RearrangeConsumables,
    RearrangeHand,
    Pass,
    StartRun,
}

pub trait BotBehavior {
    fn skip_or_select_blind(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn select_cards_from_hand(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn select_shop_action(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn select_booster_action(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn sell_jokers(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn rearrange_jokers(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn use_or_sell_consumables(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn rearrange_consumables(&mut self, game_state: &GameData) -> Vec<BotAction>;
    fn rearrange_hand(&mut self, game_state: &GameData) -> Vec<BotAction>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub state: GameState,
    pub waiting_for: String,
    pub hand: Vec<Card>,
    pub jokers: Vec<Joker>,
    pub consumables: Vec<Consumable>,
    pub shop: ShopState,
    pub current_round: RoundState,
    // Add other game state fields
}

pub struct Bot<B: BotBehavior> {
    behavior: B,
    socket: UdpSocket,
    balatro_instance: Option<Child>,
    addr: SocketAddr,
    state: HashMap<String, serde_json::Value>,
}

impl<B: BotBehavior> Bot<B> {
    pub fn new(
        behavior: B,
        deck: &str,
        stake: u32,
        seed: Option<&str>,
        challenge: Option<&str>,
        bot_port: u16,
    ) -> std::io::Result<Self> {
        let addr = format!("127.0.0.1:{}", bot_port).parse().unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(1000)))?;

        Ok(Self {
            behavior,
            socket,
            balatro_instance: None,
            addr,
            state: HashMap::new(),
        })
    }

    pub fn start_balatro(&mut self) {
        let balatro_path = r"C:\Program Files (x86)\Steam\steamapps\common\Balatro\Balatro.exe";
        self.balatro_instance = Some(
            Command::new(balatro_path)
                .arg(self.addr.port().to_string())
                .spawn()
                .expect("Failed to start Balatro"),
        );
    }

    pub fn run(&mut self) {
        let mut buffer = [0u8; 65536];

        loop {
            self.send_command("HELLO").unwrap();

            match self.socket.recv_from(&mut buffer) {
                Ok((size, src)) => {
                    let msg = String::from_utf8_lossy(&buffer[..size]);
                    if let Ok(game_data) = serde_json::from_str::<GameData>(&msg) {
                        self.handle_game_state(game_data);
                    } else if msg.starts_with("HELLO") {
                        // Handle hello response
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        eprintln!("Socket error: {}", e);
                    }
                }
            }
        }
    }

    fn handle_game_state(&mut self, game_data: GameData) {
        let action = match game_data.waiting_for.as_str() {
            "start_run" => self.handle_start_run(),
            "skip_or_select_blind" => self.behavior.skip_or_select_blind(&game_data),
            "select_cards_from_hand" => self.behavior.select_cards_from_hand(&game_data),
            "select_shop_action" => self.behavior.select_shop_action(&game_data),
            "select_booster_action" => self.behavior.select_booster_action(&game_data),
            "sell_jokers" => self.behavior.sell_jokers(&game_data),
            "rearrange_jokers" => self.behavior.rearrange_jokers(&game_data),
            "use_or_sell_consumables" => self.behavior.use_or_sell_consumables(&game_data),
            "rearrange_consumables" => self.behavior.rearrange_consumables(&game_data),
            "rearrange_hand" => self.behavior.rearrange_hand(&game_data),
            _ => vec![BotAction::Pass],
        };

        self.send_action(&action);
    }

    fn send_command(&self, command: &str) -> std::io::Result<usize> {
        self.socket.send_to(command.as_bytes(), self.addr)
    }

    fn send_action(&self, actions: &[BotAction]) {
        let cmd_str = actions
            .iter()
            .map(|a| format!("{:?}", a))
            .collect::<Vec<_>>()
            .join("|");
        self.send_command(&cmd_str).unwrap();
    }

    fn handle_start_run(&self) -> Vec<BotAction> {
        vec![
            BotAction::StartRun,
            // Include stake, deck, seed, challenge parameters
        ]
    }
}

// Example bot implementation
struct ExampleBot;

impl BotBehavior for ExampleBot {
    fn skip_or_select_blind(&mut self, game_state: &GameData) -> Vec<BotAction> {
        if game_state.current_round.blind == "Small" || game_state.current_round.blind == "Big" {
            vec![BotAction::SkipBlind]
        } else {
            vec![BotAction::SelectBlind]
        }
    }

    fn select_cards_from_hand(&mut self, game_state: &GameData) -> Vec<BotAction> {
        // Implement card selection logic
        vec![BotAction::PlayHand]
    }

    // Implement other trait methods...
}

fn main() -> std::io::Result<()> {
    let example_bot = ExampleBot;
    let mut bot = Bot::new(example_bot, "Plasma Deck", 1, Some("1OGB5WO"), None, 12346)?;

    bot.start_balatro();
    bot.run();
    Ok(())
}
