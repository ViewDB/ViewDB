// Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(slice_patterns)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
pub extern crate pumpkindb_engine;
extern crate pumpkinscript;

mod mod_core;

use pumpkindb_engine::script::{Env, EnvId, PassResult, Dispatcher, Error, TryInstruction};

pub struct ViewDBDispatcher<'a, D : Dispatcher<'a>> {
    core: mod_core::Handler<'a>,
    fallback: D
}

impl<'a, D : Dispatcher<'a>> ViewDBDispatcher<'a, D> {
    pub fn new(fallback: D) -> Self {
        ViewDBDispatcher {
            core: mod_core::Handler::new(),
            fallback,
        }
    }
}

impl<'a, D : Dispatcher<'a>> Dispatcher<'a> for ViewDBDispatcher<'a, D> {
    fn init(&mut self, env: &mut Env<'a>, pid: EnvId) {
        self.fallback.init(env, pid)
    }
    fn done(&mut self, env: &mut Env<'a>, pid: EnvId) {
        self.fallback.done(env, pid)
    }
    fn handle(&mut self, env: &mut Env<'a>, instruction: &'a [u8], pid: EnvId) -> PassResult<'a> {
        self.core.handle(env, instruction, pid)
            .if_unhandled_try(|| self.fallback.handle(env, instruction, pid))
            .if_unhandled_try(|| Err(Error::UnknownInstruction))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
