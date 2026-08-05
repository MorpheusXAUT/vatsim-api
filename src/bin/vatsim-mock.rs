use std::env;
use std::fs;
use std::process;

use vatsim_api::mock::MockServer;
use vatsim_api::mock::state::MockState;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut bind_addr = "127.0.0.1:8080".to_owned();
    let mut seed_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                i += 1;
                seed_path = Some(
                    args.get(i)
                        .unwrap_or_else(|| {
                            eprintln!("Error: --seed requires a file path argument");
                            process::exit(1);
                        })
                        .clone(),
                );
            }
            "--bind" => {
                i += 1;
                bind_addr = args
                    .get(i)
                    .unwrap_or_else(|| {
                        eprintln!("Error: --bind requires an address argument");
                        process::exit(1);
                    })
                    .clone();
            }
            other => {
                bind_addr = other.to_owned();
            }
        }
        i += 1;
    }

    let mut builder = MockServer::builder()
        .bind(&bind_addr)
        .security_headers(true);

    if let Some(path) = &seed_path {
        let data = fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Error reading seed file {path}: {e}");
            process::exit(1);
        });
        let state: MockState = serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("Error parsing seed file {path}: {e}");
            process::exit(1);
        });
        let entity_count =
            state.pilots.len() + state.controllers.len() + state.atis.len() + state.prefiles.len();
        let user_count = state.users.len();
        eprintln!("Loaded seed from {path} ({entity_count} entities, {user_count} users)");
        builder = builder.state(state);
    }

    let server = builder.build().await.unwrap_or_else(|e| {
        eprintln!("Failed to bind to {bind_addr}: {e}");
        process::exit(1);
    });

    let addr = server.local_addr().unwrap_or_else(|e| {
        eprintln!("Failed to get local address: {e}");
        process::exit(1);
    });
    eprintln!("Listening on http://{addr}");

    if let Err(e) = server.serve().await {
        eprintln!("Server error: {e}");
        process::exit(1);
    }
}
