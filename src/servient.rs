use axum::{handler::Handler, routing::MethodRouter, Router};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use wot_td::{
    builder::{FormBuilder, ThingBuilder, ToExtend},
    extend::{Extend, Extendable, ExtendableThing},
    hlist::*,
    thing::Thing,
};

/// WoT Servient serving a Thing Description
pub struct Servient<Other: ExtendableThing = Nil> {
    pub thing: Thing<Other>,
    pub router: Router,
}

impl Servient<Nil> {
    pub fn builder(title: impl Into<String>) -> ThingBuilder<NilPlus<ServientExtension>, ToExtend> {
        ThingBuilder::<NilPlus<ServientExtension>, ToExtend>::new(title)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ServientExtension {}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Form {
    #[serde(skip)]
    method_router: MethodRouter,
}

impl ExtendableThing for ServientExtension {
    type InteractionAffordance = ();
    type PropertyAffordance = ();
    type ActionAffordance = ();
    type EventAffordance = ();
    type Form = Form;
    type ExpectedResponse = ();
    type DataSchema = ();
    type ObjectSchema = ();
    type ArraySchema = ();
}

pub trait BuildServient {
    type Other: ExtendableThing;
    fn build_servient(self) -> Result<Servient<Self::Other>, Box<dyn std::error::Error>>;
}

impl<O: ExtendableThing> BuildServient for ThingBuilder<O, wot_td::builder::Extended>
where
    O: Holder<ServientExtension>,
    O::Form: Holder<Form>,
    O: Serialize,
{
    type Other = O;

    fn build_servient(self) -> Result<Servient<Self::Other>, Box<dyn std::error::Error>> {
        let thing = self.build()?;

        let mut router = Router::new();

        let thing_forms = thing.forms.iter().flat_map(|o| o.iter());
        let properties_forms = thing
            .properties
            .iter()
            .flat_map(|m| m.values().flat_map(|a| a.interaction.forms.iter()));
        let actions_forms = thing
            .actions
            .iter()
            .flat_map(|m| m.values().flat_map(|a| a.interaction.forms.iter()));
        let events_forms = thing
            .events
            .iter()
            .flat_map(|m| m.values().flat_map(|a| a.interaction.forms.iter()));

        for form in thing_forms
            .chain(properties_forms)
            .chain(actions_forms)
            .chain(events_forms)
        {
            let route = form.other.field_ref();

            router = router.route(&form.href, route.method_router.clone());
        }

        // TODO: Figure out how to share the thing description and if we want to.
        let json = serde_json::to_value(&thing)?;

        router = router.route(
            "/.well-known/wot",
            axum::routing::get(move || async { axum::Json(json) }),
        );

        Ok(Servient { thing, router })
    }
}

pub trait HttpRouter {
    type Target;
    fn http_get<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static;
    fn http_put<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static;
    fn http_post<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static;
    fn http_patch<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static;
    fn http_delete<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static;
}

impl<Other, Href, OtherForm> HttpRouter for FormBuilder<Other, Href, OtherForm>
where
    Other: ExtendableThing + Holder<ServientExtension>,
    OtherForm: Holder<Form>,
{
    type Target = FormBuilder<Other, Href, OtherForm>;
    fn http_get<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        self.other.field_mut().method_router = method_router.get(handler);
        self
    }
    fn http_put<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        self.other.field_mut().method_router = method_router.put(handler);
        self
    }
    fn http_post<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        self.other.field_mut().method_router = method_router.post(handler);
        self
    }
    fn http_patch<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        self.other.field_mut().method_router = method_router.patch(handler);
        self
    }
    fn http_delete<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        self.other.field_mut().method_router = method_router.delete(handler);
        self
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NilPlus<T> {
    #[serde(flatten)]
    pub field: T,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsPlus<T, U, V> {
    #[serde(flatten)]
    pub field: T,
    cons: Cons<U, V>,
}

impl<T, U, V> ConsPlus<T, U, V> {
    pub fn cons<Z>(self, value: Z) -> ConsPlus<T, Z, Cons<U, V>> {
        let Self { field, cons } = self;

        ConsPlus {
            field,
            cons: cons.cons(value),
        }
    }
}

impl<T> NilPlus<T> {
    pub fn cons<V>(self, value: V) -> ConsPlus<T, V, Nil> {
        let Self { field } = self;
        ConsPlus {
            field,
            cons: Nil::cons(value),
        }
    }
}

impl<T: Default> Extendable for NilPlus<T> {
    type Empty = NilPlus<T>;

    fn empty() -> Self {
        NilPlus {
            field: Default::default(),
        }
    }
}

impl<T: Default, U, V> Extendable for ConsPlus<T, U, V> {
    type Empty = NilPlus<T>;

    fn empty() -> Self::Empty {
        NilPlus {
            field: Default::default(),
        }
    }
}

impl<T, U> Extend<U> for NilPlus<T> {
    type Target = ConsPlus<T, U, Nil>;

    fn ext(self, u: U) -> Self::Target {
        let Self { field } = self;

        ConsPlus {
            field,
            cons: Nil::cons(u),
        }
    }
}

impl<T, U, V, Z> Extend<Z> for ConsPlus<T, U, V> {
    type Target = ConsPlus<T, Z, Cons<U, V>>;

    fn ext(self, t: Z) -> Self::Target {
        self.cons(t)
    }
}

impl<T: ExtendableThing> ExtendableThing for NilPlus<T> {
    type InteractionAffordance = NilPlus<T::InteractionAffordance>;
    type PropertyAffordance = NilPlus<T::PropertyAffordance>;
    type ActionAffordance = NilPlus<T::ActionAffordance>;
    type EventAffordance = NilPlus<T::EventAffordance>;
    type Form = NilPlus<T::Form>;
    type ExpectedResponse = NilPlus<T::ExpectedResponse>;
    type DataSchema = NilPlus<T::DataSchema>;
    type ObjectSchema = NilPlus<T::ObjectSchema>;
    type ArraySchema = NilPlus<T::ArraySchema>;
}

impl<T, U, V> ExtendableThing for ConsPlus<T, U, V>
where
    T: ExtendableThing,
    U: ExtendableThing,
    V: ExtendableThing,
{
    type InteractionAffordance =
        ConsPlus<T::InteractionAffordance, U::InteractionAffordance, V::InteractionAffordance>;
    type PropertyAffordance =
        ConsPlus<T::PropertyAffordance, U::PropertyAffordance, V::InteractionAffordance>;
    type ActionAffordance = ConsPlus<T::ActionAffordance, U::ActionAffordance, V::ActionAffordance>;
    type EventAffordance = ConsPlus<T::EventAffordance, U::EventAffordance, V::EventAffordance>;
    type Form = ConsPlus<T::Form, U::Form, V::Form>;
    type ExpectedResponse = ConsPlus<T::ExpectedResponse, U::ExpectedResponse, V::ExpectedResponse>;
    type DataSchema = ConsPlus<T::DataSchema, U::DataSchema, V::DataSchema>;
    type ObjectSchema = ConsPlus<T::ObjectSchema, U::ObjectSchema, V::ObjectSchema>;
    type ArraySchema = ConsPlus<T::ArraySchema, U::ArraySchema, U::ArraySchema>;
}

trait Holder<T> {
    fn field_ref(&self) -> &T;
    fn field_mut(&mut self) -> &mut T;
}

impl<T> Holder<T> for NilPlus<T> {
    fn field_ref(&self) -> &T {
        &self.field
    }

    fn field_mut(&mut self) -> &mut T {
        &mut self.field
    }
}

impl<T, U, V> Holder<T> for ConsPlus<T, U, V> {
    fn field_ref(&self) -> &T {
        &self.field
    }

    fn field_mut(&mut self) -> &mut T {
        &mut self.field
    }
}

#[cfg(test)]
mod test {
    use wot_td::{builder::affordance::*, builder::data_schema::*, thing::FormOperation};

    use crate::servient::HttpRouter;

    use super::{BuildServient, Servient};

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
}
