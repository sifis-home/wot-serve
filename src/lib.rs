//! Web of Things application server
//!
//! Provides all the building blocks to serve [Web Of Things](https://www.w3.org/WoT/) Things.

pub mod advertise;
#[doc(hidden)]
pub mod hlist;
pub mod servient;

pub use servient::Servient;

/// Web of Things Servient builder
pub mod builder {
    pub use crate::servient::builder::*;
    pub use wot_td::builder::*;
}

/// Re-export of [wot_td::thing]
pub mod thing {
    pub use wot_td::thing::*;
}
