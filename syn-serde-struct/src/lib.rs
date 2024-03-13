use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy, Debug)]
pub struct Syn<T>(pub T);

impl<T: syn_serde::Syn> Serialize for Syn<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return self.0.to_adapter().serialize(serializer);
    }
}
impl<'de, T: syn_serde::Syn> Deserialize<'de> for Syn<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        return T::Adapter::deserialize(deserializer).map(|a| Syn(T::from_adapter(&a)));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
}
