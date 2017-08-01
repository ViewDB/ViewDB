// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::hash::Hash;
pub trait Identifier : Eq + Hash + Clone + Copy {
    fn generate() -> Self;
    fn identifier(&self) -> &[u8];
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Fact<I : Identifier>(I);

impl<I : Identifier> Fact<I> {
    pub fn new() -> Self {
        Fact::new_with_identifier(Identifier::generate())
    }

    pub fn new_with_identifier(identifier: I) -> Self {
        Fact(identifier)
    }

    pub fn identifier(&self) -> &[u8] {
        self.0.identifier()
    }
}

#[cfg(feature="uuid_v4_identifier")]
extern crate uuid;

#[cfg(feature="uuid_v4_identifier")]
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct UuidIdentifier(uuid::Uuid);

#[cfg(feature="uuid_v4_identifier")]
impl Identifier for UuidIdentifier {

    fn generate() -> Self {
        UuidIdentifier(uuid::Uuid::new_v4())
    }

    fn identifier(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

pub struct Attribute<T : AsRef<[u8]>>(T, T);
impl<T : AsRef<[u8]>> Attribute<T> {
    pub fn new(name: T, value: T) -> Self {
        Attribute(name, value)
    }
}

#[derive(Clone)]
pub struct TraitPattern<T : AsRef<[u8]> + Clone>(pub T, pub Option<T>);
#[derive(Clone)]
pub struct Trait<T : AsRef<[u8]> + Clone>(Vec<TraitPattern<T>>);

impl<T : AsRef<[u8]> + Clone> From<Vec<TraitPattern<T>>> for Trait<T> {
    fn from(v: Vec<TraitPattern<T>>) -> Self {
        Trait(v)
    }
}

impl<T : AsRef<[u8]> + Clone> Trait<T> {

    pub fn iter(&self) -> ::std::slice::Iter<TraitPattern<T>> {
        self.0.iter()
    }
}

impl<T : AsRef<[u8]> + Clone> From<(T, Option<T>)> for TraitPattern<T> {
    fn from((t, o): (T, Option<T>)) -> Self {
        TraitPattern(t, o)
    }
}

pub trait TraitResolver<T : AsRef<[u8]> + Clone> {
    fn resolve(&self, name: T) -> &Trait<T>;
}


#[cfg(test)]
mod tests {

    #[cfg(feature="uuid_v4_identifier")]
    #[test]
    fn it_works() {
        let fact1 = super::Fact::<super::UuidIdentifier>::new();
        let fact2 = super::Fact::<super::UuidIdentifier>::new();
        assert!(fact1.identifier() != fact2.identifier());
    }
}
