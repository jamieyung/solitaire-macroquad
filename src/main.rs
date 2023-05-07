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
		} else if is_key_pressed(KeyCode::Space) {
			game.cycle_stock();
		} else if is_key_pressed(KeyCode::R) {
			game = Game::new();
		} else if is_key_pressed(KeyCode::D) {
			game.debug();
		}

		draw_game(&game);

		let (mx, my) = mouse_position();
		if let Some(target) = game.mouse_hit(mx, my) {
			draw_mouse_hit(target);
			if is_mouse_button_pressed(MouseButton::Left) {
				println!("target: {:?}", target);
				if let Some(_) = game.move_in_progress {
					game.exec_move_in_progress(target);
				} else {
					game.move_in_progress = None;
					if let MouseTarget::StockDeck = target {
						game.cycle_stock();
					} else if let Some(moves) = game.calc_moves(target) {
						println!("moves: {:?}", moves);
						if moves.len() == 1 {
							game.exec_move(target, moves.first().unwrap().clone());
						} else {
							game.move_in_progress = Some(MoveInProgress{ target, moves });
						}
					} else {
						println!("No moves");
					}
				}
			}
		}

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
const MOUSE_TARGET_COLOUR: Color = Color::new(1.00, 0.00, 1.00, 0.2);

fn draw_mouse_hit(target: MouseTarget) {
	match target {
		MouseTarget::StockTop => {
			draw_rectangle(INSET + PILE_H_OFFSET, INSET, CARD_W, CARD_H, MOUSE_TARGET_COLOUR);
		}
		MouseTarget::StockDeck => {
			draw_rectangle(INSET, INSET, CARD_W, CARD_H, MOUSE_TARGET_COLOUR);
		}
		MouseTarget::Foundation(suit) => {
			draw_rectangle(FOUNDATIONS_X+suit.foundation_offset()*PILE_H_OFFSET, INSET, CARD_W, CARD_H, MOUSE_TARGET_COLOUR);
		}
		MouseTarget::Pile{pile_index, n_cards, top, ..} => {
			let x = Pile::pile_x(pile_index);
			let y = top;
			let w = CARD_W;
			let h = CARD_H + PILE_CARD_V_OFFSET*((n_cards-1) as f32);
			draw_rectangle(x, y, w, h, MOUSE_TARGET_COLOUR);
		}
	}
}

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
		let x = Pile::pile_x(i);
		draw_pile(pile, x, PILES_Y);
	}

	// draw foundations
	for suit in Suit::all() {
		draw_foundation(suit.clone(), game.foundation_fill_levels.get(&suit), FOUNDATIONS_X+suit.foundation_offset()*PILE_H_OFFSET, INSET);
	}

	// draw move_in_progress
	if let Some(mip) = &game.move_in_progress {
		draw_mouse_hit(mip.target);
	}
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
		let col = c.col();
		draw_text(c.rank_letter(), x, y + CARD_FONT_SIZE*0.55, CARD_FONT_SIZE, col);
		draw_text(c.suit_letter(), x+CARD_W*0.7, y + CARD_FONT_SIZE*0.55, CARD_FONT_SIZE, col);
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
	move_in_progress: Option<MoveInProgress>,
}

impl Game {
	pub fn new() -> Game {
		let mut cards = Card::all_cards().to_vec();
		shuffle(&mut cards);

		let mut game = Game {
			stock: VecDeque::from(cards),
			piles: Vec::new(),
			foundation_fill_levels: HashMap::new(),
			move_in_progress: None,
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

	pub fn mouse_hit(&self, mx:f32, my:f32) -> Option<MouseTarget> {
		// check stock
		if !self.stock.is_empty() && Card::mouse_hit(INSET + PILE_H_OFFSET, INSET, mx, my) {
			return Some(MouseTarget::StockTop)
		}
		if !self.stock.len() > 1 && Card::mouse_hit(INSET, INSET, mx, my) {
			return Some(MouseTarget::StockDeck)
		}

		// check foundations
		for suit in Suit::all() {
			if self.foundation_fill_levels.contains_key(&suit) && Card::mouse_hit(FOUNDATIONS_X+suit.foundation_offset()*PILE_H_OFFSET, INSET, mx, my) {
				return Some(MouseTarget::Foundation(suit.clone()))
			}
		}

		// check piles
		for (pile_index, pile) in self.piles[..].into_iter().enumerate() {
			let x = Pile::pile_x(pile_index);
			if mx < x || mx > x+CARD_W {continue}

			// check visible cards in reverse order
			let n_hidden = pile.hidden.len() as f32;
			for (card_index, card) in pile.visible[..].into_iter().enumerate().rev() {
				let y = PILES_Y + (card_index as f32 + n_hidden) * PILE_CARD_V_OFFSET;
				if Card::mouse_hit(x, y, mx, my) {
					return Some(MouseTarget::Pile{
						pile_index,
						n_cards: (pile.visible.len() - card_index) as u8,
						target_card: *card,
						target_card_index: card_index,
						top: y,
					})
				}
			}
		}

		return None
	}

	pub fn foundation_top_card(&self, suit:Suit) -> Option<Card> {
		let rank = self.foundation_fill_levels.get(&suit)?;
		Some(Card::new(suit, *rank))
	}

	pub fn calc_moves(&mut self, target:MouseTarget) -> Option<Vec<Move>> {
		let mut moves: Vec<Move> = Vec::new();

		match target {
			MouseTarget::StockTop => {
				let card = self.stock.front()?;

				if card.rank == Rank::Ace {
					moves.push(Move::ToFoundation(card.suit))
				}

				// consider moves to the piles
				for (i, pile) in self.piles[..].into_iter().enumerate() {
					if pile.is_empty() && card.rank == Rank::King {
						// if the pile is empty and the card is a king, it's a valid move
						moves.push(Move::ToPile(i));
					}

					// if the card can go onto the top visible card, it's a valid move
					else {
						if let Some(top) = pile.top_card() {
							if card.can_pile_onto(top) {
								moves.push(Move::ToPile(i));
							}
						}
					}
				}

				// consider moves to the foundation
				for suit in Suit::all() {
					if let Some(top) = self.foundation_top_card(*suit) {
						if card.can_stack_onto_in_foundation(top) {
							moves.push(Move::ToFoundation(*suit))
						}
					}
				}
			}
			MouseTarget::StockDeck => {} // impossible
			MouseTarget::Foundation(suit) => {
				let card = self.foundation_top_card(suit)?;

				// consider moves to the piles
				for (i, pile) in self.piles[..].into_iter().enumerate() {
					// if the card can go onto the top visible card, it's a valid move
					if let Some(top) = pile.top_card() {
						if card.can_pile_onto(top) {
							moves.push(Move::ToPile(i));
						}
					}
				}
			}
			MouseTarget::Pile{pile_index, target_card:card, n_cards, ..} => {
				if card.rank == Rank::Ace {
					moves.push(Move::ToFoundation(card.suit))
				}

				// consider moves to other piles
				for (i, pile) in self.piles[..].into_iter().enumerate() {
					if i == pile_index { continue }

					if pile.is_empty() && card.rank == Rank::King {
						// if the pile is empty and the card is a king, it's a valid move
						moves.push(Move::ToPile(i));
					}

					// if the card can go onto the top visible card, it's a valid move
					else {
						if let Some(top) = pile.top_card() {
							if card.can_pile_onto(top) {
								moves.push(Move::ToPile(i));
							}
						}
					}
				}

				// consider moves to the foundation (iff it's a single card being targeted)
				if n_cards == 1 {
					for suit in Suit::all() {
						if let Some(top) = self.foundation_top_card(*suit) {
							if card.can_stack_onto_in_foundation(top) {
								moves.push(Move::ToFoundation(*suit))
							}
						}
					}
				}
			}
		}

		if moves.is_empty() {
			None
		} else {
			Some(moves)
		}
	}

	pub fn exec_move(&mut self, target:MouseTarget, mv:Move) -> bool {
		match target {
			MouseTarget::StockTop => {
				if let Some(card) = self.stock.front().cloned() {
					self.stock.pop_front();
					match mv {
						Move::ToPile(pile_index) => {
							self.piles[pile_index].visible.push(card);
						}
						Move::ToFoundation(suit) => {
							self.foundation_fill_levels.insert(suit, card.rank);
						}
					}
					return true
				}
			}
			MouseTarget::StockDeck => {} // impossible
			MouseTarget::Foundation(suit) => {
				match mv {
					Move::ToPile(pile_index) => {
						// TODO
					}
					Move::ToFoundation(_) => {} // impossible
				}
			}
			MouseTarget::Pile{pile_index, target_card:top_card, target_card_index, ..} => {
				let pile = &mut self.piles[pile_index];
				let removed:Vec<Card> = pile.visible.drain(target_card_index..).collect();
				if pile.visible.is_empty() {
					if let Some(next) = pile.hidden.pop() {
						pile.visible.push(next);
					}
				}
				match mv {
					Move::ToPile(dest_pile_index) => {
						for card in removed {
							self.piles[dest_pile_index].visible.push(card);
						}
					}
					Move::ToFoundation(suit) => {
						self.foundation_fill_levels.insert(suit, top_card.rank);
					}
				}
				return true
			}
		}
		return false
	}

	pub fn exec_move_in_progress(&mut self, target:MouseTarget) {
		if let Some(mip) = &self.move_in_progress {
			for mv in mip.moves.to_owned() {
				match mv {
					Move::ToPile(mip_pile_index) => {
						match target {
							MouseTarget::StockTop => {} // impossible
							MouseTarget::StockDeck => {} // impossible
							MouseTarget::Foundation(_) => {} // not relevant for this move in progress
							MouseTarget::Pile{pile_index:target_pile_index, ..} => {
								if mip_pile_index == target_pile_index {
									self.exec_move(mip.target, mv);
									self.move_in_progress = None;
									return
								}
							}
						}
					}
					Move::ToFoundation(mip_suit) => {
						match target {
							MouseTarget::StockTop => {} // impossible
							MouseTarget::StockDeck => {} // impossible
							MouseTarget::Foundation(target_suit) => {
								if mip_suit == target_suit {
									self.exec_move(mip.target, mv);
									self.move_in_progress = None;
									return
								}
							}
							MouseTarget::Pile{..} => {} // not relevant for this move in progress
						}
					}
				}
			}
		}
	}

	pub fn debug(&self) {
		// TODO change this into a succinct Display, eg. print cards out as 2 chars
		println!("stock: {:?}", self.stock);
		println!("stock top: {:?}", self.stock.front());
		println!("piles: {:?}", self.piles);
		println!("foundations: {:?}", self.foundation_fill_levels);
		println!("move_in_progress: {:?}", self.move_in_progress);
	}
}

#[derive(Copy, Clone, Debug)]
enum MouseTarget {
	StockTop, // the visible card
	StockDeck, // the rest of the stock
	Foundation(Suit),
	Pile{
		pile_index:usize, // 0 is the leftmost pile
		n_cards:u8, // 1 = only the top card, 2 = two top cards, etc
		target_card:Card, // the card that was targeted
		target_card_index:usize, // the index into visible of the targeted card
		top:f32, // the y-coord of the top of the targeted card
	},
}

#[derive(Clone, Debug)]
struct MoveInProgress {
	target: MouseTarget,
	moves: Vec<Move>,
}

#[derive(Clone, Debug)]
enum Move {
	ToPile(usize),
	ToFoundation(Suit),
}

#[derive(Clone, Debug)]
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

	// x-coord of left edge of the pile
	pub fn pile_x(pile_index:usize) -> f32 {
		return INSET + pile_index as f32 * PILE_H_OFFSET;
	}

	pub fn is_empty(&self) -> bool {
		self.hidden.is_empty() && self.visible.is_empty()
	}

	pub fn top_card(&self) -> Option<Card> {
		self.visible.as_slice().last().map(|card| *card)
	}
}

#[derive(Clone, Copy, Debug)]
struct Card {
	suit: Suit,
	rank: Rank,
}

impl Card {
	pub fn new(suit: Suit, rank: Rank) -> Card {
		return Card{suit, rank}
	}

	pub fn col(&self) -> Color {
		return match self.suit {
			Suit::Diamonds | Suit::Hearts => RED,
			Suit::Clubs | Suit::Spades => BLACK,
		}
	}

	pub fn suit_letter(&self) -> &str {
		return match self.suit {
			Suit::Diamonds => "D",
			Suit::Clubs => "C",
			Suit::Hearts => "H",
			Suit::Spades => "S",
		}
	}

	pub fn rank_letter(&self) -> &str {
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

	pub fn rank_index(&self) -> i8 {
		return match self.rank {
			Rank::Ace => 0,
			Rank::Two => 1,
			Rank::Three => 2,
			Rank::Four => 3,
			Rank::Five => 4,
			Rank::Six => 5,
			Rank::Seven => 6,
			Rank::Eight => 7,
			Rank::Nine => 8,
			Rank::Ten => 9,
			Rank::Jack => 10,
			Rank::Queen => 11,
			Rank::King => 12,
		}
	}

	// Returns an array slice containing all the cards in a standard 52-card deck
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

	pub fn mouse_hit(cx:f32, cy:f32, mx:f32, my:f32) -> bool {
		return mx >= cx && mx <= cx+CARD_W
			&& my >= cy && my <= cy+CARD_H
	}

	// returns true if self can stack on top of other in a pile, eg. if self is 2D and other is 3S.
	pub fn can_pile_onto(&self, other:Card) -> bool {
		self.col() != other.col() && other.rank_index() - self.rank_index() == 1
	}

	// returns true if self can stack on top of other in a foundation stack, eg. if self is 2D and other is AD.
	pub fn can_stack_onto_in_foundation(&self, other:Card) -> bool {
		self.col() == other.col() && self.rank_index() - other.rank_index() == 1
	}
}

fn shuffle(cards: &mut[Card]) {
	let l = cards.len();
	for n in 0..l {
		let i = rand::gen_range(0, l - n);
		cards.swap(i, l - n - 1);
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
enum Suit {
	Diamonds,
	Clubs,
	Hearts,
	Spades,
}

impl Suit {
	pub fn all() -> &'static [Suit] {
		static SUITS: [Suit; 4] = [
			Suit::Diamonds,
			Suit::Clubs,
			Suit::Hearts,
			Suit::Spades,
		];
		&SUITS
	}

	pub fn foundation_offset(&self) -> f32 {
		return match self {
			Suit::Diamonds => 0.,
			Suit::Clubs => 1.,
			Suit::Hearts => 2.,
			Suit::Spades => 3.
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
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
