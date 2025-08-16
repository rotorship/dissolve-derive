use dissolve_derive::Dissolve;

#[derive(Dissolve)]
struct InvalidIdentifier {
	#[dissolved(rename = "123invalid")]
	field: String,
}

fn main() {}
