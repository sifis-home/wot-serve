use axum::{routing::MethodRouter, Router};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use wot_td::{
    builder::{ThingBuilder, ToExtend},
    extend::ExtendableThing,
    hlist::*,
    thing::Thing,
};

/// WoT Servient serving a Thing Description
pub struct Servient<Other: ExtendableThing = Nil> {
    pub thing: Thing<Other>,
    pub router: Router,
}

impl Servient<Nil> {
    pub fn builder(
        title: impl Into<String>,
    ) -> ThingBuilder<Cons<ServientExtension, Nil>, ToExtend> {
        ThingBuilder::<Nil, ToExtend>::new(title).ext(ServientExtension {})
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ServientExtension {}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Form {
    #[serde(skip)]
    method_router: MethodRouter,
}

impl From<MethodRouter> for Form {
    fn from(method_router: MethodRouter) -> Self {
        Self { method_router }
    }
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

trait SplitForm<'a> {
    fn split_form(&'a self) -> &'a Form;
}

impl<'a, T: 'a, U: 'a> SplitForm<'a> for Cons<T, U>
where
    <&'a Cons<T, U> as HListRef>::Target: NonEmptyHList<Last = &'a Form>,
    &'a U: HListRef,
{
    fn split_form(&'a self) -> &'a Form {
        let r = self.to_ref();

        r.split_last().0
    }
}

impl<O: ExtendableThing> BuildServient for ThingBuilder<O, wot_td::builder::Extended>
where
    O::Form: for<'a> SplitForm<'a>,
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
            let other = &form.other;

            let route = other.split_form();

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

#[cfg(test)]
mod test {
    use wot_td::thing::FormOperation;

    use super::{BuildServient, Servient};
    use axum::routing::get;

    #[test]
    fn build_servient() {
        let servient = Servient::builder("test")
            .finish_extend()
            .form(|f| {
                f.href("/ref")
                    .op(FormOperation::ReadAllProperties)
                    .ext(get(|| async { "Hello, World!" }).into())
            })
            .form(|f| {
                f.href("/ref2")
                    .op(FormOperation::ReadAllProperties)
                    .ext(get(|| async { "Hello, World! 2" }).into())
            })
            .build_servient()
            .unwrap();

        dbg!(&servient.router);
    }
}
