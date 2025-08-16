use dissolve_derive::Dissolve;

#[derive(Dissolve)]
struct TupleRename(#[dissolved(rename = "name")] String);

fn main() {}
