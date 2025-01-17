/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
*/

//! vSMTP protocol implementation
//!
//! Currently only implement a ESMTPSA server.

#![doc(html_no_source)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
//
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::restriction)]
// restriction we ignore
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::blanket_clippy_restriction_lints,
    clippy::pub_use,
    clippy::shadow_reuse,
    clippy::implicit_return,
    clippy::arithmetic_side_effects,
    clippy::integer_arithmetic,
    clippy::shadow_same,
    clippy::shadow_unrelated
)]
// FIXME:
#![allow(clippy::std_instead_of_core)] // issue with thiserror
#![allow(clippy::indexing_slicing)] // issue with async_stream
#![allow(clippy::question_mark_used)]

extern crate alloc;

mod command;
mod connection_kind;
mod error;
mod reader;
mod receiver;
mod receiver_handler;
mod smtp_sasl;
mod writer;

pub use command::{
    AcceptArgs, AuthArgs, DsnReturn, EhloArgs, HeloArgs, MailFromArgs, NotifyOn, OriginalRecipient,
    RcptToArgs, UnparsedArgs, Verb,
};
pub use connection_kind::ConnectionKind;
pub use error::{Error, ErrorKind, ParseArgsError};
pub use reader::Reader;
pub use receiver::{Receiver, ReceiverContext};
pub use receiver_handler::ReceiverHandler;
pub use rsasl;
pub use smtp_sasl::{AuthError, CallbackWrap};
pub use tokio_rustls;
pub use tokio_rustls::rustls;
pub use writer::Writer;
