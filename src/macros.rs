macro_rules! simple_id {
    ($x: ident) => {
        impl std::hash::Hash for $x {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl PartialEq for $x {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl Eq for $x {}
    };
}

pub(crate) use simple_id;
