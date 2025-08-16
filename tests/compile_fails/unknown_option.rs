use dissolve_derive::Dissolve;

#[derive(Dissolve)]
struct UnknownOption {
	#[dissolved(unknown)]
	field: String,
}

fn main() {}
