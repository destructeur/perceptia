// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

//! `qualia` is crate containing enumerations and simple tools common to all the crates of
//! `perceptia`.

extern crate dbus;
extern crate libc;
extern crate libudev; // for implementation of `From` in `errors`.
extern crate nix;
extern crate time;

#[macro_use(timber)]
extern crate timber;
extern crate dharma;

pub mod enums;
pub use enums::{DeviceKind, KeyState};

pub mod perceptron;
pub use perceptron::Perceptron;

pub mod errors;
pub use errors::Error;

pub mod defs;
pub use defs::{Area, Point, Button, Position, OptionalPosition, Size, Vector, DrmBundle};

pub mod config;
pub use config::{Config, InputConfig};

pub mod buffer;
pub use buffer::Buffer;

#[macro_use]
pub mod log;
pub use log::level;

pub mod env;
pub use env::Env;

pub mod coordinator;
pub use coordinator::{Coordinator, SurfaceAccess, SurfaceId, SurfaceInfo, show_reason};

pub mod surface;
pub use surface::SurfaceContext;

pub mod context;
pub use context::Context;

pub mod ipc;
pub use ipc::Ipc;
