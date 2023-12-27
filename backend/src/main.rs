use std::time::Duration;
use clap::Parser;

use axum::http::header;
use axum::response::Html;
use rand::prelude::*;
use dashmap::DashMap;
use etherparse::{SlicedPacket, VlanSlice};
use pcap::Device;
use pcap::Capture;
use axum::{Router, routing::get, Json, http::Method, extract::{ws::WebSocket, State, WebSocketUpgrade, Path}, response::IntoResponse};
use tokio::sync::mpsc::Sender;
use tower_http::cors::{CorsLayer, Any};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Data{
    src: String,
    dst: String,
    vlan: Option<u16>
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DashMap<usize, Sender<Data>>,
    pub cards: Vec<IfCard>
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("8080"))]
    web: String,
    #[arg(short, long)]
    interface: Option<String>
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(interface) = args.interface {
        println!("[SYS] Luisteren naar interface: {} via cmd", interface);
        scan_cmd(interface);
    }

    println!("[SYS] Starting web interface {}", args.web);

    let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST])
    .allow_origin(Any);

    let state = AppState { db: DashMap::new(), cards: get_devices() };

    let app = Router::new()
        .route("/devices", get(devices))
        .route("/ws/:id", get(ws))
        .route("/", get(|| async { Html(include_str!("../dist/index.html") )}))
        .route("/assets/index-FNWSemnn.css", get(|| async {
            let css = include_str!("../dist/assets/index-FNWSemnn.css");
            ([(header::CONTENT_TYPE, "text/css")], css)
        }))
        .route("/assets/index-bf28pown.js", get(|| async {
            let js = include_str!("../dist/assets/index-bf28pown.js");
            ([(header::CONTENT_TYPE, "application/javascript")], js)
        }))
        .layer(cors)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IfCard {
    pub id: usize,
    pub name: String,
}



async fn ws(ws: WebSocketUpgrade, State(s): State<AppState>, Path(id): Path<usize> ) -> impl IntoResponse {
    ws.on_upgrade(move |ws| async move { ws_handler(ws, s, id).await })
}

async fn ws_handler( mut socket: WebSocket, s: AppState, id: usize) {
    let card = s.cards.iter().find(|c| c.id == id).unwrap().name.clone();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Data>(15);

    tokio::spawn(async move { scan( card, tx).await });

    while let Some(data) = rx.recv().await {
        let _ = socket.send(axum::extract::ws::Message::Text(
                serde_json::to_string(&data).unwrap()
                )).await;
    }
}

async fn scan( card: String, sender: Sender<Data> ){
    println!("[SYS] scanning: {card}");

    let d = Device::list().unwrap();
    let device = d.into_iter().find(|d| d.name == card).expect("Device not found");

    let mut cap = Capture::from_device(device).unwrap().immediate_mode(true).promisc(true).open().unwrap();
    while let Ok(packet) = cap.next_packet() {
        let packet = SlicedPacket::from_ethernet(packet.data).unwrap();

        let eth = packet.link.unwrap().to_header();
        let s = eth.source
                    .iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(":");

        let d = eth.destination
                    .iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(":");

        sender.send(Data {
            src: s,
            dst: d,
            vlan: packet.vlan.map(|v|
                match v {
                    VlanSlice::SingleVlan(x) => x.vlan_identifier(),
                    VlanSlice::DoubleVlan(x) => x.outer().vlan_identifier(),
                }
            )
        }).await.unwrap();
    }
}

fn scan_cmd( card: String){
    let d = Device::list().unwrap();
    let device = d.into_iter().find(|d| d.name == card).expect("Device not found");

    let mut cap = Capture::from_device(device).unwrap().immediate_mode(true).promisc(true).open().unwrap();
    while let Ok(packet) = cap.next_packet() {
        let packet = SlicedPacket::from_ethernet(packet.data).unwrap();

        let eth = packet.link.unwrap().to_header();
        let s = eth.source
                    .iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(":");

        let d = eth.destination
                    .iter()
                    .map(|byte| format!("{:02X}", byte))
                    .collect::<Vec<String>>()
                    .join(":");


        let vlan = packet.vlan.map(|v|
                match v {
                    VlanSlice::SingleVlan(x) => x.vlan_identifier(),
                    VlanSlice::DoubleVlan(x) => x.outer().vlan_identifier(),
                }
        );

        if let Some(vlan) = vlan {
            println!("[SYS] src: {s} -> dst: {d} vlan id: {}", vlan);
        }
    }
}


fn get_devices() -> Vec<IfCard> {
    let d = Device::list().unwrap();
    d.into_iter()
        .enumerate()
        .map(|(id, d)| IfCard{ id, name: d.name })
        .collect()
}

async fn devices() -> Json<Vec<IfCard>> {
    let d = get_devices();
    Json(d)
}

