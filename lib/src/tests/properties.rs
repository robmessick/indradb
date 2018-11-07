use super::super::{Datastore, EdgeKey, Transaction, Type, Vertex, WithIdVertexQuery, WithKeyEdgeQuery, VertexQueryExt, EdgeQueryExt};
use serde_json::Value as JsonValue;
use util::generate_random_secret;
use uuid::Uuid;

pub fn should_handle_vertex_properties<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let t = Type::new("test_edge_type".to_string()).unwrap();
    let v = Vertex::new(t);
    trans.create_vertex(&v).unwrap();
    let name = format!("vertex-properties-{}", generate_random_secret(8));
    let q = WithIdVertexQuery::single(v.id).property(name);

    // Check to make sure there's no initial value
    let result = trans.get_vertex_properties(q.clone()).unwrap();
    assert_eq!(result.len(), 0);

    // Set and get the value as true
    trans.set_vertex_properties(q.clone(), &JsonValue::Bool(true)).unwrap();
    let result = trans.get_vertex_properties(q.clone()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, v.id);
    assert_eq!(result[0].value, JsonValue::Bool(true));

    // Set and get the value as false
    trans.set_vertex_properties(q.clone(), &JsonValue::Bool(false)).unwrap();
    let result = trans.get_vertex_properties(q.clone()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, v.id);
    assert_eq!(result[0].value, JsonValue::Bool(false));

    // Delete & check that it's deleted
    trans.delete_vertex_properties(q.clone()).unwrap();
    let result = trans.get_vertex_properties(q).unwrap();
    assert_eq!(result.len(), 0);
}

pub fn should_not_set_invalid_vertex_properties<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let q = WithIdVertexQuery::single(Uuid::default()).property("foo");
    trans.set_vertex_properties(q.clone(), &JsonValue::Null).unwrap();
    let result = trans.get_vertex_properties(q).unwrap();
    assert_eq!(result.len(), 0);
}

pub fn should_not_delete_invalid_vertex_properties<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let q = WithIdVertexQuery::single(Uuid::default()).property("foo");

    trans.delete_vertex_properties(q).unwrap();

    let v = Vertex::new(Type::new("foo".to_string()).unwrap());
    trans.create_vertex(&v).unwrap();

    let q = WithIdVertexQuery::single(v.id).property("foo");
    trans.delete_vertex_properties(q).unwrap();
}

pub fn should_handle_edge_properties<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let vertex_t = Type::new("test_edge_type".to_string()).unwrap();
    let outbound_v = Vertex::new(vertex_t.clone());
    let inbound_v = Vertex::new(vertex_t.clone());
    trans.create_vertex(&outbound_v).unwrap();
    trans.create_vertex(&inbound_v).unwrap();
    let edge_t = Type::new("test_edge_type".to_string()).unwrap();
    let key = EdgeKey::new(outbound_v.id, edge_t.clone(), inbound_v.id);
    let q = WithKeyEdgeQuery::single(key.clone()).property(format!("edge-properties-{}", generate_random_secret(8)));

    trans.create_edge(&key).unwrap();

    // Check to make sure there's no initial value
    let result = trans.get_edge_properties(q.clone()).unwrap();
    assert_eq!(result.len(), 0);

    // Set and get the value as true
    trans.set_edge_properties(q.clone(), &JsonValue::Bool(true)).unwrap();
    let result = trans.get_edge_properties(q.clone()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].key, key);
    assert_eq!(result[0].value, JsonValue::Bool(true));

    // Set and get the value as false
    trans.set_edge_properties(q.clone(), &JsonValue::Bool(false)).unwrap();
    let result = trans.get_edge_properties(q.clone()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].key, key);
    assert_eq!(result[0].value, JsonValue::Bool(false));

    // Delete & check that it's deleted
    trans.delete_edge_properties(q.clone()).unwrap();
    let result = trans.get_edge_properties(q.clone()).unwrap();
    assert_eq!(result.len(), 0);
}

pub fn should_not_set_invalid_edge_properties<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let key = EdgeKey::new(Uuid::default(), Type::new("foo".to_string()).unwrap(), Uuid::default());
    let q = WithKeyEdgeQuery::single(key).property("bar");
    trans.set_edge_properties(q.clone(), &JsonValue::Null).unwrap();
    let result = trans.get_edge_properties(q).unwrap();
    assert_eq!(result.len(), 0);
}

pub fn should_not_delete_invalid_edge_properties<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let key = EdgeKey::new(Uuid::default(), Type::new("foo".to_string()).unwrap(), Uuid::default());
    trans.delete_edge_properties(WithKeyEdgeQuery::single(key).property("bar")).unwrap();

    let outbound_v = Vertex::new(Type::new("foo".to_string()).unwrap());
    let inbound_v = Vertex::new(Type::new("foo".to_string()).unwrap());
    trans.create_vertex(&outbound_v).unwrap();
    trans.create_vertex(&inbound_v).unwrap();

    let key = EdgeKey::new(outbound_v.id, Type::new("baz".to_string()).unwrap(), inbound_v.id);
    trans.create_edge(&key).unwrap();
    trans.delete_edge_properties(WithKeyEdgeQuery::single(key).property("bleh")).unwrap();
}
