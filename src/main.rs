use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

#[macroquad::main("Solitaire")]
async fn main() {
	let mut textures = HashMap::new();
	textures.insert(Suit::Diamonds, load_texture("textures/diamonds.png").await.unwrap());
	textures.insert(Suit::Clubs, load_texture("textures/clubs.png").await.unwrap());
	textures.insert(Suit::Hearts, load_texture("textures/hearts.png").await.unwrap());
	textures.insert(Suit::Spades, load_texture("textures/spades.png").await.unwrap());

	request_new_screen_size(SCREEN_W, SCREEN_H);

	// seed the RNG
	let duration_since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
	rand::srand(duration_since_epoch.as_secs());

	let mut game = Game::new();

	loop {
		clear_background(GREEN);

		if is_key_down(KeyCode::Escape) {
			return;
		} else if is_key_pressed(KeyCode::Space) {
			game.exec_move(Move::CycleStock);
		} else if is_key_pressed(KeyCode::R) {
			game = Game::new();
		} else if is_key_pressed(KeyCode::D) {
			game.debug();
		} else if is_key_pressed(KeyCode::U) {
			game.undo_move();
		} else if is_key_pressed(KeyCode::A) {
			game.auto_move();
		}

		draw_game(&game, &textures);

		let (mx, my) = mouse_position();
		if let Some(target) = game.mouse_hit(mx, my) {
			draw_mouse_hit(target, MOUSE_TARGET_COLOUR);
			if is_mouse_button_pressed(MouseButton::Left) {
				println!("target: {:?}", target);
				if Option::is_some(&game.move_in_progress) {
					game.exec_move_in_progress(target);
				} else {
					game.move_in_progress = None;
					if let Some(moves) = game.calc_moves(target) {
						println!("moves: {:?}", moves);
						if moves.len() == 1 {
							game.exec_move(moves.first().unwrap().clone());
						} else {
							game.move_in_progress = Some(MoveInProgress{ target, moves });
						}
					} else {
						println!("No moves");
					}
				}
			}
		}

		// TODO detect win condition

		next_frame().await;
	}
}

const SCREEN_H: f32 = 500.;
const SCREEN_W: f32 = SCREEN_H*1.2;
const N_PILES: u8 = 7; // number of piles
const INSET: f32 = 30.; // distance from edge of screen to the cards
const FOUNDATIONS_X: f32 = INSET+3.*PILE_H_OFFSET; // the leftmost x-coord of the foundation piles
const CARD_W: f32 = SCREEN_H*0.2; // card width
const CARD_H: f32 = CARD_W*1.4; // card height
const CARD_BORDER_WIDTH: f32 = 2.; // width of the black border around the cards
const CARD_FONT_SIZE: f32 = CARD_W*0.6; // card font size
const CARD_SUIT_TEX_SIZE_FOUNDATION: f32 = CARD_W*0.5;
const CARD_SUIT_TEX_SIZE_PILES: f32 = CARD_W*0.3;
const CARD_BACK_WHITE_BORDER_MARGIN: f32 = 3.; // size of the white strip around the card back colour
const CARD_BACK_COLOUR: Color = BLUE; // colour on the back of the cards
const PILES_Y: f32 = CARD_H * 1.5; // the topmost y-coord of the piles area
const PILE_CARD_V_OFFSET: f32 = CARD_FONT_SIZE*0.6; // vertical distance between cards in a pile
const PILE_H_OFFSET: f32 = CARD_W * 1.5; // horizontal distance between the left edge of adjacent piles
const MOUSE_TARGET_COLOUR: Color = Color::new(1.00, 0.00, 1.00, 0.1);
const MOVE_IN_PROGRESS_COLOUR: Color = Color::new(0.00, 1.00, 1.00, 0.5);

fn draw_mouse_hit(target: MouseTarget, col:Color) {
	match target {
		MouseTarget::StockTop => {
			draw_rectangle(INSET + PILE_H_OFFSET, INSET, CARD_W, CARD_H, col);
		}
		MouseTarget::StockDeck => {
			draw_rectangle(INSET, INSET, CARD_W, CARD_H, col);
		}
		MouseTarget::Foundation(suit) => {
			draw_rectangle(FOUNDATIONS_X+suit.foundation_offset()*PILE_H_OFFSET, INSET, CARD_W, CARD_H, col);
		}
		MouseTarget::EmptyPile(pile_index) => {
			let x = Pile::pile_x(pile_index);
			let y = PILES_Y;
			let w = CARD_W;
			let h = CARD_H;
			draw_rectangle(x, y, w, h, col);
		}
		MouseTarget::PileCard{pile_index, n_cards, top, ..} => {
			let x = Pile::pile_x(pile_index);
			let y = top;
			let w = CARD_W;
			let h = CARD_H + PILE_CARD_V_OFFSET*((n_cards-1) as f32);
			draw_rectangle(x, y, w, h, col);
		}
	}
}

fn draw_game(game: &Game, textures:&HashMap<Suit, Texture2D>) {
	// draw stock
	if game.stock.len() > 1 {
		draw_card(&Card::new(Suit::Diamonds, Rank::Ace), INSET, INSET, false, textures);
	}
	if !game.stock.is_empty() {
		draw_card(&game.stock[0], INSET + PILE_H_OFFSET, INSET, true, textures);
	}

	// draw piles
	for (i, pile) in game.piles[..].into_iter().enumerate() {
		let x = Pile::pile_x(i);
		draw_pile(pile, x, PILES_Y, textures);
	}

	// draw foundations
	for suit in Suit::all() {
		draw_foundation(suit.clone(), game.foundation_fill_levels.get(&suit), FOUNDATIONS_X+suit.foundation_offset()*PILE_H_OFFSET, INSET, textures);
	}

	// draw move_in_progress
	if let Some(mip) = &game.move_in_progress {
		draw_mouse_hit(mip.target, MOVE_IN_PROGRESS_COLOUR);
	}
}

fn draw_foundation(suit: Suit, rank: Option<&Rank>, x:f32, y:f32, textures:&HashMap<Suit, Texture2D>) {
	match rank {
		Some(r) => {
			draw_card(&Card::new(suit, r.to_owned()), x, y, true, textures);
		}
		None => {
			draw_rectangle_lines(x, y, CARD_W, CARD_H, CARD_BORDER_WIDTH, BLACK);
			let tex = *textures.get(&suit).unwrap();
			let col = Color::new(0., 0., 0., 0.7);
			draw_texture_ex(tex, x+CARD_W*0.5-CARD_SUIT_TEX_SIZE_FOUNDATION*0.5, y+CARD_H*0.5-CARD_SUIT_TEX_SIZE_FOUNDATION*0.63, col, DrawTextureParams{
				dest_size: Some(vec2(CARD_SUIT_TEX_SIZE_FOUNDATION, CARD_SUIT_TEX_SIZE_FOUNDATION)),
				..Default::default()
			});
		}
	}
}

fn draw_pile(pile: &Pile, x:f32, y:f32, textures:&HashMap<Suit, Texture2D>) {
	if pile.is_empty() {
		draw_rectangle_lines(x, y, CARD_W, CARD_H, CARD_BORDER_WIDTH, BLACK);
		return
	}

	for (i, card) in pile.hidden[..].into_iter().enumerate() {
		draw_card(card, x, y + i as f32 * PILE_CARD_V_OFFSET, false, textures);
	}
	let n_hidden = pile.hidden.len() as f32;
	for (i, card) in pile.visible[..].into_iter().enumerate() {
		draw_card(card, x, y + (i as f32 + n_hidden) * PILE_CARD_V_OFFSET, true, textures);
	}
}

fn draw_card(c: &Card, x:f32, y:f32, visible:bool, textures:&HashMap<Suit, Texture2D>) {
	draw_rectangle(x, y, CARD_W, CARD_H, WHITE);
	draw_rectangle_lines(x, y, CARD_W, CARD_H, CARD_BORDER_WIDTH, BLACK);

	if visible {
		let col = c.col();
		draw_text(c.rank.letter(), x, y + CARD_FONT_SIZE*0.55, CARD_FONT_SIZE, col);
		draw_texture_ex(*textures.get(&c.suit).unwrap(), x+CARD_W*0.63, y+CARD_H*0.02, col, DrawTextureParams{
			dest_size: Some(vec2(CARD_SUIT_TEX_SIZE_PILES, CARD_SUIT_TEX_SIZE_PILES)),
			..Default::default()
		});
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
	move_history: Vec<Move>,
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
			move_history: Vec::new(),
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
			if Card::mouse_hit(FOUNDATIONS_X+suit.foundation_offset()*PILE_H_OFFSET, INSET, mx, my) {
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
					return Some(MouseTarget::PileCard{
						pile_index,
						n_cards: (pile.visible.len() - card_index) as u8,
						target_card: *card,
						target_card_index: card_index,
						top: y,
					})
				}
			}

			if pile.is_empty() {
				return Some(MouseTarget::EmptyPile(pile_index))
			}
		}

		return None
	}

	pub fn foundation_top_card(&self, suit:Suit) -> Option<Card> {
		let rank = self.foundation_fill_levels.get(&suit)?;
		Some(Card::new(suit, *rank))
	}

	pub fn calc_moves(&self, target:MouseTarget) -> Option<Vec<Move>> {
		let mut moves: Vec<Move> = Vec::new();

		match target {
			MouseTarget::StockTop => {
				let card = self.stock.front()?;

				if card.rank == Rank::Ace {
					moves.push(Move::CardMove{
						card: *card,
						src: MoveSrc::FromStock,
						dest: MoveDest::ToFoundation(card.suit),
					})
				}

				// consider moves to the foundation
				for suit in Suit::all() {
					if let Some(top) = self.foundation_top_card(*suit) {
						if card.can_stack_onto_in_foundation(top) {
							moves.push(Move::CardMove{
								card: *card,
								src: MoveSrc::FromStock,
								dest: MoveDest::ToFoundation(card.suit),
							});
							break;
						}
					}
				}

				// consider moves to the piles
				for (i, pile) in self.piles[..].into_iter().enumerate() {
					// if the card can go onto the top visible card, it's a valid move
					if let Some(top) = pile.top_card() {
						if card.can_pile_onto(top) {
							moves.push(Move::CardMove{
								card: *card,
								src: MoveSrc::FromStock,
								dest: MoveDest::ToPile(i),
							});
						}
					}

					else if pile.is_empty() && card.rank == Rank::King {
						// if the pile is empty and the card is a king, it's a valid move
						moves.push(Move::CardMove{
							card: *card,
							src: MoveSrc::FromStock,
							dest: MoveDest::ToPile(i),
						});
					}
				}
			}
			MouseTarget::StockDeck => {
				if self.stock.len() > 1 {
					moves.push(Move::CycleStock);
				}
			}
			MouseTarget::Foundation(suit) => {
				let card = self.foundation_top_card(suit)?;

				// consider moves to the piles
				for (i, pile) in self.piles[..].into_iter().enumerate() {
					// if the card can go onto the top visible card, it's a valid move
					if let Some(top) = pile.top_card() {
						if card.can_pile_onto(top) {
							moves.push(Move::CardMove{
								card,
								src: MoveSrc::FromFoundation(suit),
								dest: MoveDest::ToPile(i),
							});
						}
					}
				}
			}
			MouseTarget::EmptyPile(_) => {} // impossible
			MouseTarget::PileCard{pile_index, target_card:card, target_card_index, n_cards, ..} => {
				if card.rank == Rank::Ace {
					moves.push(Move::CardMove{
						card,
						src: MoveSrc::FromPile{ pile_index, n_cards, target_card_index },
						dest: MoveDest::ToFoundation(card.suit),
					});
				}

				// consider moves to the foundation (iff it's a single card being targeted)
				if n_cards == 1 {
					for suit in Suit::all() {
						if let Some(top) = self.foundation_top_card(*suit) {
							if card.can_stack_onto_in_foundation(top) {
								moves.push(Move::CardMove{
									card,
									src: MoveSrc::FromPile{ pile_index, n_cards, target_card_index },
									dest: MoveDest::ToFoundation(*suit),
								});
								break;
							}
						}
					}
				}

				// consider moves to other piles
				for (i, pile) in self.piles[..].into_iter().enumerate() {
					if i == pile_index { continue }

					// if the card can go onto the top visible card, it's a valid move
					if let Some(top) = pile.top_card() {
						if card.can_pile_onto(top) {
							moves.push(Move::CardMove{
								card,
								src: MoveSrc::FromPile{ pile_index, n_cards, target_card_index },
								dest: MoveDest::ToPile(i),
							});
						}
					}

					else if pile.is_empty() && card.rank == Rank::King {
						// if the pile is empty and the card is a king, it's a valid move
						moves.push(Move::CardMove{
							card,
							src: MoveSrc::FromPile{ pile_index, n_cards, target_card_index },
							dest: MoveDest::ToPile(i),
						});
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

	pub fn exec_move(&mut self, mv:Move) -> bool {
		match mv {
			Move::CardMove{ card, src, dest } => {
				match src {
					MoveSrc::FromStock => {
						if let Some(card) = self.stock.front().cloned() {
							self.stock.pop_front();
							match dest {
								MoveDest::ToPile(pile_index) => {
									self.piles[pile_index].visible.push(card);
								}
								MoveDest::ToFoundation(suit) => {
									self.foundation_fill_levels.insert(suit, card.rank);
								}
							}
							// if there is a previous card, put it back at the front
							if !self.stock.is_empty() {
								if let Some(prev) = self.stock.pop_back() {
									self.stock.push_front(prev)
								}
							}
							self.move_history.push(mv);
							return true
						}
					}
					MoveSrc::FromFoundation(suit) => {
						match dest {
							MoveDest::ToPile(pile_index) => {
								if let Some(top_card) = self.pop_foundation(suit) {
									self.piles[pile_index].visible.push(top_card);
									self.move_history.push(mv);
									return true
								}
							}
							MoveDest::ToFoundation(_) => {} // impossible
						}
					}
					MoveSrc::FromPile{ pile_index, target_card_index, .. } => {
						let pile = &mut self.piles[pile_index];
						let removed:Vec<Card> = pile.visible.drain(target_card_index..).collect();
						if pile.visible.is_empty() {
							if let Some(next) = pile.hidden.pop() {
								pile.visible.push(next);
							}
						}
						match dest {
							MoveDest::ToPile(dest_pile_index) => {
								for card in removed {
									self.piles[dest_pile_index].visible.push(card);
								}
							}
							MoveDest::ToFoundation(suit) => {
								self.foundation_fill_levels.insert(suit, card.rank);
							}
						}
						self.move_history.push(mv);
						return true
					}
				}
			}
			Move::CycleStock => {
				// does nothing if the stock has 0 or 1 cards
				if self.stock.len() > 1 {
					let card = self.stock.pop_front().unwrap();
					self.stock.push_back(card);
					self.move_history.push(mv);
					return true
				}
			}
		}
		return false
	}

	pub fn exec_move_in_progress(&mut self, target:MouseTarget) {
		if let Some(mip) = &self.move_in_progress {
			for mv in mip.moves.to_owned() {
				match mv {
					Move::CardMove { dest, .. } => {
						match dest {
							MoveDest::ToPile(mip_pile_index) => {
								match target {
									MouseTarget::StockTop => {} // impossible
									MouseTarget::StockDeck => {} // impossible
									MouseTarget::Foundation(_) => {} // not relevant for this move in progress
									MouseTarget::EmptyPile(target_pile_index) => {
										if mip_pile_index == target_pile_index {
											self.exec_move(mv);
											self.move_in_progress = None;
											return
										}
									}
									MouseTarget::PileCard{pile_index:target_pile_index, ..} => {
										if mip_pile_index == target_pile_index {
											self.exec_move(mv);
											self.move_in_progress = None;
											return
										}
									}
								}
							}
							MoveDest::ToFoundation(mip_suit) => {
								match target {
									MouseTarget::StockTop => {} // impossible
									MouseTarget::StockDeck => {} // impossible
									MouseTarget::Foundation(target_suit) => {
										if mip_suit == target_suit {
											self.exec_move(mv);
											self.move_in_progress = None;
											return
										}
									}
									MouseTarget::EmptyPile(_) => {} // not relevant for this move in progress
									MouseTarget::PileCard{..} => {} // not relevant for this move in progress
								}
							}
						}
					}
					Move::CycleStock => {
						self.exec_move(mv);
					}
				}
			}
		}
		// if we got to here, it means the move failed. Clear it
		self.move_in_progress = None;
	}

	// pops the top card off the given foundation.
	// returns None if the foundation is empty.
	pub fn pop_foundation(&mut self, suit:Suit) -> Option<Card> {
		let top_card = self.foundation_top_card(suit)?;
		if let Some(pred) = top_card.rank.pred() {
			self.foundation_fill_levels.insert(suit, pred);
		} else {
			self.foundation_fill_levels.remove(&suit);
		}
		Some(top_card)
	}

	// if there are no moves to undo, does nothing
	pub fn undo_move(&mut self) {
		if let Some(mv) = self.move_history.pop() {
			match mv {
				Move::CardMove{ card:_, src, dest } => {
					match src {
						MoveSrc::FromStock => {
							match dest {
								MoveDest::ToPile(dest_pile_index) => {
									if let Some(card) = self.piles[dest_pile_index].visible.pop() {
										self.stock.push_front(card);
									}
								}
								MoveDest::ToFoundation(suit) => {
									if let Some(card) = self.pop_foundation(suit) {
										self.stock.push_front(card);
									}
								}
							}
						}
						MoveSrc::FromFoundation(suit) => {
							match dest {
								MoveDest::ToPile(dest_pile_index) => {
									if let Some(card) = self.piles[dest_pile_index].visible.pop() {
										self.foundation_fill_levels.insert(suit, card.rank);
									}
								}
								MoveDest::ToFoundation(_) => {} // impossible
							}
						}
						MoveSrc::FromPile{ pile_index, n_cards, target_card_index, .. } => {
							match dest {
								MoveDest::ToPile(dest_pile_index) => {
									let dest_pile = &mut self.piles[dest_pile_index];
									let index = dest_pile.visible.len() - n_cards as usize;
									let removed:Vec<Card> = dest_pile.visible.drain(index..).collect();

									let src_pile = &mut self.piles[pile_index];

									// check if need to re-hide the prev hidden card
									if target_card_index == 0 && src_pile.visible.len() == 1 {
										if let Some(card_to_rehide) = src_pile.visible.pop() {
											src_pile.hidden.push(card_to_rehide);
										}
									}

									for card in removed {
										src_pile.visible.push(card);
									}
								}
								MoveDest::ToFoundation(suit) => {
									if let Some(card) = self.pop_foundation(suit) {
										let src_pile = &mut self.piles[pile_index];

										// check if need to re-hide the prev hidden card
										if target_card_index == 0 && src_pile.visible.len() == 1 {
											if let Some(card_to_rehide) = src_pile.visible.pop() {
												src_pile.hidden.push(card_to_rehide);
											}
										}

										src_pile.visible.push(card);
									}
								}
							}
						}
					}
				}
				Move::CycleStock => {
					if self.stock.len() > 1 {
						let card = self.stock.pop_back().unwrap();
						self.stock.push_front(card);
					}
				}
			}
		}
	}

	pub fn auto_move(&mut self) {
		if let Some(moves) = self.calc_moves(MouseTarget::StockTop) {
			let mv = moves.as_slice().first().unwrap().to_owned();
			self.exec_move(mv);
			return;
		}

		for (pile_index, pile) in self.piles[..].into_iter().enumerate() {
			// check visible cards in reverse order
			for (card_index, card) in pile.visible[..].into_iter().enumerate().rev() {
				let target = MouseTarget::PileCard{
					pile_index,
					n_cards: (pile.visible.len() - card_index) as u8,
					target_card: *card,
					target_card_index: card_index,
					top: 0., // doesn't matter for this purpose
				};
				if let Some(moves) = self.calc_moves(target) {
					let mv = moves.as_slice().first().unwrap().to_owned();
					self.exec_move(mv);
					return;
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
	StockTop, // the visible card of the stock
	StockDeck, // the rest of the stock
	Foundation(Suit), // one of the foundation piles
	EmptyPile(usize), // an empty pile (valid target if moving a King to an empty space)
	PileCard{ // a particular card in a pile
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

#[derive(Copy, Clone, Debug)]
enum Move {
	CardMove{
		card: Card,
		src: MoveSrc,
		dest: MoveDest,
	},
	CycleStock,
}

#[derive(Copy, Clone, Debug)]
enum MoveSrc {
	FromStock,
	FromFoundation(Suit),
	FromPile{
		pile_index:usize, // 0 is the leftmost pile
		n_cards:u8, // 1 = only the top card, 2 = two top cards, etc
		target_card_index:usize, // the index into visible of the targeted card. Note: if this is
								 // 0 and the hidden size > 0, that means the move uncovers a
								 // hidden card
	},
}

#[derive(Copy, Clone, Debug)]
enum MoveDest {
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
		self.col() != other.col() && other.rank.index() - self.rank.index() == 1
	}

	// returns true if self can stack on top of other in a foundation stack, eg. if self is 2D and other is AD.
	pub fn can_stack_onto_in_foundation(&self, other:Card) -> bool {
		self.suit == other.suit && self.rank.index() - other.rank.index() == 1
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

impl Rank {
	pub fn from_index(i: i8) -> Option<Rank> {
		return match i {
			0 => Some(Rank::Ace),
			1 => Some(Rank::Two),
			2 => Some(Rank::Three),
			3 => Some(Rank::Four),
			4 => Some(Rank::Five),
			5 => Some(Rank::Six),
			6 => Some(Rank::Seven),
			7 => Some(Rank::Eight),
			8 => Some(Rank::Nine),
			9 => Some(Rank::Ten),
			10 => Some(Rank::Jack),
			11 => Some(Rank::Queen),
			12 => Some(Rank::King),
			_ => None
		}
	}

	pub fn letter(&self) -> &str {
		return match self {
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

	pub fn index(&self) -> i8 {
		return match self {
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

	// returns the predecessor of the current rank.
	// eg. pred(King) == Some(Queen), pred(Ace) = None.
	pub fn pred(&self) -> Option<Rank> {
		let prev_index = self.index() - 1;
		Rank::from_index(prev_index)
	}
}
