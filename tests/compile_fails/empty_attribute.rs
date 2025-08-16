use dissolve_derive::Dissolve;

#[derive(Dissolve)]
struct EmptyAttribute {
	#[dissolved]
	field: String,
}

fn main() {}
