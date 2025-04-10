use crate::lang::{Edge, Lang, NodeType};
use crate::lang::{Function, FunctionCall};
use anyhow::Result;
use std::fmt::Debug;

use super::{EdgeType, NodeData, NodeKeys};

pub trait Graph: Default + Debug {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
    fn with_capacity(_nodes: usize, _edges: usize) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
    fn create_filtered_graph(&self, final_filter: &[String]) -> Self
    where
        Self: Sized;

    fn extend_graph(&mut self, other: Self)
    where
        Self: Sized;

    fn get_graph_size(&self) -> (u32, u32);

    //Nodes
    fn find_nodes_by_name(&self, node_type: NodeType, name: &str) -> Vec<NodeData>;
    fn find_nodes_in_range(&self, node_type: NodeType, row: u32, file: &str) -> Option<NodeData>;
    fn find_node_at(&self, node_type: NodeType, file: &str, line: u32) -> Option<NodeData>;
    fn add_node_with_parent(
        &mut self,
        node_type: NodeType,
        node_data: NodeData,
        parent_type: NodeType,
        parent_file: &str,
    );
    fn find_node_by_name_in_file(
        &self,
        node_type: NodeType,
        name: &str,
        file: &str,
    ) -> Option<NodeData>;
    fn find_node_by_name_and_file_end_with(
        &self,
        node_type: NodeType,
        name: &str,
        suffix: &str,
    ) -> Option<NodeData>;
    fn find_nodes_by_file_ends_with(&self, node_type: NodeType, file: &str) -> Vec<NodeData>;
    // this method is used only in ruby (so far)

    fn find_source_edge_by_name_and_file(
        &self,
        edge_type: EdgeType,
        target_name: &str,
        target_file: &str,
    ) -> Option<NodeKeys>;

    //Special cases
    fn process_endpoint_groups(&mut self, eg: Vec<NodeData>, lang: &Lang) -> Result<()>;
    fn class_inherits(&mut self);
    fn class_includes(&mut self);
    fn add_instances(&mut self, nodes: Vec<NodeData>);
    fn add_functions(&mut self, functions: Vec<Function>);
    fn add_page(&mut self, page: (NodeData, Option<Edge>));
    fn add_pages(&mut self, pages: Vec<(NodeData, Vec<Edge>)>);
    fn add_endpoints(&mut self, endpoints: Vec<(NodeData, Option<Edge>)>);
    fn add_test_node(&mut self, test_data: NodeData, test_type: NodeType, test_edge: Option<Edge>);
    fn add_calls(&mut self, calls: (Vec<FunctionCall>, Vec<FunctionCall>, Vec<Edge>));
    fn filter_out_nodes_without_children(
        &mut self,
        parent_type: NodeType,
        child_type: NodeType,
        child_meta_key: &str,
    );
    fn get_data_models_within(&mut self, lang: &Lang);
    fn prefix_paths(&mut self, root: &str);

    //Specific
    fn find_endpoint(&self, name: &str, file: &str, verb: &str) -> Option<NodeData>;

    fn find_data_model_nodes(&self, name: &str) -> Vec<NodeData>;
    fn find_resource_nodes(&self, node_type: NodeType, verb: &str, path: &str) -> Vec<NodeData>;
    fn find_handlers_for_endpoint(&self, endpoint: &NodeData) -> Vec<NodeData>;
    fn check_direct_data_model_usage(&self, function_name: &str, data_model: &str) -> bool;
    fn find_functions_called_by(&self, function: &NodeData) -> Vec<NodeData>;
    fn check_indirect_data_model_usage(
        &self,
        function_name: &str,
        data_model: &str,
        visited: &mut Vec<String>,
    ) -> bool;
    fn find_nodes_by_type(&self, node_type: NodeType) -> Vec<NodeData>;
    fn find_nodes_with_edge_type(
        &self,
        source_type: NodeType,
        target_type: NodeType,
        edge_type: EdgeType,
    ) -> Vec<(NodeData, NodeData)>;
    fn count_edges_of_type(&self, edge_type: EdgeType) -> usize;
}
