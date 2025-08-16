use dissolve_derive::Dissolve;

// Test 1: All fields skipped should fail
#[derive(Dissolve)]
struct AllSkipped {
	#[dissolved(skip)]
	field1: String,

	#[dissolved(skip)]
	field2: i32,
}

fn main() {}
