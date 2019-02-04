/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::Path;

use failure::Error;
use sgxs::loader::{Load, MappingInfo};

use loader::{EnclaveBuilder, ErasedTcs};
use std::os::raw::c_void;
use usercalls::EnclaveState;
use usercalls::ApiExtension;
use std::fmt;

pub struct Command {
    main: ErasedTcs,
    threads: Vec<ErasedTcs>,
    address: *mut c_void,
    size: usize,
    api_ext_impl : Option<Box<ApiExtension>>,
}

impl fmt::Debug for Command {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Command")
            .field("main", &format_args!("{:?}", self.main))
            .field("threads", &format_args!("{:?}", self.threads))
            .field("address", &self.address)
            .field("size", &self.size)
            .finish()
    }
}

impl MappingInfo for Command {
    fn address(&self) -> *mut c_void {
        self.address
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl Command {
    /// # Panics
    /// Panics if the number of TCSs is 0.
    pub(crate) fn internal_new(mut tcss: Vec<ErasedTcs>, address: *mut c_void, size: usize,
                      api_ext_impl : Option<Box<ApiExtension>>) -> Command {
        let main = tcss.remove(0);
        Command {
            main,
            threads: tcss,
            address,
            size,
            api_ext_impl,
        }
    }

    pub fn new<P: AsRef<Path>, L: Load>(enclave_path: P, loader: &mut L) -> Result<Command, Error> {
        EnclaveBuilder::new(enclave_path.as_ref()).build(loader)
    }

    pub fn run(self) -> Result<(), Error> {
        EnclaveState::main_entry(self.main, self.threads, self.api_ext_impl)
    }
}
