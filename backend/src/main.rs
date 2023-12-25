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

#[tokio::main]
async fn main() {

    let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST])
    .allow_origin(Any);

    let state = AppState { db: DashMap::new(), cards: get_devices() };

    let app = Router::new()
        .route("/devices", get(devices))
        .route("/ws/:id", get(ws))
        .layer(cors)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();

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
    println!("scanning: {card}");

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
            )}
        ).await.unwrap();
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

