//! Web of Thing Servient

use std::net::SocketAddr;

use crate::{advertise::Advertiser, advertise::ThingType, hlist::NilPlus};
use axum::Router;
use wot_td::{
    builder::{ThingBuilder, ToExtend},
    extend::ExtendableThing,
    hlist::*,
    thing::Thing,
};

mod builder;

pub use builder::*;

/// WoT Servient serving a Thing Description
pub struct Servient<Other: ExtendableThing = Nil> {
    /// hostname for the thing
    ///
    /// Used in the DNS-SD advertisement by default
    pub name: String,
    /// The Thing Description representing the servient
    pub thing: Thing<Other>,
    /// The http router for the servient
    pub router: Router,
    /// DNS-SD advertisement
    pub sd: Advertiser,
    /// Address the http server will bind to
    pub http_addr: SocketAddr,
    /// The type of thing advertised
    pub thing_type: ThingType,
}

impl Servient<Nil> {
    /// Instantiate a ThingBuilder with its Form augmented with [[HttpRouter]] methods.
    pub fn builder(title: impl Into<String>) -> ThingBuilder<NilPlus<ServientExtension>, ToExtend> {
        ThingBuilder::<NilPlus<ServientExtension>, ToExtend>::new(title)
    }
}

#[cfg(test)]
mod test {
    use wot_td::{builder::affordance::*, builder::data_schema::*, thing::FormOperation};

    use crate::advertise::ThingType;

    use super::*;

    #[test]
    fn build_servient() {
        let servient = Servient::builder("test")
            .finish_extend()
            .form(|f| {
                f.href("/ref")
                    .http_get(|| async { "Hello, World!" })
                    .op(FormOperation::ReadAllProperties)
            })
            .form(|f| {
                f.href("/ref2")
                    .http_get(|| async { "Hello, World! 2" })
                    .op(FormOperation::ReadAllProperties)
            })
            .build_servient()
            .unwrap();

        dbg!(&servient.router);
    }

    #[test]
    fn build_servient_property() {
        let servient = Servient::builder("test")
            .finish_extend()
            .property("hello", |b| {
                b.finish_extend_data_schema().null().form(|f| {
                    f.href("/hello")
                        .http_get(|| async { "Reading Hello, World!" })
                        .http_put(|| async { "Writing Hello, World!" })
                        .op(FormOperation::ReadProperty)
                        .op(FormOperation::WriteProperty)
                })
            })
            .build_servient()
            .unwrap();

        dbg!(&servient.router);
    }

    #[test]
    fn build_servient_action() {
        let servient = Servient::builder("test")
            .finish_extend()
            .action("hello", |b| {
                b.input(|i| i.finish_extend().number()).form(|f| {
                    f.href("/say_hello")
                        .http_post(|| async { "Saying Hello, World!" })
                })
            })
            .action("update", |b| {
                b.form(|f| {
                    f.href("/update_hello")
                        .http_patch(|| async { "Updating Hello, World!" })
                })
            })
            .action("delete", |b| {
                b.form(|f| {
                    f.href("/delete_hello")
                        .http_delete(|| async { "Goodbye, World!" })
                })
            })
            .build_servient()
            .unwrap();

        dbg!(&servient.router);
    }

    #[test]
    fn servient_setup() {
        let addr = "0.0.0.0:3000".parse().unwrap();
        let servient = Servient::builder("test me")
            .finish_extend()
            .http_bind(addr)
            .thing_type(ThingType::Directory)
            .build_servient()
            .unwrap();

        assert_eq!(servient.http_addr, addr);
        assert_eq!(servient.thing_type, ThingType::Directory);
    }
}
