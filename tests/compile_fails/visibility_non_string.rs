use dissolve_derive::Dissolve;

#[derive(Dissolve)]
#[dissolve(visibility = 42)]
struct VisibilityNonString {
    field: String,
}

fn main() {}
