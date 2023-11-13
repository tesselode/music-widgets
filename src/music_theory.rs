#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeSignature {
	pub top: u8,
	pub bottom: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chord {
	pub note: Note,
	pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Note {
	pub name: NoteName,
	pub accidental: Accidental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteName {
	A,
	B,
	C,
	D,
	E,
	F,
	G,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
	None,
	Flat,
	Sharp,
}
