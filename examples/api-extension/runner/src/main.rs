/* Copyright (c) Fortanix, Inc.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate aesm_client;
extern crate enclave_runner;
extern crate sgxs_loaders;
extern crate failure;
#[macro_use]
extern crate clap;

use aesm_client::AesmClient;
use enclave_runner::{ApiExtension, EnclaveBuilder, OutputBuffer, SyncStream};
use failure::{Error, ResultExt};
use sgxs_loaders::isgx::Device as IsgxDevice;

use clap::{App, Arg};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind as IoErrorKind, Result as IoResult};

arg_enum!{
    #[derive(PartialEq, Debug)]
    #[allow(non_camel_case_types)]
    pub enum Signature {
        coresident,
        dummy
    }
}

struct FileStream {
    f : File,
}
impl SyncStream for FileStream {
    fn read(&self, buf: &mut [u8]) -> IoResult<usize> {
        self.f.read(buf)
    }
    fn write(&self, buf: &[u8]) -> IoResult<usize> {
        self.f.write(buf)
    }
    fn flush(&self) -> IoResult<()> {
        self.f.flush()
    }
}

struct FileOps;
impl ApiExtension for FileOps {
    fn connect_stream(
        &self,
        addr: &[u8],
        _local_addr: Option<&mut OutputBuffer>,
        _peer_addr: Option<&mut OutputBuffer>,
    ) -> IoResult<Box<SyncStream>> {
        let name = String::from_utf8(addr.to_vec()).map_err(|_| IoErrorKind::ConnectionRefused)?;
         let file = OpenOptions::new().write(true).read(true).truncate(true).create(true)
                             .open(&name)?;
        let stream = FileStream{f : file};
        Ok(Box::new(stream))
    }
}



fn main() -> Result<(), Error> {
    let args = App::new("ftxsgx-runner")
        .arg(
            Arg::with_name("file")
                .required(true)
        )
        .arg(Arg::with_name("signature")
            .short("s")
            .long("signature")
            .required(false)
            .takes_value(true)
            .possible_values(&Signature::variants()))
        .get_matches();

    let file = args.value_of("file").unwrap();

    let mut device = IsgxDevice::new()
        .context("While opening SGX device")?
        .einittoken_provider(AesmClient::new())
        .build();

    let mut enclave_builder = EnclaveBuilder::new(file.as_ref());
    enclave_builder.api_ext_impl(Box::new(FileOps));

    match args.value_of("signature").map(|v| v.parse().expect("validated")) {
        Some(Signature::coresident) => { enclave_builder.coresident_signature().context("While loading coresident signature")?; }
        Some(Signature::dummy) => { enclave_builder.dummy_signature(); },
        None => (),
    }

    let enclave = enclave_builder.build(&mut device).context("While loading SGX enclave")?;

    enclave.run().map_err(|e| {
        println!("Error while executing SGX enclave.\n{}", e);
        std::process::exit(-1)
    })
}
