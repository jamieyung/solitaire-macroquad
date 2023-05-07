use macroquad::prelude::*;

#[macroquad::main("Hello")]
async fn main() {
	// seed the RNG
	let duration_since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
	rand::srand(duration_since_epoch.as_secs());

	let game = Game::new();

    loop {
		clear_background(GREEN);

		if is_key_down(KeyCode::Q) {
			return;
		}

		draw_game(&game);

		next_frame().await;
	}
}

const CARD_W: f32 = 50.;
const CARD_H: f32 = 70.;
const PILE_CARD_OFFSET: f32 = 20.;

fn draw_game(game: &Game) {
	for (i, pile) in game.piles[..].into_iter().enumerate() {
		let x = 30. + i as f32 * CARD_W * 1.5;
		let y = 30.;
		draw_pile(pile, x, y);
	}
}

fn draw_pile(pile: &Pile, x:f32, y:f32) {
	for (i, card) in pile.hidden[..].into_iter().enumerate() {
		draw_card(card, x, y + i as f32 * PILE_CARD_OFFSET, false);
	}
	let n_hidden = pile.hidden.len() as f32;
	for (i, card) in pile.visible[..].into_iter().enumerate() {
		draw_card(card, x, y + (i as f32 + n_hidden) * PILE_CARD_OFFSET, true);
	}
}

fn draw_card(c: &Card, x:f32, y:f32, visible:bool) {
	draw_rectangle(x, y, CARD_W, CARD_H, WHITE);
	draw_rectangle_lines(x, y, CARD_W, CARD_H, 2., BLACK);

	if visible {
		let col = c.card_col();
		draw_text(c.card_rank_letter(), x, y + 17.0, 30.0, col);
		draw_text(c.card_suit_letter(), x+CARD_W*0.7, y + 17.0, 30.0, col);
	} else {
		draw_rectangle(x+3., y+3., CARD_W-6., CARD_H-6., BLUE);
	}
}

struct Game {
	piles: Vec<Pile>,
}

impl Game {
	pub fn new() -> Game {
		let mut game = Game {
			piles: Vec::new(),
		};

		let mut cards = Card::all_cards().to_vec();
		shuffle(&mut cards);

		print!("{:#?}", cards);

		let mut pile = Pile {
			hidden: Vec::new(),
			visible: Vec::new(),
		};

		pile.hidden.push(Card {
			suit: Suit::Diamonds,
			rank: Rank::Ace,
		});

		pile.hidden.push(Card {
			suit: Suit::Clubs,
			rank: Rank::Six,
		});

		pile.hidden.push(Card {
			suit: Suit::Hearts,
			rank: Rank::Ten,
		});

		pile.visible.push(Card {
			suit: Suit::Spades,
			rank: Rank::King,
		});

		game.piles.push(pile);

		return game;
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
			Rank::One => "1",
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

#[derive(Clone, Debug)]
enum Suit {
	Diamonds,
	Clubs,
	Hearts,
	Spades,
}

#[derive(Clone, Debug)]
enum Rank {
	Ace,
	One,
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
