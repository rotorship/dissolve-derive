use dissolve_derive::Dissolve;

#[derive(Dissolve)]
struct EmptyRename {
	#[dissolved(rename = "")]
	field: String,
}

fn main() {}
