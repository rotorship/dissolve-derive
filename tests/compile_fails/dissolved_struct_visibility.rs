mod inner {
    use dissolve_derive::Dissolve;

    #[derive(Dissolve)]
    #[dissolve(visibility = "pub(self)")]
    pub struct PrivateDissolved {
        pub value: i32,
    }
}

fn main() {
    let s = inner::PrivateDissolved { value: 42 };

    let inner::PrivateDissolvedDissolved { value } = s.dissolve();
}
