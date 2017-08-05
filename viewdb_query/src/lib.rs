// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#[macro_use]
extern crate try_opt;
extern crate viewdb_core;

#[cfg(test)] #[macro_use]
extern crate assert_matches;

pub(crate) use viewdb_core::{TraitPattern, TraitResolver};
#[cfg(test)]
pub(crate) use viewdb_core::{Trait};

pub mod condition;
pub use condition::{Condition, Value};

#[cfg(test)]
mod tests {
    use super::{Value, Trait, TraitResolver, Condition};
    use super::Condition::{Equal};
    use condition::processing::{Processor, ProcessorExtension, TraitsExpansion, PresentEqualCompaction,
                                ComparisonSuppression, BooleanLiteralSuppression, ImplicitFact};

    #[derive(Clone)]
    pub struct Test<T : AsRef<[u8]> + Clone>((T, Trait<T>), (T, Trait<T>), (T, Trait<T>));

    impl<T : AsRef<[u8]> + Clone> TraitResolver<T> for Test<T>  {
        fn resolve(&self, name: T) -> &Trait<T> {
            if name.as_ref() == (self.0).0.as_ref() {
                return &(self.0).1
            }
            if name.as_ref() == (self.1).0.as_ref() {
                return &(self.1).1
            }
            if name.as_ref() == (self.2).0.as_ref() {
                return &(self.2).1
            }
            unreachable!()
        }
    }

    #[test]
    fn it_works() {
        let cond =
                Condition::trait_scope("Object", Equal(Value::Attribute("https://viewdb.org/attributes#object"), Value::Data("123")))
                .and(Condition::trait_scope("NameChanged", Equal(Value::Attribute("https://viewdb.org/attributes#value"), Value::Binding("Name"))))
                .and(Condition::trait_scope("Timestamp", Equal(Value::Attribute("https://viewdb.org/attributes#timestamp"), Value::Binding("Timestamp"))));

        let object = vec![("https://viewdb.org/attributes#object", None).into()].into();
        let name_changed = vec![("https://viewdb.org/attributes#factType", Some("NameChanged")).into(),
                                      ("https://viewdb.org/attributes#value", None).into()].into();
        let timestamp = vec![("https://viewdb.org/attributes#timestamp", None).into()].into();

        let te = TraitsExpansion::new(Test(("Object", object), ("NameChanged", name_changed), ("Timestamp", timestamp)));

        let cond1 = te.process(cond)
                    .after_that(PresentEqualCompaction)
                    .after_that(ComparisonSuppression)
                    .after_that(BooleanLiteralSuppression)
                    .after_that(ImplicitFact)
                    .unwrap();

        println!("{:#?}", cond1);
    }
}
