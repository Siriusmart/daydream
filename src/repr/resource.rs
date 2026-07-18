use std::{error::Error, fmt::Debug, hash::Hash, sync::Arc};

#[derive(Debug, Clone)]
pub enum Resource<T: Clone + Debug + Hash + PartialEq + Eq> {
    Loading,
    Loaded(T),
    Failed(Arc<dyn Error + Send + Sync>),
}

impl<T: Clone + Debug + Hash + PartialEq + Eq> Hash for Resource<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Loading => {}
            Self::Loaded(v) => v.hash(state),
            Self::Failed(f) => f.to_string().hash(state),
        }
    }
}

impl<T: Clone + Debug + Hash + PartialEq + Eq> PartialEq for Resource<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Loading, Self::Loading) => true,
            (Self::Loaded(a), Self::Loaded(b)) => a == b,
            (Self::Failed(a), Self::Failed(b)) => a.to_string() == b.to_string(),
            _ => false,
        }
    }
}

impl<T: Clone + Debug + Hash + PartialEq + Eq> Eq for Resource<T> {}

impl<T: Clone + Debug + Hash + PartialEq + Eq> Resource<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn as_ref(&self) -> Resource<&T> {
        match self {
            Self::Loaded(v) => Resource::Loaded(v),
            Self::Loading => Resource::Loading,
            Self::Failed(v) => Resource::Failed(v.clone()),
        }
    }

    pub fn map_loaded_or<U, F: Fn(T) -> U>(self, f: F, or: U) -> U {
        if let Self::Loaded(value) = self {
            f(value)
        } else {
            or
        }
    }

    pub fn and<U: Clone + Debug + Hash + PartialEq + Eq>(
        self,
        other: Resource<U>,
    ) -> Resource<(T, U)> {
        match self {
            Self::Loading => match other {
                Resource::Failed(err) => Resource::Failed(err),
                _ => Resource::Loading,
            },
            Self::Loaded(t) => match other {
                Resource::Loading => Resource::Loading,
                Resource::Loaded(u) => Resource::Loaded((t, u)),
                Resource::Failed(err) => Resource::Failed(err),
            },
            Self::Failed(err) => Resource::Failed(err),
        }
    }

    pub fn and_ref<'a, U: Clone + Debug + Hash + PartialEq + Eq>(
        &'a self,
        other: &'a Resource<U>,
    ) -> Resource<(&'a T, &'a U)> {
        match self {
            Self::Loading => match other {
                Resource::Failed(err) => Resource::Failed(err.clone()),
                _ => Resource::Loading,
            },
            Self::Loaded(t) => match other {
                Resource::Loading => Resource::Loading,
                Resource::Loaded(u) => Resource::Loaded((t, u)),
                Resource::Failed(err) => Resource::Failed(err.clone()),
            },
            Self::Failed(err) => Resource::Failed(err.clone()),
        }
    }
}
