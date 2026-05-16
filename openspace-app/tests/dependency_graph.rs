use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

const FEATURE_CRATES: &[&str] = &[
    "openspace-terminal",
    "openspace-chat",
    "openspace-ai",
    "openspace-editor",
    "openspace-lsp",
    "openspace-fs",
    "openspace-git",
];

fn parse_workspace_members() -> Vec<String> {
    let root_cargo = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml"),
    )
    .expect("root Cargo.toml exists");

    let doc = root_cargo.parse::<toml::Table>().expect("valid TOML");
    let members = doc["workspace"]["members"]
        .as_array()
        .expect("workspace.members is array");

    members
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn parse_crate_deps(crate_name: &str) -> Vec<String> {
    let cargo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(crate_name)
        .join("Cargo.toml");
    let cargo = fs::read_to_string(&cargo_path).expect("crate Cargo.toml exists");
    let doc = cargo.parse::<toml::Table>().expect("valid TOML");

    let mut deps = Vec::new();
    if let Some(table) = doc.get("dependencies").and_then(|d| d.as_table()) {
        for key in table.keys() {
            deps.push(key.to_string());
        }
    }
    deps
}

fn has_cycle(graph: &HashMap<String, Vec<String>>) -> Option<Vec<String>> {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();

    fn dfs(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = dfs(neighbor, graph, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    if let Some(pos) = path.iter().position(|p| p == neighbor) {
                        return Some(path[pos..].to_vec());
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }

    for node in graph.keys() {
        if !visited.contains(node) {
            if let Some(cycle) = dfs(node, graph, &mut visited, &mut rec_stack, &mut path) {
                return Some(cycle);
            }
        }
    }
    None
}

#[test]
fn no_cycles_in_dependency_graph() {
    let members = parse_workspace_members();
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for member in &members {
        let deps = parse_crate_deps(member);
        let internal_deps: Vec<String> = deps
            .into_iter()
            .filter(|d| members.contains(d))
            .collect();
        graph.insert(member.clone(), internal_deps);
    }

    if let Some(cycle) = has_cycle(&graph) {
        panic!("Dependency cycle detected: {:?}", cycle);
    }
}

#[test]
fn no_feature_to_feature_lateral_dependencies() {
    let members = parse_workspace_members();
    let feature_set: HashSet<String> = FEATURE_CRATES.iter().map(|s| s.to_string()).collect();

    for feature in FEATURE_CRATES {
        let deps = parse_crate_deps(feature);
        let internal_deps: Vec<String> = deps
            .into_iter()
            .filter(|d| members.contains(d))
            .collect();

        for dep in &internal_deps {
            if dep != "openspace-core" && feature_set.contains(dep) {
                panic!(
                    "Forbidden lateral dependency: {} -> {}. \
                     Feature crates may only depend on openspace-core, not on other feature crates.",
                    feature, dep
                );
            }
        }
    }
}
