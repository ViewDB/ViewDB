// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Condition, Value};
use super::super::{TraitPattern, TraitResolver};

use std::marker::PhantomData;

pub trait Processor<T> where T: AsRef<[u8]> + Clone {
    fn process(&self, condition: Condition<T>) -> Option<Condition<T>>;
}

pub trait Recursive<T> : Processor<T> where T: AsRef<[u8]> + Clone {
    fn process_recursively(&self, condition: Condition<T>) -> Option<Condition<T>> {
        match condition {
            Condition::Fact(c) => Some(Condition::fact(try_opt!(self.process_recursively(try_opt!(self.process(*c)))))),
            Condition::Trait(t, c) => Some(Condition::trait_scope(t, try_opt!(self.process_recursively(try_opt!(self.process(*c)))))),
            Condition::And(c1, c2) => {
                match self.process(*c1).and_then(|v| self.process_recursively(v)) {
                    None => self.process(*c2).and_then(|v| self.process_recursively(v)),
                    Some(c) => Some(match self.process(*c2).and_then(|v| self.process_recursively(v)) {
                        None => c,
                        Some(c_) => c.and(c_)
                    })
                }
            },
            Condition::Or(c1, c2) => {
                match self.process(*c1).and_then(|v| self.process_recursively(v)) {
                    None => self.process(*c2).and_then(|v| self.process_recursively(v)),
                    Some(c) => Some(match self.process(*c2).and_then(|v| self.process_recursively(v)) {
                        None => c,
                        Some(c_) => c.or(c_)
                    })
                }
            },
            Condition::Not(c) => Some(!try_opt!(self.process_recursively(try_opt!(self.process(*c))))),
            _ => Some(condition),
        }
    }
}

pub struct TraitsExpansion<T : AsRef<[u8]> + Clone + PartialOrd, R : TraitResolver<T>>(R, PhantomData<T>);
impl<T: AsRef<[u8]> + Clone + PartialOrd, R : TraitResolver<T>> Recursive<T> for TraitsExpansion<T, R> {}


impl<T: AsRef<[u8]> + Clone + PartialOrd, R : TraitResolver<T>> TraitsExpansion<T, R> {
    pub fn new(resolver: R) -> Self {
        TraitsExpansion(resolver, PhantomData)
    }
}

impl<T: AsRef<[u8]> + Clone + PartialOrd, R : TraitResolver<T>> Processor<T> for TraitsExpansion<T, R> {
    fn process(&self, condition: Condition<T>) -> Option<Condition<T>> {
        match condition {
            Condition::Trait(name, boxed) => {
                let trait_def = self.0.resolve(name);
                let mut cond = try_opt!(self.process(*boxed));
                for pattern in trait_def.iter() {
                    match pattern {
                        &TraitPattern(ref attr, None) => {
                            cond = cond.and(Condition::Present(Value::Attribute(attr.clone())));
                        }
                        &TraitPattern(ref attr, Some(ref val)) => {
                            cond = cond.and(Condition::Equal(Value::Attribute(attr.clone()), Value::Data(val.clone())));
                        }
                    }
                }
                Some(cond)
            },
            c => self.process_recursively(c),
        }
    }
}

pub struct PresentEqualCompaction;
impl<T: AsRef<[u8]> + Clone + PartialOrd> Recursive<T> for PresentEqualCompaction {}

impl<T: AsRef<[u8]> + Clone + PartialOrd> Processor<T> for PresentEqualCompaction {
    fn process(&self, condition: Condition<T>) -> Option<Condition<T>> {
        fn contains_equal<T: AsRef<[u8]> + Clone + PartialOrd>(cond: &Condition<T>, attr: &Value<T>) -> bool {
            match cond {
                &Condition::Equal(ref a, _) if a == attr => true,
                &Condition::Fact(ref c) => contains_equal(c, attr),
                &Condition::And(ref c1, ref c2) => contains_equal(c1, attr) || contains_equal(c2, attr),
                &Condition::Or(ref c1, ref c2) => contains_equal(c1, attr) && contains_equal(c2, attr),
                _ => false,
            }
        }
        match condition {
            Condition::And(c1, c2) => {
                match (*c1, *c2) {
                    (Condition::Present(a1), cond) | (cond, Condition::Present(a1)) =>
                        if contains_equal(&cond, &a1) {
                            Some(cond)
                        } else {
                            Some(Condition::Present(a1).and(try_opt!(self.process(cond))))
                        },
                    (c1, c2) =>
                        Some(try_opt!(self.process(c1)).and(try_opt!(self.process(c2)))),
                }
            },
            c => self.process_recursively(c),
        }
    }
}

pub struct ComparisonSuppression;
impl<T: AsRef<[u8]> + Clone + PartialOrd> Recursive<T> for ComparisonSuppression {}

impl<T: AsRef<[u8]> + Clone + PartialOrd> Processor<T> for ComparisonSuppression {
    fn process(&self, condition: Condition<T>) -> Option<Condition<T>> {
        match condition {
            Condition::Equal(Value::Data(ref v1), Value::Data(ref v2)) if v1 == v2 => None,
            Condition::Equal(Value::Data(_), Value::Data(_)) => Some(Condition::False),
            Condition::GreaterThan(Value::Data(ref v1), Value::Data(ref v2)) if v1 > v2 => None,
            Condition::GreaterThan(Value::Data(_), Value::Data(_)) => Some(Condition::False),
            Condition::LessThan(Value::Data(ref v1), Value::Data(ref v2)) if v1 < v2 => None,
            Condition::LessThan(Value::Data(_), Value::Data(_)) => Some(Condition::False),
            Condition::Equal(Value::Attribute(ref a1), Value::Attribute(ref a2)) if a1 == a2 => None,
            Condition::Equal(Value::Binding(ref b1), Value::Attribute(ref b2)) if b1 == b2 => None,
            c => self.process_recursively(c),
        }
    }
}

pub struct BooleanLiteralSuppression;
impl<T: AsRef<[u8]> + Clone + PartialOrd> Recursive<T> for BooleanLiteralSuppression {}

impl<T: AsRef<[u8]> + Clone + PartialOrd> Processor<T> for BooleanLiteralSuppression {
    fn process(&self, condition: Condition<T>) -> Option<Condition<T>> {
        match condition {
            Condition::And(a, b) =>
                if *a == Condition::False || *b == Condition::False {
                    None
                } else if *a == Condition::True {
                    Some(*b)
                } else if *b == Condition::True {
                    Some(*a)
                } else {
                    Some(a.and(*b))
                },
            c => self.process_recursively(c),
        }
    }
}
