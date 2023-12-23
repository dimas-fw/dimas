//! Copyright © 2023 Stephan Kunz

use makepad_widgets::file_tree::*;
use makepad_widgets::*;

use crate::{
    network_client::{NetworkClient, NetworkClientAction},
    network_protocol::*,
};

#[derive(Debug, Default)]
pub struct NetworkSystem {
    network_client: NetworkClient,
    net_nodes: LiveIdMap<FileNodeId, NetworkNode>,
}

#[derive(Debug, Default, Clone)]
pub struct NetworkNode {
    pub parent_edge: Option<NetworkEdge>,
    pub name: String,
    pub child_edges: Option<Vec<NetworkEdge>>,
}

#[derive(Debug, Default, Clone)]
pub struct NetworkEdge {
    pub name: String,
    pub network_node_id: FileNodeId,
}

#[derive(Debug, Default, Clone)]
pub enum NetworkTreeAction {
    #[default]
    Nothing,
    RedrawTree,
}

impl NetworkSystem {
    pub fn init(&mut self, cx: &mut Cx) {
        self.network_client.init(cx);
    }

    pub fn handle_event(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        ui: &WidgetRef,
    ) -> Vec<NetworkTreeAction> {
        let mut actions = Vec::new();
        self.handle_event_with(cx, event, ui, &mut |_, action| actions.push(action));
        actions
    }

    fn handle_event_with(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        _ui: &WidgetRef,
        dispatch_action: &mut dyn FnMut(&mut Cx, NetworkTreeAction),
    ) {
        //dbg!("thw {}", event);
        for action in self.network_client.handle_event(cx, event) {
            match action {
                NetworkClientAction::Nothing => {
                    dbg!("NetworkClientAction::Nothing");
                }
                NetworkClientAction::TreeChanged(tree_data) => {
                    //dbg!("NetworkClientAction::TreeChanged(..)");
                    self.load_network_tree(tree_data);
                    dispatch_action(cx, NetworkTreeAction::RedrawTree);
                }
            }
        }
    }

    pub fn draw_node(&self, cx: &mut Cx2d, net_node_id: FileNodeId, network_tree: &mut FileTree) {
        //dbg!("called draw");
        if let Some(net_node) = self.net_nodes.get(&net_node_id) {
            match &net_node.child_edges {
                Some(child_edges) => {
                    if network_tree
                        .begin_folder(cx, net_node_id, &net_node.name)
                        .is_ok()
                    {
                        for child_edge in child_edges {
                            self.draw_node(cx, child_edge.network_node_id, network_tree);
                        }
                        network_tree.end_folder();
                    }
                }
                None => {
                    network_tree.file(cx, net_node_id, &net_node.name);
                }
            }
        }
    }

    pub fn load_network_tree(&mut self, tree_data: NetworkTreeData) {
        fn create_network_node(
            network_node_id: Option<FileNodeId>,
            node_path: String,
            file_nodes: &mut LiveIdMap<FileNodeId, NetworkNode>,
            parent_edge: Option<NetworkEdge>,
            node: NetworkTreeNode,
        ) -> FileNodeId {
            let network_node_id = network_node_id.unwrap_or(LiveId::from_str(&node_path).into());
            let name = parent_edge
                .as_ref()
                .map_or_else(|| String::from("network"), |edge| edge.name.clone());
            let node = NetworkNode {
                parent_edge,
                name,
                child_edges: match node {
                    NetworkTreeNode::Root { entries } => {
                        //dbg!("Root");
                        Some(
                            entries
                                .into_iter()
                                .map(|entry| NetworkEdge {
                                    name: entry.name.clone(),
                                    network_node_id: create_network_node(
                                        None,
                                        if node_path.is_empty() {
                                            format!("{}/{}", node_path, entry.name.clone())
                                        } else {
                                            entry.name.clone()
                                        },
                                        file_nodes,
                                        Some(NetworkEdge {
                                            name: entry.name,
                                            network_node_id,
                                        }),
                                        entry.node,
                                    ),
                                })
                                .collect::<Vec<_>>(),
                        )
                    }
                    NetworkTreeNode::Host { .. } => {
                        //dbg!("Host");
                        None
                    }
                    NetworkTreeNode::Unknown { .. } => {
                        //dbg!("Unknown");
                        None
                    }
                },
            };
            file_nodes.insert(network_node_id, node);
            network_node_id
        }

        //dbg!(&tree_data);
        self.net_nodes.clear();

        create_network_node(
            Some(live_id!(root).into()),
            "".to_string(),
            &mut self.net_nodes,
            None,
            tree_data.root,
        );
    }
}
