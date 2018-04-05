pub use indradb::tests;
pub use regex::Regex;
use indradb;
use serde::Deserialize;
use serde_json::value::Value as JsonValue;
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;
use std::sync::Arc;
use grpcio::{Environment, ChannelBuilder, ClientDuplexSender, ClientDuplexReceiver, WriteFlags};
use futures::{Future, Sink, Stream};
use futures::stream::Wait;
use futures::sink::Send;

use super::request::*;
use super::vertices::Vertex;
use super::response::TransactionResponse;
use super::service_grpc::IndraDbClient;

const START_PORT: usize = 27615;

lazy_static! {
    static ref PORT: AtomicUsize = AtomicUsize::new(START_PORT);
}

fn create_client(port: usize) -> IndraDbClient {
    let env = Arc::new(Environment::new(1));
    let channel = ChannelBuilder::new(env).connect(&format!("127.0.0.1:{}", port));
    IndraDbClient::new(channel)
}

#[derive(Debug)]
pub struct GrpcDatastore {
    port: usize,
    server: Child,
}

impl GrpcDatastore {
    fn default() -> Self {
        let port = PORT.fetch_add(1, Ordering::SeqCst);

        let mut envs = HashMap::new();
        envs.insert("PORT", port.to_string());

        let server = Command::new("../target/debug/indradb-server")
            .envs(envs)
            .spawn()
            .expect("Server failed to start");

        let client = create_client(port);

        for _ in 0..5 {
            let request = PingRequest::new();

            if let Ok(response) = client.ping(&PingRequest::new()) {
                if response.get_ok() {
                    return Self {
                        port: port,
                        server: server,
                    };
                }
            }

            sleep(Duration::from_secs(1));
        }

        panic!("Server failed to initialize after a few seconds");
    }
}

impl Drop for GrpcDatastore {
    fn drop(&mut self) {
        if let Err(err) = self.server.kill() {
            panic!(format!("Could not kill server instance: {}", err))
        }
    }
}

impl indradb::Datastore<GrpcTransaction> for GrpcDatastore {
    fn transaction(&self) -> Result<GrpcTransaction, indradb::Error> {
        let client = create_client(self.port);
        Ok(GrpcTransaction::new(client))
    }
}

pub struct GrpcTransaction {
    client: IndraDbClient,
    sink: Option<ClientDuplexSender<TransactionRequest>>,
    receiver: Wait<ClientDuplexReceiver<TransactionResponse>>
}

impl GrpcTransaction {
    fn new(client: IndraDbClient) -> Self {
        let (sink, receiver) = client.transaction().unwrap();
        
        GrpcTransaction {
            client: client,
            sink: Some(sink),
            receiver: receiver.wait()
        }
    }

    fn request(&mut self, req: TransactionRequest) -> TransactionResponse {
        let sink: ClientDuplexSender<TransactionRequest> = self.sink.take().unwrap();
        self.sink = Some(sink.send((req, WriteFlags::default())).wait().unwrap());
        self.receiver.next().unwrap().unwrap()
    }
}

impl indradb::Transaction for GrpcTransaction {
    fn create_vertex(&mut self, v: &indradb::Vertex) -> Result<bool, indradb::Error> {
        let mut inner = CreateVertexRequest::new();
        inner.set_vertex(Vertex::from(v.clone()));
        let mut request = TransactionRequest::new();
        request.set_create_vertex(inner);
        let response = self.request(request);

        if response.has_error() {
            Err(response.get_error().into())
        } else {
            Ok(response.get_ok())
        }
    }

    fn create_vertex_from_type(&self, t: indradb::Type) -> Result<Uuid, indradb::Error> {
        // self.request(&json!({
        //     "action": "create_vertex_from_type",
        //     "type": t
        // }))
        unimplemented!();
    }

    fn get_vertices(&self, q: &indradb::VertexQuery) -> Result<Vec<indradb::Vertex>, indradb::Error> {
        // self.request(&json!({
        //     "action": "get_vertices",
        //     "query": q
        // }))
        unimplemented!();
    }

    fn delete_vertices(&self, q: &indradb::VertexQuery) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "delete_vertices",
        //     "query": q
        // }))
        unimplemented!();
    }

    fn get_vertex_count(&self) -> Result<u64, indradb::Error> {
        // self.request(&json!({
        //     "action": "get_vertex_count"
        // }))
        unimplemented!();
    }

    fn create_edge(&self, e: &indradb::EdgeKey) -> Result<bool, indradb::Error> {
        // self.request(&json!({
        //     "action": "create_edge",
        //     "key": e,
        // }))
        unimplemented!();
    }

    fn get_edges(&self, q: &indradb::EdgeQuery) -> Result<Vec<indradb::Edge>, indradb::Error> {
        // self.request(&json!({
        //     "action": "get_edges",
        //     "query": q
        // }))
        unimplemented!();
    }

    fn delete_edges(&self, q: &indradb::EdgeQuery) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "delete_edges",
        //     "query": q
        // }))
        unimplemented!();
    }

    fn get_edge_count(&self, id: Uuid, type_filter: Option<&indradb::Type>, direction: indradb::EdgeDirection) -> Result<u64, indradb::Error> {
        // self.request(&json!({
        //     "action": "get_edge_count",
        //     "id": id,
        //     "type_filter": type_filter,
        //     "direction": direction
        // }))
        unimplemented!();
    }

    fn get_global_metadata(&self, name: &str) -> Result<Option<JsonValue>, indradb::Error> {
        // self.request(&json!({
        //     "action": "get_global_metadata",
        //     "name": name
        // }))
        unimplemented!();
    }

    fn set_global_metadata(&self, name: &str, value: &JsonValue) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "set_global_metadata",
        //     "name": name,
        //     "value": value
        // }))
        unimplemented!();
    }

    fn delete_global_metadata(&self, name: &str) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "delete_global_metadata",
        //     "name": name
        // }))
        unimplemented!();
    }

    fn get_vertex_metadata(&self, q: &indradb::VertexQuery, name: &str) -> Result<Vec<indradb::VertexMetadata>, indradb::Error> {
        // self.request(&json!({
        //     "action": "get_vertex_metadata",
        //     "query": q,
        //     "name": name
        // }))
        unimplemented!();
    }

    fn set_vertex_metadata(&self, q: &indradb::VertexQuery, name: &str, value: &JsonValue) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "set_vertex_metadata",
        //     "query": q,
        //     "name": name,
        //     "value": value
        // }))
        unimplemented!();
    }

    fn delete_vertex_metadata(&self, q: &indradb::VertexQuery, name: &str) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "delete_vertex_metadata",
        //     "query": q,
        //     "name": name
        // }))
        unimplemented!();
    }

    fn get_edge_metadata(&self, q: &indradb::EdgeQuery, name: &str) -> Result<Vec<indradb::EdgeMetadata>, indradb::Error> {
        // self.request(&json!({
        //     "action": "get_edge_metadata",
        //     "query": q,
        //     "name": name
        // }))
        unimplemented!();
    }

    fn set_edge_metadata(&self, q: &indradb::EdgeQuery, name: &str, value: &JsonValue) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "set_edge_metadata",
        //     "query": q,
        //     "name": name,
        //     "value": value
        // }))
        unimplemented!();
    }

    fn delete_edge_metadata(&self, q: &indradb::EdgeQuery, name: &str) -> Result<(), indradb::Error> {
        // self.request(&json!({
        //     "action": "delete_edge_metadata",
        //     "query": q,
        //     "name": name
        // }))
        unimplemented!();
    }
}

full_test_impl!({ GrpcDatastore::default() });
