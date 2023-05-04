use std::time::Duration;

use serde::{Deserialize, Serialize};
use wot_serve::servient::*;
use wot_td::{
    builder::{affordance::BuildableInteractionAffordance, data_schema::SpecializableDataSchema},
    extend::ExtendableThing,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct A {}

impl ExtendableThing for A {
    type InteractionAffordance = ();
    type PropertyAffordance = ();
    type ActionAffordance = ();
    type EventAffordance = ();
    type Form = ();
    type ExpectedResponse = ();
    type DataSchema = ();
    type ObjectSchema = ();
    type ArraySchema = ();
}

#[tokio::main]
async fn main() {
    let servient = Servient::builder("TestThing")
        .ext(A {})
        .finish_extend()
        .http_bind("127.0.0.1:8080".parse().unwrap())
        .property("hello", |b| {
            b.ext(())
                .ext_interaction(())
                .ext_data_schema(())
                .finish_extend_data_schema()
                .form(|b| {
                    b.ext(())
                        .http_get(|| async { "Hello World!" })
                        .href("/hello")
                })
                .string()
        })
        .action("say_hello", |b| {
            b.ext(())
                .ext_interaction(())
                .form(|b| {
                    b.ext(())
                        .http_post(|| async { "I'm saying hello" })
                        .href("/say_hello")
                })
                .input(|b| b.ext(()).finish_extend().null())
                .form(|b| {
                    b.ext(())
                        .href("/say_hello/{action_id}")
                        .http_get(|| async { "Checking ..." })
                        .op(wot_td::thing::FormOperation::QueryAction)
                })
                .form(|b| {
                    b.ext(())
                        .href("/say_hello/{action_id}")
                        .http_delete(|| async { "Canceling ..." })
                        .op(wot_td::thing::FormOperation::CancelAction)
                })
                .uri_variable("action_id", |b| b.ext(()).finish_extend().string())
        })
        .build_servient()
        .unwrap();

    eprintln!("Listening to 127.0.0.1:8080");
    dbg!(&servient.router);
    dbg!(&servient.thing);

    println!("Running the servient for 10 seconds.");
    let _ = tokio::time::timeout(Duration::from_secs(10), async {
        servient.serve().await.unwrap()
    })
    .await;
}
