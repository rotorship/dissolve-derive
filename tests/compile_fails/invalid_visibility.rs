use dissolve_derive::Dissolve;

#[derive(Dissolve)]
#[dissolve(visibility = "invalid_visibility")]
struct InvalidVisibility {
    field: String,
}

fn main() {}
