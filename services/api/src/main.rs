use salvo::cors::Cors;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate serde_json;

#[derive(Deserialize, Serialize)]
struct IncomingHeaders {
    host: String,
}

#[handler]
async fn info(req: &mut Request, res: &mut Response) {
    let headers: IncomingHeaders = req.parse_headers().unwrap();
    let json = json!({
        "@": "Saerro Listening Post",
        "@GitHub": "https://github.com/genudine/saerro",
        "@Disclaimer": "Genudine Dynamics is not responsible for any damages caused by this software. Use at your own risk.",
        "@Support": "#api-dev in https://discord.com/servers/planetside-2-community-251073753759481856",
        "Worlds": {
            "Connery": format!("https://{}/w/1", headers.host),
            "Miller": format!("https://{}/w/10", headers.host),
            "Cobalt": format!("https://{}/w/13", headers.host),
            "Emerald": format!("https://{}/w/17", headers.host),
            "Jaeger": format!("https://{}/w/19", headers.host),
            "SolTech": format!("https://{}/w/40", headers.host),
            "Genudine": format!("https://{}/w/1000", headers.host),
            "Ceres": format!("https://{}/w/2000", headers.host),
        },
        "All Worlds": format!("https://{}/m/?ids=1,10,13,17,19,40,1000,2000", headers.host),
    });

    res.render(serde_json::to_string_pretty(&json).unwrap());
}

#[tokio::main]
async fn main() {
    let cors_handler = Cors::builder()
        .allow_any_origin()
        .allow_method("GET")
        .build();

    let router = Router::new().hoop(cors_handler).get(info);
    Server::new(TcpListener::bind("127.0.0.1:7878"))
        .serve(router)
        .await;
}
