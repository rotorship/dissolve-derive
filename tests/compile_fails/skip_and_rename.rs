use dissolve_derive::Dissolve;

#[derive(Dissolve)]
struct SkipAndRename {
	#[dissolved(skip, rename = "new_name")]
	field: String,
}

fn main() {}
