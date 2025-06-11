use crate::utils::create_node_key;
use anyhow::Result;
use lazy_static::lazy_static;
use neo4rs::{query, ConfigBuilder, Graph as Neo4jConnection};
use serde_json;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Once},
};
use tracing::{debug, info};
use uuid::Uuid;

use super::*;

lazy_static! {
    static ref CONNECTION: tokio::sync::Mutex<Option<Arc<Neo4jConnection>>> =
        tokio::sync::Mutex::new(None);
    static ref INIT: Once = Once::new();
}

const DATA_BANK: &str = "Data_Bank";

pub struct Neo4jConnectionManager;

impl Neo4jConnectionManager {
    pub async fn initialize(uri: &str, username: &str, password: &str) -> Result<()> {
        let mut conn_guard = CONNECTION.lock().await;
        if conn_guard.is_some() {
            return Ok(());
        }

        info!("Connecting to Neo4j at {}", uri);
        let config = ConfigBuilder::new()
            .uri(uri)
            .user(username)
            .password(password)
            .build()?;

        match Neo4jConnection::connect(config).await {
            Ok(connection) => {
                info!("Successfully connected to Neo4j");
                *conn_guard = Some(Arc::new(connection));
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to connect to Neo4j: {}", e)),
        }
    }

    pub async fn get_connection() -> Option<Arc<Neo4jConnection>> {
        CONNECTION.lock().await.clone()
    }

    pub async fn clear_connection() {
        let mut conn = CONNECTION.lock().await;
        *conn = None;
    }
    pub async fn initialize_from_env() -> Result<()> {
        let uri =
            std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let username = std::env::var("NEO4J_USERNAME").unwrap_or_else(|_| "neo4j".to_string());
        let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "testtest".to_string());

        Self::initialize(&uri, &username, &password).await
    }
}

pub struct QueryBuilder {
    query: String,
    params: HashMap<String, String>,
}

impl QueryBuilder {
    pub fn new(query_string: &str) -> Self {
        Self {
            query: query_string.to_string(),
            params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params.extend(params);
        self
    }

    pub fn build(&self) -> (String, HashMap<String, String>) {
        (self.query.clone(), self.params.clone())
    }

    pub fn to_neo4j_query(&self) -> neo4rs::Query {
        let mut query_obj = query(&self.query);

        for (key, value) in &self.params {
            query_obj = query_obj.param(key, value.as_str());
        }

        query_obj
    }
}

pub struct NodeQueryBuilder {
    node_type: NodeType,
    node_data: NodeData,
}

impl NodeQueryBuilder {
    pub fn new(node_type: &NodeType, node_data: &NodeData) -> Self {
        Self {
            node_type: node_type.clone(),
            node_data: node_data.clone(),
        }
    }

    pub fn build_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), self.node_data.name.clone());
        params.insert("file".to_string(), self.node_data.file.clone());
        params.insert("start".to_string(), self.node_data.start.to_string());
        params.insert("end".to_string(), self.node_data.end.to_string());
        params.insert("body".to_string(), self.node_data.body.clone());

        let ref_id = if std::env::var("TEST_REF_ID").is_ok() {
            "test_ref_id".to_string()
        } else {
            Uuid::new_v4().to_string()
        };

        params.insert("ref_id".to_string(), ref_id);

        if let Some(data_type) = &self.node_data.data_type {
            params.insert("data_type".to_string(), data_type.clone());
        }
        if let Some(docs) = &self.node_data.docs {
            params.insert("docs".to_string(), docs.clone());
        }
        if let Some(hash) = &self.node_data.hash {
            params.insert("hash".to_string(), hash.clone());
        }
        for (key, value) in &self.node_data.meta {
            match key.as_str() {
                "handler" => {
                    params.insert("handler".to_string(), value.clone());
                }
                "verb" => {
                    params.insert("verb".to_string(), value.clone());
                }
                _ => {
                    params.insert(key.clone(), value.clone());
                }
            }
        }

        params
    }

    pub fn build(&self) -> (String, HashMap<String, String>) {
        let mut params = self.build_params();

        let node_key = create_node_key(&Node::new(self.node_type.clone(), self.node_data.clone()));
        params.insert("node_key".to_string(), node_key);

        let property_list = params
            .keys()
            .filter(|k| k != &"node_key")
            .map(|k| format!("n.{} = ${}", k, k))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "MERGE (n:{}:{} {{node_key: $node_key}})
            ON CREATE SET {}
            ON MATCH SET {}",
            self.node_type.to_string(),
            DATA_BANK,
            property_list,
            property_list
        );

        (query, params)
    }
}
pub struct EdgeQueryBuilder {
    edge: Edge,
}

impl EdgeQueryBuilder {
    pub fn new(edge: &Edge) -> Self {
        Self { edge: edge.clone() }
    }

    pub fn build_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        params.insert(
            "source_name".to_string(),
            self.edge.source.node_data.name.clone(),
        );
        params.insert(
            "source_file".to_string(),
            self.edge.source.node_data.file.clone(),
        );
        params.insert(
            "source_start".to_string(),
            self.edge.source.node_data.start.to_string(),
        );

        if let Some(verb) = &self.edge.source.node_data.verb {
            params.insert("source_verb".to_string(), verb.clone());
        }

        params.insert(
            "target_name".to_string(),
            self.edge.target.node_data.name.clone(),
        );
        params.insert(
            "target_file".to_string(),
            self.edge.target.node_data.file.clone(),
        );
        params.insert(
            "target_start".to_string(),
            self.edge.target.node_data.start.to_string(),
        );

        if let Some(verb) = &self.edge.target.node_data.verb {
            params.insert("target_verb".to_string(), verb.clone());
        }

        params
    }

    pub fn build(&self) -> (String, HashMap<String, String>) {
        let params = self.build_params();

        let rel_type = self.edge.edge.to_string();
        let source_type = self.edge.source.node_type.to_string();
        let target_type = self.edge.target.node_type.to_string();

        let query = format!(
            "MATCH (source:{} {{name: $source_name, file: $source_file}}), \
                       (target:{} {{name: $target_name, file: $target_file}}) \
                 MERGE (source)-[r:{}]->(target)",
            source_type, target_type, rel_type
        );
        (query, params)
    }
}
pub async fn execute_batch(
    conn: &Neo4jConnection,
    queries: Vec<(String, HashMap<String, String>)>,
) -> Result<()> {
    let mut txn = conn.start_txn().await?;

    for (i, (query_str, params)) in queries.iter().enumerate() {
        let mut query_obj = query(&query_str);
        for (k, v) in params {
            query_obj = query_obj.param(&k, v.as_str());
        }

        if let Err(e) = txn.run(query_obj).await {
            println!("Neo4j query #{} {} failed: {}", i, query_str, e);
            txn.rollback().await?;
            return Err(anyhow::anyhow!("Neo4j batch query error: {}", e));
        }
    }

    txn.commit().await?;
    Ok(())
}
pub struct TransactionManager<'a> {
    conn: &'a Neo4jConnection,
    queries: Vec<(String, HashMap<String, String>)>,
}

impl<'a> TransactionManager<'a> {
    pub fn new(conn: &'a Neo4jConnection) -> Self {
        Self {
            conn,
            queries: Vec::new(),
        }
    }

    pub fn add_query(&mut self, query: (String, HashMap<String, String>)) -> &mut Self {
        self.queries.push(query);
        self
    }

    pub fn add_queries(
        &mut self,
        mut queries: Vec<(String, HashMap<String, String>)>,
    ) -> &mut Self {
        self.queries.append(&mut queries);
        self
    }

    pub fn add_node(&mut self, node_type: &NodeType, node_data: &NodeData) -> &mut Self {
        self.queries.push(add_node_query(node_type, node_data));
        self
    }

    pub fn add_edge(&mut self, edge: &Edge) -> &mut Self {
        self.queries.push(add_edge_query(edge));
        self
    }

    pub async fn execute(self) -> Result<()> {
        execute_batch(self.conn, self.queries).await?;
        Ok(())
    }
}

pub fn add_node_query(
    node_type: &NodeType,
    node_data: &NodeData,
) -> (String, HashMap<String, String>) {
    NodeQueryBuilder::new(node_type, node_data).build()
}

pub fn add_edge_query(edge: &Edge) -> (String, HashMap<String, String>) {
    EdgeQueryBuilder::new(edge).build()
}

pub async fn execute_node_query(
    conn: &Neo4jConnection,
    query_str: String,
    params: HashMap<String, String>,
) -> Result<Vec<NodeData>> {
    let mut query_obj = query(&query_str);

    for (key, value) in params {
        query_obj = query_obj.param(&key, value);
    }

    match conn.execute(query_obj).await {
        Ok(mut result) => {
            let mut nodes = Vec::new();

            while let Some(row) = result.next().await? {
                if let Ok(node) = row.get::<neo4rs::Node>("n") {
                    let name = node.get::<String>("name").unwrap_or_default();
                    let file = node.get::<String>("file").unwrap_or_default();
                    let start = node.get::<i32>("start").unwrap_or_default();
                    let end = node.get::<i32>("end").unwrap_or_default();
                    let body = node.get::<String>("body").unwrap_or_default();
                    let data_type = node.get::<String>("data_type").unwrap_or_default();
                    let docs = node.get::<String>("docs").unwrap_or_default();
                    let hash = node.get::<String>("hash").unwrap_or_default();
                    let meta_json = node.get::<String>("meta").unwrap_or_default();
                    let meta = serde_json::from_str::<BTreeMap<String, String>>(&meta_json)
                        .unwrap_or_default();
                    let node_data = NodeData {
                        name,
                        file,
                        start: start as usize,
                        end: end as usize,
                        body,
                        data_type: Some(data_type),
                        docs: Some(docs),
                        hash: Some(hash),
                        meta: meta,
                    };
                    nodes.push(node_data);
                }
            }
            Ok(nodes)
        }
        Err(e) => {
            debug!("Error executing query: {}", e);
            Ok(vec![])
        }
    }
}

pub fn count_nodes_edges_query() -> String {
    "MATCH (n) 
     WITH COUNT(n) as nodes
     MATCH ()-[r]->() 
     RETURN nodes, COUNT(r) as edges"
        .to_string()
}
pub fn graph_node_analysis_query() -> String {
    "MATCH (n) 
     RETURN labels(n)[0] as node_type, n.name as name, n.file as file, n.start as start, 
            n.end as end, n.body as body, n.data_type as data_type, n.docs as docs, 
            n.hash as hash, n.meta as meta
     ORDER BY node_type, name"
        .to_string()
}
pub fn graph_edges_analysis_query() -> String {
    "MATCH (source)-[r]->(target) 
     RETURN labels(source)[0] as source_type, source.name as source_name, source.file as source_file, source.start as source_start,
            type(r) as edge_type, labels(target)[0] as target_type, 
            target.name as target_name, target.file as target_file, target.start as target_start
     ORDER BY source_type, source_name, edge_type, target_type, target_name"
        .to_string()
}
pub fn count_edges_by_type_query(edge_type: &EdgeType) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("edge_type".to_string(), edge_type.to_string());

    let query = "MATCH ()-[r]->() 
                WHERE type(r) = $edge_type 
                RETURN COUNT(r) as count";

    (query.to_string(), params)
}

pub fn find_nodes_by_type_query(node_type: &NodeType) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("node_type".to_string(), node_type.to_string());

    let query = format!(
        "MATCH (n:{}) 
         RETURN n",
        node_type.to_string()
    );

    (query, params)
}
pub fn find_nodes_by_name_query(
    node_type: &NodeType,
    name: &str,
) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("name".to_string(), name.to_string());

    let query = format!(
        "MATCH (n:{}) 
                       WHERE n.name = $name 
                       RETURN n",
        node_type.to_string()
    );

    (query, params)
}

pub fn find_node_by_name_file_query(
    node_type: &NodeType,
    name: &str,
    file: &str,
) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("name".to_string(), name.to_string());
    params.insert("file".to_string(), file.to_string());

    let query = format!(
        "MATCH (n:{}) 
                       WHERE n.name = $name AND n.file = $file 
                       RETURN n",
        node_type.to_string()
    );

    (query, params)
}

pub fn find_nodes_by_file_pattern_query(
    node_type: &NodeType,
    file_pattern: &str,
) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("file_pattern".to_string(), file_pattern.to_string());
    let query = format!(
        "MATCH (n:{}) 
                       WHERE n.file CONTAINS $file_pattern 
                       RETURN n",
        node_type.to_string()
    );

    (query, params)
}

pub fn find_nodes_by_name_contains_query(
    node_type: &NodeType,
    name_part: &str,
) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("name_part".to_string(), name_part.to_string());

    let query = format!(
        "MATCH (n:{}) 
         WHERE n.name CONTAINS $name_part 
         RETURN n",
        node_type.to_string()
    );

    (query, params)
}

pub fn find_nodes_with_edge_type_query(
    source_type: &NodeType,
    target_type: &NodeType,
    edge_type: &EdgeType,
) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("edge_type".to_string(), edge_type.to_string());

    let query = format!(
        "MATCH (source:{})-[r:{}]->(target:{})
         RETURN source.name as source_name, source.file as source_file, source.start as source_start, \
                target.name as target_name, target.file as target_file, target.start as target_start",
        source_type.to_string(),
        edge_type.to_string(),
        target_type.to_string()
    );

    (query, params)
}

pub fn prefix_paths_query(root: &str) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    let root = if root.ends_with('/') {
        root.to_string()
    } else {
        format!("{}/", root)
    };
    params.insert("root".to_string(), root.clone());
    params.insert("root".to_string(), root.to_string());

    let query = "MATCH (n)
                WHERE n.file IS NOT NULL AND NOT n.file STARTS WITH $root
                SET n.file = $root + n.file";

    (query.to_string(), params)
}

pub fn extract_node_data_from_neo4j_node(node: &neo4rs::Node) -> NodeData {
    let name = node.get::<String>("name").unwrap_or_default();
    let file = node.get::<String>("file").unwrap_or_default();
    let start_str = node.get::<String>("start").unwrap_or_default();
    let start = start_str.parse::<usize>().unwrap_or_default();
    let end_str = node.get::<String>("end").unwrap_or_default();
    let end = end_str.parse::<usize>().unwrap_or_default();
    let body = node.get::<String>("body").unwrap_or_default();
    let data_type = node.get::<String>("data_type").ok();
    let docs = node.get::<String>("docs").ok();
    let hash = node.get::<String>("hash").ok();
    let meta_json = node.get::<String>("meta").unwrap_or_default();
    let meta: BTreeMap<String, String> = serde_json::from_str(&meta_json).unwrap_or_default();

    NodeData {
        name,
        file,
        start,
        end,
        body,
        data_type,
        docs,
        hash,
        meta,
    }
}

pub fn get_repository_hash_query(repo_url: &str) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();

    let repo_name = if repo_url.contains('/') {
        let parts: Vec<&str> = repo_url.split('/').collect();
        let name = parts.last().unwrap_or(&repo_url);
        name.trim_end_matches(".git")
    } else {
        repo_url
    };

    params.insert("repo_name".to_string(), repo_name.to_string());

    let query = "MATCH (r:Repository) 
                 WHERE r.name CONTAINS $repo_name 
                 RETURN r.hash as hash";

    (query.to_string(), params)
}

pub fn remove_nodes_by_file_query(file_path: &str) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    let file_name = file_path.split('/').last().unwrap_or(file_path);
    params.insert("file_name".to_string(), file_name.to_string());

    let query = "
        MATCH (n)
        WHERE n.file = $file_name OR n.file ENDS WITH $file_name
        WITH DISTINCT n
        OPTIONAL MATCH (n)-[r]-() 
        DELETE r
        WITH n
        DETACH DELETE n
        RETURN count(n) as deleted
    ";

    (query.to_string(), params)
}

pub fn update_repository_hash_query(
    repo_name: &str,
    new_hash: &str,
) -> (String, HashMap<String, String>) {
    let mut params = HashMap::new();
    params.insert("repo_name".to_string(), repo_name.to_string());
    params.insert("new_hash".to_string(), new_hash.to_string());

    let query = "MATCH (r:Repository) 
                 WHERE r.name CONTAINS $repo_name 
                 SET r.hash = $new_hash";

    (query.to_string(), params)
}

pub fn data_bank_bodies_query_no_token_count() -> String {
    "MATCH (n:Data_Bank) 
     WHERE n.token_count IS NULL AND n.body IS NOT NULL
     RETURN n.node_key as node_key, n.body as body"
        .to_string()
}

pub fn update_token_count_query() -> String {
    "MATCH (n {node_key: $node_key})
     SET n.token_count = $token_count"
        .to_string()
}
