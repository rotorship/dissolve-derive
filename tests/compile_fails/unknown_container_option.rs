use dissolve_derive::Dissolve;

#[derive(Dissolve)]
#[dissolve(unknown_option = "value")]
struct UnknownOption {
    field: String,
}

fn main() {}
