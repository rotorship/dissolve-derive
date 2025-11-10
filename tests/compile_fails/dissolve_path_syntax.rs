use dissolve_derive::Dissolve;

#[derive(Dissolve)]
#[dissolve = "something"]
struct PathSyntax {
    field: String,
}

fn main() {}
