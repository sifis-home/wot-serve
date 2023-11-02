use std::net::SocketAddr;

use crate::{
    advertise::{Advertiser, ThingType},
    hlist::*,
    servient::Servient,
};
use axum::{handler::Handler, response::Redirect, routing::MethodRouter, Router};
use tower_http::cors::*;

use datta::{Operator, UriTemplate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wot_td::{
    builder::{AdditionalExpectedResponseBuilder, FormBuilder, ThingBuilder},
    extend::{Extend, ExtendableThing},
    protocol::http,
    thing::FormOperation,
};

#[doc(hidden)]
/// ThingBuilder ExtendableThing used to build a Servient
///
/// It is not needed to know about it nor use it directly.
/// Instantiate a correct builder by calling [`Servient::builder`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServientExtension {
    /// Listening address
    #[serde(skip)]
    addr: Option<SocketAddr>,
    /// Thing type
    #[serde(skip)]
    thing_type: ThingType,
    #[serde(skip)]
    permissive_cors: bool,
}

impl Default for ServientExtension {
    fn default() -> Self {
        ServientExtension {
            addr: None,
            thing_type: ThingType::default(),
            permissive_cors: true,
        }
    }
}

#[doc(hidden)]
/// Form Extension
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Form {
    #[serde(skip)]
    method_router: MethodRouter,

    #[serde(flatten)]
    htv: http::Form,
}

impl ExtendableThing for ServientExtension {
    type InteractionAffordance = ();
    type PropertyAffordance = ();
    type ActionAffordance = ();
    type EventAffordance = ();
    type Form = Form;
    type ExpectedResponse = http::Response;
    type DataSchema = ();
    type ObjectSchema = ();
    type ArraySchema = ();
}

/// Extension trait for the [`Servient`] configuration.
pub trait ServientSettings {
    /// Bind the http server to addr
    fn http_bind(self, addr: SocketAddr) -> Self;
    /// Set the thing type to be advertised.
    fn thing_type(self, ty: ThingType) -> Self;
    /// Disable the default CORS settings.
    fn http_disable_permissive_cors(self) -> Self;
}

impl<O: ExtendableThing> ServientSettings for ThingBuilder<O, wot_td::builder::Extended>
where
    O: Holder<ServientExtension>,
{
    fn http_bind(mut self, addr: SocketAddr) -> Self {
        self.other.field_mut().addr = Some(addr);
        self
    }

    fn thing_type(mut self, ty: ThingType) -> Self {
        self.other.field_mut().thing_type = ty;
        self
    }

    fn http_disable_permissive_cors(mut self) -> Self {
        self.other.field_mut().permissive_cors = false;
        self
    }
}

/// Trait extension to build a [`Servient`] from an extended [`ThingBuilder`]
///
/// TODO: Add an example
pub trait BuildServient {
    /// Extension type for the [`Servient`] and underlying [`Thing`].
    ///
    /// [`Thing`]: wot_td::thing::Thing
    type Other: ExtendableThing;
    /// Build the configured [`Servient`].
    fn build_servient(self) -> Result<Servient<Self::Other>, Box<dyn std::error::Error>>;
}

fn uritemplate_to_axum(uri: &str) -> String {
    use datta::TemplateComponent::*;
    let t = UriTemplate::new(uri);
    let mut path = String::new();

    for component in t.components() {
        match component {
            Literal(ref l) => path.push_str(l),
            VarList(ref op, ref varspec) => match op {
                Operator::Null => {
                    assert_eq!(
                        varspec.len(),
                        1,
                        "more than one variable in the expression is not supported."
                    );
                    path.push(':');
                    path.push_str(&varspec[0].name);
                }
                Operator::Slash => {
                    for v in varspec {
                        path.push_str("/:");
                        path.push_str(&v.name);
                    }
                }
                Operator::Question | Operator::Hash => break,
                Operator::Ampersand | Operator::Dot | Operator::Semi | Operator::Plus => {
                    panic!("Unsupported operator")
                }
            },
        }
    }

    path
}

impl<O: ExtendableThing> BuildServient for ThingBuilder<O, wot_td::builder::Extended>
where
    O: Holder<ServientExtension>,
    O::Form: Holder<Form>,
    O: Serialize,
{
    type Other = O;

    /// Build the configured Servient
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

            let href = uritemplate_to_axum(&form.href);

            router = router.route(&href, route.method_router.clone());
        }

        // TODO: Figure out how to share the thing description and if we want to.
        let json = serde_json::to_value(&thing)?;

        // We serve The thing from the root
        router = router.route("/", axum::routing::get(move || async { axum::Json(json) }));

        // We redirect this path to / to support relative Forms with empty base
        // See: https://www.rfc-editor.org/rfc/rfc3986#section-5.1.3
        router = router.route(
            "/.well-known/wot",
            axum::routing::get(move || async { Redirect::to("/") }),
        );

        if thing.other.field_ref().permissive_cors {
            let cors = CorsLayer::new()
                .allow_methods(tower_http::cors::Any)
                .allow_origin(tower_http::cors::Any);
            router = router.layer(cors);
        }

        let sd = Advertiser::new()?;

        let name = {
            let name = thing
                .title
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();
            let uuid = Uuid::new_v4();

            format!("{}{}", name, uuid.as_simple())
        };

        let http_addr = thing
            .other
            .field_ref()
            .addr
            .unwrap_or_else(|| "0.0.0.0:8080".parse().unwrap());

        let thing_type = thing.other.field_ref().thing_type;

        Ok(Servient {
            name,
            thing,
            router,
            sd,
            http_addr,
            thing_type,
        })
    }
}

/// Extension trait to build http routes while assembling [`Form`] using the
/// extended [`FormBuilder`].
///
/// [`Form`]: wot_td::thing::Form
/// [`FormBuilder`]: wot_td::builder::FormBuilder
pub trait HttpRouter {
    /// Specialisation of [wot_td::builder::FormBuilder]
    type Target;
    /// Route GET requests to the given handler.
    fn http_get<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static;
    /// Route PUT requests to the given handler.
    fn http_put<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static;
    /// Route POST requests to the given handler.
    fn http_post<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static;
    /// Route PATCH requests to the given handler.
    fn http_patch<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static;
    /// Route DELETE requests to the given handler.
    fn http_delete<H, T>(self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static;
}

pub struct ServientFormBuilder<Other: ExtendableThing, Href, OtherForm, const HAS_OP: bool>(
    FormBuilder<Other, Href, OtherForm>,
);

impl<Other: ExtendableThing, OtherForm, const HAS_OP: bool>
    ServientFormBuilder<Other, (), OtherForm, HAS_OP>
{
    /// Create a new builder with the specified Href
    ///
    /// See [FormBuilder::href].
    #[inline]
    pub fn href(
        self,
        value: impl Into<String>,
    ) -> ServientFormBuilder<Other, String, OtherForm, HAS_OP> {
        ServientFormBuilder(self.0.href(value))
    }
}

impl<Other: ExtendableThing, Href, OtherForm, const HAS_OP: bool>
    ServientFormBuilder<Other, Href, OtherForm, HAS_OP>
{
    /// Set the security definitions that must be satisfied to access the resource
    ///
    /// See [FormBuilder::security].
    #[inline]
    pub fn security(self, value: impl Into<String>) -> Self {
        Self(self.0.security(value))
    }

    /// Set the authorization scope identifiers
    ///
    /// See [FormBuilder::scope]
    #[inline]
    pub fn scope(self, value: impl Into<String>) -> Self {
        Self(self.0.scope(value))
    }

    /// Adds an additional response to the form builder.
    ///
    /// See [FormBuilder::additional_response]
    #[inline]
    pub fn additional_response<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut AdditionalExpectedResponseBuilder) -> &mut AdditionalExpectedResponseBuilder,
    {
        Self(self.0.additional_response(f))
    }

    /// Extends the form, passing a closure that returns `T`.
    ///
    /// See [FormBuilder::ext_with]
    #[inline]
    pub fn ext_with<F, T>(self, f: F) -> ServientFormBuilder<Other, Href, OtherForm::Target, HAS_OP>
    where
        OtherForm: Extend<T>,
        F: FnOnce() -> T,
    {
        ServientFormBuilder(self.0.ext_with(f))
    }

    /// Extends the form with an additional element.
    ///
    /// See [FormBuilder::ext].
    #[inline]
    pub fn ext<T>(self, t: T) -> ServientFormBuilder<Other, Href, OtherForm::Target, HAS_OP>
    where
        OtherForm: Extend<T>,
    {
        ServientFormBuilder(self.0.ext(t))
    }
}

impl<Other: ExtendableThing, Href, OtherForm> ServientFormBuilder<Other, Href, OtherForm, false> {
    /// Set the form intended operation
    ///
    /// See [FormBuilder::op].
    pub fn op(self, new_op: FormOperation) -> ServientFormBuilder<Other, Href, OtherForm, true> {
        ServientFormBuilder(self.0.op(new_op))
    }
}

impl<Other: ExtendableThing, Href, OtherForm, const HAS_OP: bool>
    From<ServientFormBuilder<Other, Href, OtherForm, HAS_OP>>
    for FormBuilder<Other, Href, OtherForm>
{
    #[inline]
    fn from(value: ServientFormBuilder<Other, Href, OtherForm, HAS_OP>) -> Self {
        value.0
    }
}

impl<Other, Href, OtherForm> HttpRouter for FormBuilder<Other, Href, OtherForm>
where
    Other: ExtendableThing + Holder<ServientExtension>,
    OtherForm: Holder<Form>,
{
    type Target = ServientFormBuilder<Other, Href, OtherForm, false>;

    /// Route GET requests to the given handler.
    fn http_get<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        let f = self.other.field_mut();
        f.method_router = method_router.get(handler);
        f.htv.method_name = Some(http::Method::Get);
        ServientFormBuilder(self)
    }
    /// Route PUT requests to the given handler.
    fn http_put<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        let f = self.other.field_mut();
        f.method_router = method_router.put(handler);
        f.htv.method_name = Some(http::Method::Put);
        ServientFormBuilder(self)
    }
    /// Route POST requests to the given handler.
    fn http_post<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        let f = self.other.field_mut();
        f.method_router = method_router.post(handler);
        f.htv.method_name = Some(http::Method::Post);
        ServientFormBuilder(self)
    }
    /// Route PATCH requests to the given handler.
    fn http_patch<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        let f: &mut Form = self.other.field_mut();
        f.method_router = method_router.patch(handler);
        f.htv.method_name = Some(http::Method::Patch);
        ServientFormBuilder(self)
    }
    /// Route DELETE requests to the given handler.
    fn http_delete<H, T>(mut self, handler: H) -> Self::Target
    where
        H: Handler<T, (), axum::body::Body>,
        T: 'static,
    {
        let method_router = std::mem::take(&mut self.other.field_mut().method_router);
        let f: &mut Form = self.other.field_mut();
        f.method_router = method_router.delete(handler);
        f.htv.method_name = Some(http::Method::Delete);
        ServientFormBuilder(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn uritemplate(uri: &str, axum: &str) {
        let a = uritemplate_to_axum(uri);

        assert_eq!(&a, axum);
    }

    #[test]
    fn plain_uri() {
        uritemplate("/properties/on", "/properties/on");
    }

    #[test]
    fn hierarchical_uri() {
        uritemplate("/properties{/prop,sub}", "/properties/:prop/:sub");
    }

    #[test]
    fn templated_uri() {
        uritemplate("/actions/fade/{action_id}", "/actions/fade/:action_id");
    }

    #[test]
    fn query_uri() {
        uritemplate("/weather/{?lat,long}", "/weather/");
    }
}
