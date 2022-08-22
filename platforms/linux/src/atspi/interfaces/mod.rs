// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

mod accessible;
mod action;
mod application;
mod events;
mod value;

use enumflags2::{bitflags, BitFlags, FromBitsError};
use serde::{
    de::{self, Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
use std::fmt;
use zvariant::{Signature, Type};

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) enum Interface {
    Accessible,
    Application,
    Action,
    Value,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Interfaces(BitFlags<Interface, u32>);

impl Interfaces {
    pub fn new<B: Into<BitFlags<Interface>>>(value: B) -> Self {
        Self(value.into())
    }

    pub fn from_bits(bits: u32) -> Result<Interfaces, FromBitsError<Interface>> {
        Ok(Interfaces(BitFlags::from_bits(bits)?))
    }

    pub fn contains<B: Into<BitFlags<Interface>>>(self, other: B) -> bool {
        self.0.contains(other)
    }

    pub fn insert<B: Into<BitFlags<Interface>>>(&mut self, other: B) {
        self.0.insert(other);
    }

    pub fn iter(self) -> impl Iterator<Item = Interface> {
        self.0.iter()
    }
}

const INTERFACE_NAMES: &[&str] = &[
    "org.a11y.atspi.Accessible",
    "org.a11y.atspi.Application",
    "org.a11y.atspi.Action",
    "org.a11y.atspi.Value",
];

impl<'de> Deserialize<'de> for Interfaces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct InterfacesVisitor;

        impl<'de> Visitor<'de> for InterfacesVisitor {
            type Value = Interfaces;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence comprised of D-Bus interface names")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                match SeqAccess::next_element::<Vec<String>>(&mut seq)? {
                    Some(names) => {
                        let mut bits = 0;
                        for name in names {
                            if let Ok(index) = INTERFACE_NAMES.binary_search(&name.as_str()) {
                                bits &= 2u32.pow(index as u32);
                            }
                        }
                        Ok(Interfaces::from_bits(bits).unwrap())
                    }
                    None => Err(de::Error::custom("Vec containing D-Bus interface names")),
                }
            }
        }

        deserializer.deserialize_seq(InterfacesVisitor)
    }
}

impl Serialize for Interfaces {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut interfaces = Vec::with_capacity(INTERFACE_NAMES.len());
        for interface in self.iter() {
            interfaces.push(INTERFACE_NAMES[(interface as u32).trailing_zeros() as usize]);
        }
        let mut seq = serializer.serialize_seq(Some(interfaces.len()))?;
        for interface in interfaces {
            seq.serialize_element(interface)?;
        }
        seq.end()
    }
}

impl Type for Interfaces {
    fn signature() -> Signature<'static> {
        Signature::from_str_unchecked("as")
    }
}

impl From<Interface> for Interfaces {
    fn from(value: Interface) -> Self {
        Self(value.into())
    }
}

impl std::ops::BitAnd for Interfaces {
    type Output = Interfaces;

    fn bitand(self, other: Self) -> Self::Output {
        Interfaces(self.0 & other.0)
    }
}

impl std::ops::BitXor for Interfaces {
    type Output = Interfaces;

    fn bitxor(self, other: Self) -> Self::Output {
        Interfaces(self.0 ^ other.0)
    }
}

pub(crate) use accessible::*;
pub(crate) use action::*;
pub(crate) use application::*;
pub(crate) use events::*;
pub(crate) use value::*;
