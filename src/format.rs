#![allow(unreachable_patterns)] // ComplexTypeDiff::Literal triggers this

use serde::{Deserialize, Serialize};

#[cfg(feature = "diff")]
use structdiff::{Difference, StructDiff};

pub mod prototype;
pub mod runtime;

#[cfg(not(feature = "diff"))]
type FakeDiffableVec<T> = Vec<T>;

#[cfg(feature = "diff")]
mod diff_helper {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use structdiff::StructDiff;

    pub(super) trait Named {
        fn name(&self) -> &str;
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct DiffableVec<V> {
        map: HashMap<String, V>,
    }

    pub type DiffableVecDiff<V> = HashMap<String, Vec<<V as StructDiff>::Diff>>;
    pub type SingleDiff<V> = Vec<<V as StructDiff>::Diff>;

    impl<T: Named> From<Vec<T>> for DiffableVec<T> {
        fn from(value: Vec<T>) -> Self {
            Self {
                map: value
                    .into_iter()
                    .map(|p| (p.name().to_owned(), p))
                    .collect(),
            }
        }
    }

    impl<T> Default for DiffableVec<T> {
        fn default() -> Self {
            Self {
                map: HashMap::new(),
            }
        }
    }

    impl<T> std::ops::Deref for DiffableVec<T> {
        type Target = HashMap<String, T>;

        fn deref(&self) -> &Self::Target {
            &self.map
        }
    }

    impl<T> DiffableVec<T>
    where
        T: StructDiff + Default,
    {
        #[must_use]
        pub fn diff(&self, other: &Self) -> DiffableVecDiff<T> {
            let mut diff = HashMap::new();

            for (k, v) in &self.map {
                if let Some(o) = other.map.get(k) {
                    let d = v.diff(o);
                    if !d.is_empty() {
                        diff.insert(k.clone(), d);
                    }
                } else {
                    diff.insert(k.clone(), v.diff(&T::default()));
                }
            }

            for (k, v) in &other.map {
                if !self.map.contains_key(k) {
                    diff.insert(k.clone(), T::default().diff(v));
                }
            }

            diff
        }

        pub fn full(&self) -> DiffableVecDiff<T> {
            self.map
                .iter()
                .map(|(k, v)| (k.clone(), v.diff(&T::default())))
                .collect()
        }
    }

    impl<T> serde::Serialize for DiffableVec<T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let value = self.values().collect::<Vec<_>>();
            value.serialize(serializer)
        }
    }

    impl<'de, T> serde::Deserialize<'de> for DiffableVec<T>
    where
        T: Deserialize<'de> + Named,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = Vec::<T>::deserialize(deserializer)?;
            Ok(Self {
                map: value
                    .into_iter()
                    .map(|p| (p.name().to_owned(), p))
                    .collect(),
            })
        }
    }

    pub fn vec_diff<T: StructDiff + Default>(orig: &[T], new: &[T]) -> Vec<SingleDiff<T>> {
        let mut diff = Vec::new();

        for (i, v) in orig.iter().enumerate() {
            if let Some(n) = new.get(i) {
                diff.push(v.diff(n));
            } else {
                diff.push(v.diff(&T::default()));
            }
        }

        new.iter()
            .skip(orig.len())
            .for_each(|n| diff.push(T::default().diff(n)));

        diff
    }
}

#[cfg(feature = "diff")]
pub trait Doc {
    type Diff;

    #[must_use]
    fn diff(&self, other: &Self) -> Self::Diff;
}

pub trait Info {
    fn print_info(&self);
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone)]
#[cfg_attr(feature = "diff", derive(Difference))]
#[serde(rename_all = "lowercase")]
pub enum Application {
    #[default]
    Factorio,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone)]
#[cfg_attr(feature = "diff", derive(Difference))]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    #[default]
    Prototype,
    Runtime,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone)]
#[cfg_attr(feature = "diff", derive(Difference))]
pub struct Common {
    pub application: Application,
    pub stage: Stage,
    pub application_version: String,
    pub api_version: u8,
}

impl Info for Common {
    fn print_info(&self) {
        eprintln!(
            "{:?} @ {}: {:?}",
            self.application, self.application_version, self.stage
        );
    }
}

#[cfg(feature = "non_type_info")]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Clone, Default, Hash)]
#[cfg_attr(feature = "diff", derive(Difference))]
pub struct Image {
    pub filename: String,
    pub caption: Option<String>,
}
