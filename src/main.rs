use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

#[macroquad::main("Solitaire")]
async fn main() {
	// seed the RNG
	let duration_since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
	rand::srand(duration_since_epoch.as_secs());

	let mut game = Game::new();

    loop {
		clear_background(GREEN);

		if is_key_down(KeyCode::Escape) {
			return;
		} else if is_key_pressed(KeyCode::Q) {
			game.cycle_stock();
		}

		draw_game(&game);
		next_frame().await;
	}
}

const N_PILES: u8 = 7; // number of piles
const INSET: f32 = 30.; // distance from edge of screen to the cards
const FOUNDATIONS_X: f32 = INSET+3.*PILE_H_OFFSET; // the leftmost x-coord of the foundation piles
const CARD_W: f32 = 60.; // card width
const CARD_H: f32 = CARD_W*1.4; // card height
const CARD_BORDER_WIDTH: f32 = 2.; // width of the black border around the cards
const CARD_FONT_SIZE: f32 = CARD_W*0.6; // card font size
const CARD_BACK_WHITE_BORDER_MARGIN: f32 = 3.; // size of the white strip around the card back colour
const CARD_BACK_COLOUR: Color = BLUE; // colour on the back of the cards
const PILES_Y: f32 = CARD_H * 2.; // the topmost y-coord of the piles area
const PILE_CARD_V_OFFSET: f32 = CARD_FONT_SIZE*0.6; // vertical distance between cards in a pile
const PILE_H_OFFSET: f32 = CARD_W * 1.5; // horizontal distance between the left edge of adjacent piles

fn draw_game(game: &Game) {
	// draw stock
	if game.stock.len() > 1 {
		draw_card(&Card::new(Suit::Diamonds, Rank::Ace), INSET, INSET, false);
	}
	if !game.stock.is_empty() {
		draw_card(&game.stock[0], INSET + PILE_H_OFFSET, INSET, true);
	}

	// draw piles
	for (i, pile) in game.piles[..].into_iter().enumerate() {
		let x = INSET + i as f32 * PILE_H_OFFSET;
		let y = PILES_Y;
		draw_pile(pile, x, y);
	}

	// draw foundations
	draw_foundation(Suit::Diamonds, game.foundation_fill_levels.get(&Suit::Diamonds), FOUNDATIONS_X, INSET);
	draw_foundation(Suit::Clubs, game.foundation_fill_levels.get(&Suit::Clubs), FOUNDATIONS_X+1.*PILE_H_OFFSET, INSET);
	draw_foundation(Suit::Hearts, game.foundation_fill_levels.get(&Suit::Hearts), FOUNDATIONS_X+2.*PILE_H_OFFSET, INSET);
	draw_foundation(Suit::Spades, game.foundation_fill_levels.get(&Suit::Spades), FOUNDATIONS_X+3.*PILE_H_OFFSET, INSET);
}

fn draw_foundation(suit: Suit, rank: Option<&Rank>, x:f32, y:f32) {
	match rank {
		Some(r) => {
			draw_card(&Card::new(suit, r.to_owned()), x, y, true);
		}
		None => {
			draw_rectangle_lines(x, y, CARD_W, CARD_H, CARD_BORDER_WIDTH, BLACK);
		}
	}
}

fn draw_pile(pile: &Pile, x:f32, y:f32) {
	for (i, card) in pile.hidden[..].into_iter().enumerate() {
		draw_card(card, x, y + i as f32 * PILE_CARD_V_OFFSET, false);
	}
	let n_hidden = pile.hidden.len() as f32;
	for (i, card) in pile.visible[..].into_iter().enumerate() {
		draw_card(card, x, y + (i as f32 + n_hidden) * PILE_CARD_V_OFFSET, true);
	}
}

fn draw_card(c: &Card, x:f32, y:f32, visible:bool) {
	draw_rectangle(x, y, CARD_W, CARD_H, WHITE);
	draw_rectangle_lines(x, y, CARD_W, CARD_H, CARD_BORDER_WIDTH, BLACK);

	if visible {
		let col = c.card_col();
		draw_text(c.card_rank_letter(), x, y + CARD_FONT_SIZE*0.55, CARD_FONT_SIZE, col);
		draw_text(c.card_suit_letter(), x+CARD_W*0.7, y + CARD_FONT_SIZE*0.55, CARD_FONT_SIZE, col);
	} else {
		draw_rectangle(
			x+CARD_BACK_WHITE_BORDER_MARGIN, y+CARD_BACK_WHITE_BORDER_MARGIN,
			CARD_W-2.*CARD_BACK_WHITE_BORDER_MARGIN, CARD_H-2.*CARD_BACK_WHITE_BORDER_MARGIN, CARD_BACK_COLOUR);
	}
}

struct Game {
	stock: VecDeque<Card>,
	piles: Vec<Pile>,
	foundation_fill_levels: HashMap<Suit, Rank>,
}

impl Game {
	pub fn new() -> Game {
		let mut cards = Card::all_cards().to_vec();
		shuffle(&mut cards);

		let mut game = Game {
			stock: VecDeque::from(cards),
			piles: Vec::new(),
			foundation_fill_levels: HashMap::new(),
		};

		for pile_size in 1..=N_PILES {
			let mut pile = Pile::new();
			for i in 0..pile_size {
				let card = game.stock.pop_front().unwrap();
				if i == pile_size - 1 {
					pile.visible.push(card);
				} else {
					pile.hidden.push(card);
				}
			}
			game.piles.push(pile);
		}

		return game;
	}

	// does nothing if the stock has fewer than 2 cards
	pub fn cycle_stock(&mut self) {
		if self.stock.len() < 2 {
			return
		}
		let card = self.stock.pop_front().unwrap();
		self.stock.push_back(card);
	}
}

struct Pile {
	hidden: Vec<Card>,
	visible: Vec<Card>,
}

impl Pile {
	pub fn new() -> Pile {
		return Pile {
			hidden: Vec::new(),
			visible: Vec::new(),
		};
	}
}

#[derive(Clone, Debug)]
struct Card {
	suit: Suit,
	rank: Rank,
}

impl Card {
	pub fn new(suit: Suit, rank: Rank) -> Card {
		return Card{suit, rank}
	}

	pub fn card_col(&self) -> Color {
		return match self.suit {
			Suit::Diamonds | Suit::Hearts => RED,
			Suit::Clubs | Suit::Spades => BLACK,
		}
	}

	pub fn card_suit_letter(&self) -> &str {
		return match self.suit {
			Suit::Diamonds => "D",
			Suit::Clubs => "C",
			Suit::Hearts => "H",
			Suit::Spades => "S",
		}
	}

	pub fn card_rank_letter(&self) -> &str {
		return match self.rank {
			Rank::Ace => "A",
			Rank::Two => "2",
			Rank::Three => "3",
			Rank::Four => "4",
			Rank::Five => "5",
			Rank::Six => "6",
			Rank::Seven => "7",
			Rank::Eight => "8",
			Rank::Nine => "9",
			Rank::Ten => "10",
			Rank::Jack => "J",
			Rank::Queen => "Q",
			Rank::King => "K",
		}
	}
 
	/// Returns an array slice containing all the cards in a standard 52-card deck
    pub fn all_cards() -> &'static [Card] {
        static CARDS: [Card; 52] = [
            Card { suit: Suit::Spades, rank: Rank::Two },
            Card { suit: Suit::Spades, rank: Rank::Three },
            Card { suit: Suit::Spades, rank: Rank::Four },
            Card { suit: Suit::Spades, rank: Rank::Five },
            Card { suit: Suit::Spades, rank: Rank::Six },
            Card { suit: Suit::Spades, rank: Rank::Seven },
            Card { suit: Suit::Spades, rank: Rank::Eight },
            Card { suit: Suit::Spades, rank: Rank::Nine },
            Card { suit: Suit::Spades, rank: Rank::Ten },
            Card { suit: Suit::Spades, rank: Rank::Jack },
            Card { suit: Suit::Spades, rank: Rank::Queen },
            Card { suit: Suit::Spades, rank: Rank::King },
            Card { suit: Suit::Spades, rank: Rank::Ace },
            Card { suit: Suit::Hearts, rank: Rank::Two },
            Card { suit: Suit::Hearts, rank: Rank::Three },
            Card { suit: Suit::Hearts, rank: Rank::Four },
            Card { suit: Suit::Hearts, rank: Rank::Five },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Eight },
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Ten },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
            Card { suit: Suit::Hearts, rank: Rank::Queen },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::Ace },
            Card { suit: Suit::Diamonds, rank: Rank::Two },
            Card { suit: Suit::Diamonds, rank: Rank::Three },
            Card { suit: Suit::Diamonds, rank: Rank::Four },
            Card { suit: Suit::Diamonds, rank: Rank::Five },
            Card { suit: Suit::Diamonds, rank: Rank::Six },
            Card { suit: Suit::Diamonds, rank: Rank::Seven },
            Card { suit: Suit::Diamonds, rank: Rank::Eight },
            Card { suit: Suit::Diamonds, rank: Rank::Nine },
            Card { suit: Suit::Diamonds, rank: Rank::Ten },
            Card { suit: Suit::Diamonds, rank: Rank::Jack },
            Card { suit: Suit::Diamonds, rank: Rank::Queen },
            Card { suit: Suit::Diamonds, rank: Rank::King },
            Card { suit: Suit::Diamonds, rank: Rank::Ace },
            Card { suit: Suit::Clubs, rank: Rank::Two },
            Card { suit: Suit::Clubs, rank: Rank::Three },
            Card { suit: Suit::Clubs, rank: Rank::Four },
            Card { suit: Suit::Clubs, rank: Rank::Five },
            Card { suit: Suit::Clubs, rank: Rank::Six },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Eight },
            Card { suit: Suit::Clubs, rank: Rank::Nine },
            Card { suit: Suit::Clubs, rank: Rank::Ten },
            Card { suit: Suit::Clubs, rank: Rank::Jack },
            Card { suit: Suit::Clubs, rank: Rank::Queen },
            Card { suit: Suit::Clubs, rank: Rank::King },
            Card { suit: Suit::Clubs, rank: Rank::Ace }
        ];
        &CARDS
    }
}

fn shuffle(cards: &mut[Card]) {
	let l = cards.len();
	for n in 0..l {
		let i = rand::gen_range(0, l - n);
		cards.swap(i, l - n - 1);
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Suit {
	Diamonds,
	Clubs,
	Hearts,
	Spades,
}

#[derive(Clone, Debug)]
enum Rank {
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
