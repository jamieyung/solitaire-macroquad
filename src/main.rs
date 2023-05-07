use macroquad::prelude::*;

#[macroquad::main("Hello")]
async fn main() {
	let mut pile = Pile {
		cards: Vec::new(),
	};

	pile.cards.push(Card {
		suit: Suit::Diamonds,
		val: Value::Ace,
		hidden: false,
	});

	pile.cards.push(Card {
		suit: Suit::Clubs,
		val: Value::Six,
		hidden: false,
	});

	pile.cards.push(Card {
		suit: Suit::Hearts,
		val: Value::Ten,
		hidden: false,
	});

	pile.cards.push(Card {
		suit: Suit::Spades,
		val: Value::King,
		hidden: false,
	});

    loop {
		clear_background(GREEN);

		if is_key_down(KeyCode::Q) {
			return;
		}

		draw_pile(&pile, 10., 10.);

		next_frame().await;
	}
}

const CARD_W: f32 = 50.;
const CARD_H: f32 = 70.;
const PILE_CARD_OFFSET: f32 = 20.;

fn draw_pile(pile: &Pile, x:f32, y:f32) {
	let s = &pile.cards[..];
	for (i, card) in s.into_iter().enumerate() {
		draw_card(card, x, y + (i as f32) * PILE_CARD_OFFSET);
	}
}

fn draw_card(c: &Card, x:f32, y:f32) {
	draw_rectangle(x, y, CARD_W, CARD_H, WHITE);
	draw_rectangle_lines(x, y, CARD_W, CARD_H, 2., BLACK);

	if c.hidden {
		draw_rectangle(x+3., y+3., CARD_W-6., CARD_H-6., BLUE);
	} else {
		let col = card_col(c);
		draw_text(card_value_letter(c), x, y + 17.0, 30.0, col);
		draw_text(card_suit_letter(c), x+CARD_W*0.7, y + 17.0, 30.0, col);
	}
}

fn card_col(c: &Card) -> Color {
	return match c.suit {
		Suit::Diamonds | Suit::Hearts => RED,
		Suit::Clubs | Suit::Spades => BLACK,
	}
}

fn card_suit_letter(c: &Card) -> &str {
	return match c.suit {
		Suit::Diamonds => "D",
		Suit::Clubs => "C",
		Suit::Hearts => "H",
		Suit::Spades => "S",
	}
}

fn card_value_letter(c: &Card) -> &str {
	return match c.val {
		Value::Ace => "A",
		Value::One => "1",
		Value::Two => "2",
		Value::Three => "3",
		Value::Four => "4",
		Value::Five => "5",
		Value::Six => "6",
		Value::Seven => "7",
		Value::Eight => "8",
		Value::Nine => "9",
		Value::Ten => "10",
		Value::Jack => "J",
		Value::Queen => "Q",
		Value::King => "K",
	}
}

struct Pile {
	cards: Vec<Card>
}

struct Card {
	suit: Suit,
	val: Value,
	hidden: bool,
}

enum Suit {
	Diamonds,
	Clubs,
	Hearts,
	Spades
}

enum Value {
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
	King
}
