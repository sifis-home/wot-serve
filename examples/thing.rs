use wot_serve::servient::*;
use wot_td::builder::{
    affordance::BuildableInteractionAffordance, data_schema::SpecializableDataSchema,
};

#[tokio::main]
async fn main() {
    let servient = Servient::builder("TestThing")
        .finish_extend()
        .property("hello", |b| {
            b.ext(())
                .ext_interaction(())
                .ext_data_schema(())
                .finish_extend_data_schema()
                .form(|b| b.http_get(|| async { "Hello World!" }).href("/hello"))
                .string()
        })
        .build_servient()
        .unwrap();

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(servient.router.into_make_service())
        .await
        .unwrap();
}
