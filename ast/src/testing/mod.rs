use crate::lang::{ArrayGraph, Lang};
use crate::utils::get_use_lsp;
use graphs::GraphTestExpectations;
use lsp::Language;
use std::env;
use std::str::FromStr;
use tracing_test::traced_test;

pub mod angular;
pub mod go;
pub mod graphs;
pub mod kotlin;
pub mod python;
pub mod react;
pub mod ruby;
pub mod svelte;
pub mod swift;
pub mod test_backend;
pub mod test_frontend;
pub mod utils;

static TEST_EXPECTATIONS: &[GraphTestExpectations] = &[
    GraphTestExpectations {
        lang_id: "go",
        repo_path: "src/testing/go",
        expected_nodes: 30,
        expected_edges: 48,
        expected_nodes_lsp: Some(64),
        expected_edges_lsp: Some(92),
        critical_edges: Vec::new(),
        critical_edges_lsp: None,
    },
    GraphTestExpectations {
        lang_id: "angular",
        repo_path: "src/testing/angular",
        expected_nodes: 77,
        expected_edges: 78,
        expected_nodes_lsp: None,
        expected_edges_lsp: None,
        critical_edges: Vec::new(),
        critical_edges_lsp: None,
    },
    GraphTestExpectations {
        lang_id: "kotlin",
        repo_path: "src/testing/kotlin",
        expected_nodes: 145,
        expected_edges: 76,
        expected_nodes_lsp: None,
        expected_edges_lsp: None,
        critical_edges: Vec::new(),
        critical_edges_lsp: None,
    },
    GraphTestExpectations {
        lang_id: "swift",
        repo_path: "src/testing/swift",
        expected_nodes: 55,
        expected_edges: 73,
        expected_nodes_lsp: None,
        expected_edges_lsp: None,
        critical_edges: Vec::new(),
        critical_edges_lsp: None,
    },
    GraphTestExpectations {
        lang_id: "ruby",
        repo_path: "src/testing/ruby",
        expected_nodes: 55,
        expected_edges: 79,
        expected_nodes_lsp: None,
        expected_edges_lsp: None,
        critical_edges: Vec::new(),
        critical_edges_lsp: None,
    },
    GraphTestExpectations {
        lang_id: "python",
        repo_path: "src/testing/python",
        expected_nodes: 59,
        expected_edges: 78,
        expected_nodes_lsp: None,
        expected_edges_lsp: None,
        critical_edges: Vec::new(),
        critical_edges_lsp: None,
    },
    GraphTestExpectations {
        lang_id: "svelte",
        repo_path: "src/testing/svelte",
        expected_nodes: 44,
        expected_edges: 42,
        expected_nodes_lsp: None,
        expected_edges_lsp: None,
        critical_edges: Vec::new(),
        critical_edges_lsp: None,
    },
];
#[cfg(test)]
fn pre_test() {
    env::set_var("LSP_SKIP_POST_CLONE", "true");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[traced_test]
async fn run_server_tests() {
    pre_test();
    let implemented_servers = ["go", "python", "ruby", "rust", "typescript", "java"];
    for server in implemented_servers.iter() {
        tracing::info!("Running server tests for {}", server);
        let repo = Some(server.to_string());
        let language = Lang::from_language(Language::from_str(server).unwrap());

        let tester = test_backend::BackendTester::<ArrayGraph>::from_repo(language, repo)
            .await
            .unwrap();
        tester.test_backend().unwrap();
    }
}

// #[test(tokio::test)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_client_tests() {
    pre_test();
    let implemented_clients = ["react", "kotlin", "swift"];

    for server in implemented_clients.iter() {
        let repo = Some(server.to_string());
        let language = Lang::from_language(Language::from_str(server).unwrap());

        let tester = test_frontend::FrontendTester::<ArrayGraph>::from_repo(language, repo)
            .await
            .unwrap();
        tester.test_frontend().unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_graphs_similarity() {
    pre_test();

    let use_lsp = get_use_lsp();

    for expectation in TEST_EXPECTATIONS {
        graphs::run_graph_similarity_test(expectation, use_lsp)
            .await
            .unwrap();
    }
}
