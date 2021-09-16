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

use crate::digraph::PyDiGraph;

use hashbrown::HashMap;

use pyo3::class::iter::{IterNextOutput, PyIterProtocol};
use pyo3::prelude::*;
use pyo3::Python;

use petgraph::prelude::*;
use petgraph::stable_graph::NodeIndex;

fn find_next_layer(
    py: Python,
    dag: &PyDiGraph,
    cur_layer: &[NodeIndex],
    next_layer: &mut Vec<NodeIndex>,
    predecessor_count: &mut HashMap<NodeIndex, usize>,
) -> Vec<PyObject> {
    for node in cur_layer {
        for raw_edge in dag
            .graph
            .edges_directed(*node, petgraph::Direction::Outgoing)
        {
            let succ = raw_edge.target();

            predecessor_count
                .entry(succ)
                .and_modify(|e| *e -= 1)
                .or_insert(
                    dag.graph
                        .edges_directed(succ, petgraph::Direction::Incoming)
                        .count()
                        - 1,
                );
            if *predecessor_count.get(&succ).unwrap() == 0 {
                next_layer.push(succ);
                predecessor_count.remove(&succ);
            }
        }
    }

    next_layer
        .iter()
        .map(|layer_node| dag.graph[*layer_node].clone_ref(py))
        .collect()
}

#[pyclass]
pub struct LayerIter {
    dag: Py<PyDiGraph>,
    cur_layer: Vec<NodeIndex>,
    predecessor_count: HashMap<NodeIndex, usize>,
}

#[pyproto]
impl PyIterProtocol for LayerIter {
    fn __iter__(slf: PyRef<Self>) -> Py<LayerIter> {
        slf.into()
    }

    fn __next__(
        mut slf: PyRefMut<Self>,
    ) -> IterNextOutput<Vec<PyObject>, &'static str> {
        let mut next_layer: Vec<NodeIndex> = Vec::new();
        let mut tmp_predecessor_count = slf.predecessor_count.clone();

        let py = slf.py();
        let layer_node_data: Vec<PyObject> = find_next_layer(
            py,
            &slf.dag.borrow(py),
            &slf.cur_layer,
            &mut next_layer,
            &mut tmp_predecessor_count,
        );

        slf.cur_layer = next_layer;
        slf.predecessor_count = tmp_predecessor_count;

        if layer_node_data.is_empty() {
            IterNextOutput::Return("All Layers Returned")
        } else {
            IterNextOutput::Yield(layer_node_data)
        }
    }
}

/// Note that the output differs from `retworkx.layers` as it
/// does *not* yield the given starting layer.
#[pyfunction]
#[pyo3(text_signature = "(dag, first_layer, /)")]
pub fn lazy_layers(dag: Py<PyDiGraph>, first_layer: Vec<usize>) -> LayerIter {
    LayerIter {
        dag,
        cur_layer: first_layer.into_iter().map(NodeIndex::new).collect(),
        predecessor_count: HashMap::new(),
    }
}
