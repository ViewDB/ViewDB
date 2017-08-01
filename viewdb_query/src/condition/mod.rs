// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value<T : AsRef<[u8]> + Clone> {
    Data(T),
    Binding(T),
    Attribute(T),
    AttributeTxid(T),
}

#[derive(Debug, PartialEq)]
pub enum Condition<T : AsRef<[u8]> + Clone> {
    // Fact scoping
    Fact(Box<Condition<T>>),
    // Boolean logic
    Not(Box<Condition<T>>),
    And(Box<Condition<T>>, Box<Condition<T>>),
    Or(Box<Condition<T>>, Box<Condition<T>>),
    // Trait scoping
    Trait(T, Box<Condition<T>>),
    // Conditions
    Present(Value<T>),
    Equal(Value<T>, Value<T>),
    LessThan(Value<T>, Value<T>),
    GreaterThan(Value<T>, Value<T>),
    True, False,
}

pub mod processing;

impl<T : AsRef<[u8]> + Clone> Condition<T> {

    #[inline]
    pub fn not(cond: Condition<T>) -> Self {
        Condition::Not(Box::new(cond))
    }

    #[inline]
    pub fn and(self, c2: Condition<T>) -> Self {
        Condition::And(Box::new(self), Box::new(c2))
    }

    #[inline]
    pub fn or(self, c2: Condition<T>) -> Self {
        Condition::Or(Box::new(self), Box::new(c2))
    }

    #[inline]
    pub fn trait_scope(t: T, c: Condition<T>) -> Self {
        Condition::Trait(t, Box::new(c))
    }

    #[inline]
    pub fn fact(c: Condition<T>) -> Self {
        Condition::Fact(Box::new(c))
    }

}

use std::ops::Not;

impl<T: AsRef<[u8]> + Clone> Not for Condition<T> {
    type Output = Condition<T>;

    fn not(self) -> Self::Output {
        Condition::Not(Box::new(self))
    }
}