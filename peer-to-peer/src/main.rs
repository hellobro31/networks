use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    mplex,
    noise::{Keypair as NoiseKeypair, NoiseConfig, X25519Spec},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    PeerId, NetworkBehaviour, Transport,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use libp2p::futures::StreamExt;
use tokio::{
    fs::File,
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::mpsc,
    select,
    signal,
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use std::env;
use tokio::io::AsyncWriteExt;
use futures_util::sink::SinkExt;
use std::collections::HashMap;
use log::{info, error};
use serde_json;

const STORAGE_FILE_PATH: &str = "../recipes.json";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Recipes = Vec<Recipe>;

static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("recipes"));

#[derive(Debug, Serialize, Deserialize)]
struct Recipe {
    id: usize,
    name: String,
    ingredients: String,
    instructions: String,
    public: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum ListMode {
    ALL,
    One(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct ListRequest {
    mode: ListMode,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListResponse {
    mode: ListMode,
    data: Recipes,
    receiver: String,
}

#[derive(NetworkBehaviour)]
struct RecipeBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
    #[behaviour(ignore)]
    response_sender: mpsc::UnboundedSender<ListResponse>,
}

async fn read_local_recipes() -> Result<Recipes> {
    let path = STORAGE_FILE_PATH; // Ensure this path is set to your 'recipes.json' file
    let mut file = match File::open(path).await {
        Ok(file) => file,
        Err(e) => return Err(Box::new(e)),
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents).await {
        return Err(Box::new(e));
    }

    match serde_json::from_str(&contents) {
        Ok(recipes) => Ok(recipes),
        Err(e) => Err(Box::new(e)),
    }
}

async fn write_local_recipes(recipes: &Recipes) -> Result<()> {
    let path = STORAGE_FILE_PATH; // Ensure this path is set to your 'recipes.json' file
    let temp_path = format!("{}.tmp", path);

    let json = match serde_json::to_string(&recipes) {
        Ok(json) => json,
        Err(e) => return Err(Box::new(e)),
    };

    let mut file = match File::create(&temp_path).await {
        Ok(file) => file,
        Err(e) => return Err(Box::new(e)),
    };

    if let Err(e) = file.write(json.as_bytes()).await {
        return Err(Box::new(e));
    }

    // Ensure data is flushed and file is synced to disk
    if let Err(e) = file.sync_all().await {
        return Err(Box::new(e));
    }

    // Rename temp file to permanent file path
    match tokio::fs::rename(&temp_path, path).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}



impl NetworkBehaviourEventProcess<FloodsubEvent> for RecipeBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        match event {
            FloodsubEvent::Message(message) => {
                if let Ok(request) = serde_json::from_slice::<ListRequest>(&message.data) {
                    match request.mode {
                        ListMode::ALL => {
                            info!("Received list all recipes request from {}", message.source);
                            tokio::spawn(async move {
                                let recipes = match read_local_recipes().await {
                                    Ok(recipes) => recipes,
                                    Err(_) => vec![],
                                };
                                let response = ListResponse {
                                    mode: ListMode::ALL,
                                    data: recipes,
                                    receiver: message.source.to_string(),
                                };
                                let _response_json = serde_json::to_string(&response).unwrap();
                            });
                        },
                        ListMode::One(peer_id) => {
                            if &peer_id == &PEER_ID.to_string() {
                                info!("Received specific list request from {}", message.source);
                            }
                        },
                    }
                }
            },
            _ => (),
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for RecipeBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(peers) => {
                for (peer_id, _addr) in peers {
                    info!("Discovered new peer: {:?}", peer_id);
                    self.floodsub.add_node_to_partial_view(peer_id);
                }
            },
            MdnsEvent::Expired(peers) => {
                for (peer_id, _addr) in peers {
                    info!("Peer expired: {:?}", peer_id);
                    self.floodsub.remove_node_from_partial_view(&peer_id);
                }
            }
        }
    }
}

async fn add_new_recipe(recipe: Recipe) -> Result<()> {
    let mut recipes = read_local_recipes().await?;
    recipes.push(recipe);
    write_local_recipes(&recipes).await?;
    Ok(())
}

async fn publish_recipe(recipe_id: usize) -> Result<()> {
    let mut recipes = read_local_recipes().await?;
    if let Some(recipe) = recipes.iter_mut().find(|r| r.id == recipe_id) {
        recipe.public = true;
        write_local_recipes(&recipes).await?;
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Recipe not found")))
    }
}

async fn fetch_public_recipes() -> Result<Recipes> {
    let recipes = read_local_recipes().await?;
    Ok(recipes.into_iter().filter(|r| r.public).collect())
}

// // Function to serialize peer ids
// fn serialize_peer_ids(peers: Vec<PeerId>) -> Vec<String> {
//     peers.into_iter()
//         .map(|peer_id| peer_id.to_base58())
//         .collect()
// }


async fn handle_websocket_connection(raw_stream: TcpStream) {
    let ws_stream = accept_async(raw_stream).await.expect("Failed to accept WebSocket connection");
    let (mut write, mut read) = ws_stream.split();

    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                info!("Received message via WebSocket: {}", text);
                let parts: Vec<&str> = text.split_whitespace().collect();
                match parts[0] {
                    "ls" => match parts[1] {
                        "r" if parts[2] == "all" => {
                            let recipes = fetch_public_recipes().await.unwrap_or_else(|_| vec![]);
                            let response = serde_json::to_string(&recipes).unwrap();
                            let send_message = Message::Text(response);
                            write.send(send_message).await.expect("Failed to send message");
                        },
                        _ => {},
                    },
                    "publish" if parts[1] == "r" => {
                        let id = parts[2].parse::<usize>().unwrap();
                        if let Ok(_) = publish_recipe(id).await {
                            let mut data = HashMap::new();
                            data.insert("status", "Recipe published");
                            let response = serde_json::to_string(&data).unwrap();
                            let send_message = Message::Text(response);
                            write.send(send_message).await.expect("Failed to send message");
                        }
                    },
                    "ls" if parts[1] == "p" => {
                        // Placeholder for actual peer listing function
                        // This function needs to be asynchronous and capable of being awaited
                        // let peer_list = list_active_peers(&mut swarm).await?;
                        // let response = serde_json::to_string(&peer_list).unwrap();
                        // let send_message = Message::Text(response);
                        // write.send(send_message).await.expect("Failed to send message");
                    },
                    "create" if parts[1] == "r" => {
                        let data: Vec<&str> = parts[2].split('|').collect();
                        if data.len() == 3 {
                            let recipe = Recipe {
                                id: 0, // ID will be generated
                                name: data[0].to_string(),
                                ingredients: data[1].to_string(),
                                instructions: data[2].to_string(),
                                public: false,
                            };
                            if let Ok(_) = add_new_recipe(recipe).await {
                                let mut data2 = HashMap::new();
                                data2.insert("status", "Recipe created");
                                let response = serde_json::to_string(&data2).unwrap();
                                let send_message = Message::Text(response);
                                write.send(send_message).await.expect("Failed to send message");
                            }
                        }
                    },
                    _ => {
                        let mut data3 = HashMap::new();
                        data3.insert("error", "Invalid command");
                        let error_message = serde_json::to_string(&data3).unwrap();
                        let send_message = Message::Text(error_message);
                        write.send(send_message).await.expect("Failed to send message");
                    }
                }
            }
            Ok(_) => {},
            Err(e) => {
                error!("WebSocket error: {:?}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    // Obtain port from command line arguments, default to 9000 if not provided
    let args: Vec<String> = env::args().collect();
    let port = args.get(1).unwrap_or(&"9000".to_string()).parse::<u16>().unwrap_or(9000);
    
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Server is listening on {}", addr);

    let (response_sender, _response_rcv) = mpsc::unbounded_channel();

    let auth_keys = NoiseKeypair::<X25519Spec>::new().into_authentic(&KEYS)?;
    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour = RecipeBehaviour {
        floodsub: Floodsub::new(PEER_ID.clone()),
        mdns: Mdns::new(Default::default()).await?,
        response_sender,
    };

    let mut swarm = SwarmBuilder::new(transp, behaviour, PEER_ID.clone()).build();
    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        select! {
            conn = listener.accept() => {
                let (stream, _) = conn?;
                tokio::spawn(handle_websocket_connection(stream));
            },
            event = swarm.select_next_some() => {
                info!("Unhandled Swarm Event: {:?}", event);
            },
            _ = signal::ctrl_c() => {
                info!("CTRL-C received, shutting down");
                break;
            },
        }
    }

    Ok(())
}