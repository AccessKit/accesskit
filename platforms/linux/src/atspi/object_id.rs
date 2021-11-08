use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use zvariant::{
    derive::{Type, Value},
    Str
};

#[derive(Clone, Debug, Deserialize, Serialize, Type, Value)]
pub struct ObjectId<'a>(#[serde(borrow)] Str<'a>);

impl<'a> ObjectId<'a> {
    pub unsafe fn from_str_unchecked(id: &'a str) -> ObjectId<'a> {
        Self(Str::from(id))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<NonZeroU64> for ObjectId<'static> {
    fn from(value: NonZeroU64) -> Self {
        Self(Str::from(value.to_string()))
    }
}
