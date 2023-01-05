// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

mod accessible;
mod action;
mod application;
mod component;
mod events;
mod value;

use crate::atspi::{ObjectRef, OwnedObjectAddress};
use zbus::{
    fdo,
    names::{BusName, OwnedUniqueName, UniqueName},
};

fn object_address(
    destination: Option<&BusName>,
    object_ref: Option<ObjectRef>,
) -> fdo::Result<(OwnedObjectAddress,)> {
    match object_ref {
        Some(ObjectRef::Managed(id)) => {
            Ok((OwnedObjectAddress::accessible(app_name(destination)?, id),))
        }
        Some(ObjectRef::Unmanaged(address)) => Ok((address,)),
        None => Ok((OwnedObjectAddress::null(app_name(destination)?),)),
    }
}

fn app_name(destination: Option<&BusName>) -> fdo::Result<OwnedUniqueName> {
    let destination = destination.ok_or(fdo::Error::ZBus(zbus::Error::MissingField))?;
    Ok(UniqueName::from_str_unchecked(destination.as_str()).into())
}

pub(crate) use accessible::*;
pub(crate) use action::*;
pub(crate) use application::*;
pub(crate) use component::*;
pub(crate) use events::*;
pub(crate) use value::*;
