use axum::routing::MethodRouter;
use serde::{Deserialize, Serialize};
use wot_td::extend::{Extend, Extendable, ExtendableThing};

/// WoT Servient serving a Thing Description
pub struct Servient {}

#[derive(Debug, Default)]
struct ServientExtension;

#[derive(Default, Serialize, Deserialize)]
struct Form {
    #[serde(skip)]
    method_router: MethodRouter,
}

impl Extendable for Form {
    type Empty = Form;
    fn empty() -> Self::Empty {
        Form::default()
    }
}

impl Extend<MethodRouter> for Form {
    type Target = Form;
    fn ext(mut self, t: MethodRouter) -> Self::Target {
        self.method_router = t;
        self
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

#[cfg(test)]
mod test {
    use wot_td::builder::ThingBuilder;
    use wot_td::thing::FormOperation;

    use super::ServientExtension;
    use axum::{routing::get, Router};

    #[test]
    fn build_server() {
        let t = ThingBuilder::<ServientExtension, _>::new("test")
            .finish_extend()
            .form(|f| {
                f.href("/ref")
                    .op(FormOperation::ReadAllProperties)
                    .ext(get(|| async { "Hello, World!" }))
            })
            .build()
            .unwrap();

        let mut router = Router::new();

        for form in t.forms.as_ref().unwrap().iter() {
            router = router.route(&form.href, form.other.method_router.clone());
        }
    }
}
