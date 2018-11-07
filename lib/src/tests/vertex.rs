use super::super::{Datastore, Transaction, WithIdVertexQuery, RangeVertexQuery, VertexQueryExt, EdgeQueryExt};
use super::util::{create_edge_from, create_edges};
use models;
use std::collections::HashSet;
use std::u32;
use uuid::Uuid;

pub fn should_create_vertex_from_type<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let t = models::Type::new("test_vertex_type".to_string()).unwrap();
    trans.create_vertex_from_type(t).unwrap();
}

pub fn should_get_all_vertices<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let mut inserted_ids = create_vertices(&trans);

    let range = trans.get_vertices(RangeVertexQuery::new(u32::MAX)).unwrap();

    assert!(range.len() >= 5);

    let mut covered_ids: HashSet<Uuid> = HashSet::new();

    for vertex in &range {
        if let Ok(index) = inserted_ids.binary_search(&vertex.id) {
            assert_eq!(vertex.t, models::Type::new("test_vertex_type".to_string()).unwrap());
            inserted_ids.remove(index);
        }

        assert!(!covered_ids.contains(&vertex.id));
        covered_ids.insert(vertex.id);
    }
}

pub fn should_get_no_vertices_with_zero_limit<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    create_vertices(&trans);
    let range = trans.get_vertices(RangeVertexQuery::new(0)).unwrap();
    assert_eq!(range.len(), 0);
}

pub fn should_get_all_vertices_out_of_range<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    create_vertices(&trans);
    let range = trans.get_vertices(RangeVertexQuery::new(u32::MAX).start_id(Uuid::parse_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").unwrap())).unwrap();
    assert_eq!(range.len(), 0);
}

pub fn should_get_single_vertices<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let vertex_t = models::Type::new("test_vertex_type".to_string()).unwrap();
    let vertex = models::Vertex::new(vertex_t);
    trans.create_vertex(&vertex).unwrap();
    let range = trans.get_vertices(WithIdVertexQuery::single(vertex.id)).unwrap();
    assert_eq!(range.len(), 1);
    assert_eq!(range[0].id, vertex.id);
    assert_eq!(range[0].t.0, "test_vertex_type");
}

pub fn should_get_single_vertices_nonexisting<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let vertex_t = models::Type::new("test_vertex_type".to_string()).unwrap();
    let vertex = models::Vertex::new(vertex_t);
    trans.create_vertex(&vertex).unwrap();
    let range = trans.get_vertices(WithIdVertexQuery::single(Uuid::default())).unwrap();
    assert_eq!(range.len(), 0);
}

pub fn should_get_vertices<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let mut inserted_ids = create_vertices(&trans);

    let range = trans.get_vertices(WithIdVertexQuery::new(vec![inserted_ids[0], inserted_ids[1], inserted_ids[2], Uuid::default()])).unwrap();

    assert!(range.len() == 3);

    let mut covered_ids: HashSet<Uuid> = HashSet::new();

    for vertex in &range {
        if let Ok(index) = inserted_ids.binary_search(&vertex.id) {
            assert_eq!(vertex.t, models::Type::new("test_vertex_type".to_string()).unwrap());
            inserted_ids.remove(index);
        }

        assert!(!covered_ids.contains(&vertex.id));
        covered_ids.insert(vertex.id);
    }
}

pub fn should_get_vertices_piped<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let vertex_t = models::Type::new("test_vertex_type".to_string()).unwrap();
    let edge_t = models::Type::new("test_edge_type".to_string()).unwrap();

    let v = models::Vertex::new(vertex_t);
    trans.create_vertex(&v).unwrap();
    let inserted_id_2 = create_edge_from(&trans, v.id);

    // This query should get `inserted_id_2`
    let query_1 = WithIdVertexQuery::single(v.id).outbound(1).t(edge_t.clone()).inbound(1);
    let range = trans.get_vertices(query_1.clone()).unwrap();
    assert_eq!(range.len(), 1);
    assert_eq!(range[0].id, inserted_id_2);

    // This query should get `v`
    let query_2 = query_1.inbound(1).t(edge_t).outbound(1);
    let range = trans.get_vertices(query_2).unwrap();
    assert_eq!(range.len(), 1);
    assert_eq!(range[0], v);
}

pub fn should_delete_a_valid_vertex<D: Datastore>(datastore: &mut D) {
    let (outbound_id, _) = create_edges(datastore);
    let trans = datastore.transaction().unwrap();
    let q = WithIdVertexQuery::single(outbound_id);
    trans.delete_vertices(q.clone()).unwrap();
    let v = trans.get_vertices(q).unwrap();
    assert_eq!(v.len(), 0);
    let t = models::Type::new("test_edge_type".to_string()).unwrap();
    let count = trans
        .get_edge_count(outbound_id, Some(&t), models::EdgeDirection::Outbound)
        .unwrap();
    assert_eq!(count, 0);
}

pub fn should_not_delete_an_invalid_vertex<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    trans.delete_vertices(WithIdVertexQuery::single(Uuid::default())).unwrap();
}

pub fn should_get_a_vertex_count<D: Datastore>(datastore: &mut D) {
    let trans = datastore.transaction().unwrap();
    let vertex_t = models::Type::new("test_vertex_type".to_string()).unwrap();
    let v = models::Vertex::new(vertex_t);
    trans.create_vertex(&v).unwrap();
    let count = trans.get_vertex_count().unwrap();
    assert!(count >= 1);
}

fn create_vertices<T>(trans: &T) -> Vec<Uuid>
where
    T: Transaction,
{
    let t = models::Type::new("test_vertex_type".to_string()).unwrap();

    let vertices = vec![
        models::Vertex::new(t.clone()),
        models::Vertex::new(t.clone()),
        models::Vertex::new(t.clone()),
        models::Vertex::new(t.clone()),
        models::Vertex::new(t.clone()),
    ];

    for vertex in &vertices {
        trans.create_vertex(vertex).unwrap();
    }

    let mut vertex_ids: Vec<Uuid> = vertices.into_iter().map(|v| v.id).collect();
    vertex_ids.sort();
    vertex_ids
}
