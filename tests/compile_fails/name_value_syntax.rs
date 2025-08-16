use dissolve_derive::Dissolve;

#[derive(Dissolve)]
struct NameValueSyntax {
	#[dissolved = "skip"]
	field: String,
}

fn main() {}
