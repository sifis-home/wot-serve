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
        .build_servient()
        .unwrap();

    eprintln!("Listening to 127.0.0.1:8080");
    dbg!(&servient.router);

    println!("Running the servient for 2 seconds.");
    let _ = tokio::time::timeout(Duration::from_secs(2), async {
        axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
            .serve(servient.router.into_make_service())
            .await
            .unwrap();
    })
    .await;
}
