use axum::http::StatusCode;
use chord_generator::{generate::generate, wave::Wave};
// use chord_generator::morph::morph;
use serde::Deserialize;
use axum::{
    Router, extract::Query, routing::get,
};

#[derive(Deserialize)]
struct GenerateParams {
    wave_type: Wave,
    notes: String,
    duration: f64
}

// http://localhost:3000/generate?wave_type=Sine&notes=C,E,G&duration=2.5
async fn generate_chord(Query(GenerateParams {
    wave_type,
    notes,
    duration
}): Query<GenerateParams>) -> Result<String, StatusCode> {
    let notes: Vec<String> = notes
        .split(',')
        .map(str::to_string)
        .collect();

    match generate(wave_type, notes, duration) {
        Ok(_result) => Ok("generated WAV file".to_string()),
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Router::new()
    .route("/generate", get(generate_chord));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}