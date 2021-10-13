// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License. You may obtain
// a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

use std::cmp::Eq;
use std::hash::Hash;

use hashbrown::HashMap;

use petgraph::visit::{
    depth_first_search, DfsEvent, GraphProp, IntoNeighbors,
    IntoNodeIdentifiers, NodeCount, NodeIndexable, VisitMap, Visitable,
};
use petgraph::Undirected;

fn _build_chain<G, VM: VisitMap<G::NodeId>>(
    graph: G,
    parent: &[usize],
    mut u_id: G::NodeId,
    mut v_id: G::NodeId,
    visited: &mut VM,
) -> Vec<(G::NodeId, G::NodeId)>
where
    G: Visitable + NodeIndexable,
{
    let mut chain = Vec::new();
    while visited.visit(v_id) {
        chain.push((u_id, v_id));
        u_id = v_id;
        let u = graph.to_index(u_id);
        let v = parent[u];
        v_id = graph.from_index(v);
    }
    chain.push((u_id, v_id));

    chain
}

pub fn chain_decomposition<G>(
    graph: G,
    source: Option<G::NodeId>,
) -> Vec<Vec<(G::NodeId, G::NodeId)>>
where
    G: IntoNodeIdentifiers
        + IntoNeighbors
        + Visitable
        + NodeIndexable
        + NodeCount
        + GraphProp<EdgeType = Undirected>,
    G::NodeId: Eq + Hash,
{
    let roots = match source {
        Some(node) => vec![node],
        None => graph.node_identifiers().collect(),
    };

    let mut parent = vec![std::usize::MAX; graph.node_bound()];
    let mut back_edges: HashMap<G::NodeId, Vec<G::NodeId>> = HashMap::new();

    // depth-first-index (DFI) ordered nodes.
    let mut nodes = Vec::with_capacity(graph.node_count());
    depth_first_search(graph, roots, |event| match event {
        DfsEvent::Discover(u, _) => {
            nodes.push(u);
        }
        DfsEvent::TreeEdge(u, v) => {
            let u = graph.to_index(u);
            let v = graph.to_index(v);
            parent[v] = u;
        }
        DfsEvent::BackEdge(u_id, v_id) => {
            let u = graph.to_index(u_id);
            let v = graph.to_index(v_id);

            // do *not* consider ``(u, v)`` as a back edge if ``(v, u)`` is a tree edge.
            if parent[u] != v {
                back_edges
                    .entry(v_id)
                    .and_modify(|v_edges| v_edges.push(u_id))
                    .or_insert(vec![u_id]);
            }
        }
        _ => {}
    });

    let visited = &mut graph.visit_map();
    let mut chains = Vec::new();

    for u in nodes {
        visited.visit(u);
        if let Some(vs) = back_edges.get(&u) {
            for v in vs {
                let chain = _build_chain(graph, &parent, u, *v, visited);
                chains.push(chain);
            }
        }
    }
    chains
}
