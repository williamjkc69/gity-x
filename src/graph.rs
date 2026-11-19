use crate::git::Commit;

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from_lane: usize,
    pub to_lane: usize,
    pub color_idx: usize,
}

#[derive(Debug, Clone)]
pub struct GraphRow {
    pub lane: usize,
    pub color_idx: usize,
    pub is_merge: bool,
    pub top_edges: Vec<GraphEdge>,
    pub bottom_edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct CommitGraph {
    pub rows: Vec<GraphRow>,
    pub max_lanes: usize,
}

#[derive(Debug, Clone)]
struct ActiveLane {
    expected_hash: String,
    color_idx: usize,
}

impl CommitGraph {
    pub fn build(commits: &[Commit]) -> Self {
        let mut lanes: Vec<Option<ActiveLane>> = Vec::new();
        let mut next_color: usize = 0;
        let mut rows = Vec::with_capacity(commits.len());
        let mut max_lanes: usize = 1;

        for commit in commits {
            let mut top_edges = Vec::new();
            let mut bottom_edges = Vec::new();

            // 1. Find all active lanes expecting this commit.
            let matching_lanes: Vec<usize> = lanes
                .iter()
                .enumerate()
                .filter_map(|(idx, slot)| {
                    slot.as_ref()
                        .filter(|al| al.expected_hash == commit.hash)
                        .map(|_| idx)
                })
                .collect();

            // Determine the lane & color for this commit node.
            let (node_lane, node_color) = if let Some(&first_match) = matching_lanes.first() {
                let color = lanes[first_match].as_ref().unwrap().color_idx;
                (first_match, color)
            } else {
                // New tip (branch head / tag) -> allocate free lane or append.
                let color = next_color;
                next_color += 1;
                let lane_idx = allocate_lane(&mut lanes);
                lanes[lane_idx] = Some(ActiveLane {
                    expected_hash: commit.hash.clone(),
                    color_idx: color,
                });
                (lane_idx, color)
            };

            // 2. Build top_edges (top of row -> center of row).
            //    Also remember which lanes are pure pass-throughs (not matching this commit).
            let mut passthrough_lanes: Vec<(usize, usize)> = Vec::new();
            for (idx, slot) in lanes.iter().enumerate() {
                if let Some(active) = slot {
                    if matching_lanes.contains(&idx) {
                        // Incoming edge into this commit node (straight if idx == node_lane, curved otherwise).
                        top_edges.push(GraphEdge {
                            from_lane: idx,
                            to_lane: node_lane,
                            color_idx: active.color_idx,
                        });
                    } else if idx != node_lane {
                        // Pass-through lane from above to center.
                        top_edges.push(GraphEdge {
                            from_lane: idx,
                            to_lane: idx,
                            color_idx: active.color_idx,
                        });
                        passthrough_lanes.push((idx, active.color_idx));
                    }
                }
            }

            // Free any secondary lanes that converged into this commit.
            for &m_idx in matching_lanes.iter().skip(1) {
                lanes[m_idx] = None;
            }

            // 3. Pass-through lanes continue straight from center to bottom.
            for &(idx, color) in &passthrough_lanes {
                bottom_edges.push(GraphEdge {
                    from_lane: idx,
                    to_lane: idx,
                    color_idx: color,
                });
            }

            // 4. Route outgoing edges to parents (center of row -> bottom of row).
            if commit.parents.is_empty() {
                // Root commit: lane terminates here.
                lanes[node_lane] = None;
            } else {
                // First parent
                let first_parent = &commit.parents[0];
                if let Some(existing_lane) = find_lane_for_hash(&lanes, first_parent, Some(node_lane)) {
                    // Merge into an already-active lane expecting first_parent.
                    bottom_edges.push(GraphEdge {
                        from_lane: node_lane,
                        to_lane: existing_lane,
                        color_idx: node_color,
                    });
                    lanes[node_lane] = None;
                } else {
                    // Continue straight down in node_lane.
                    lanes[node_lane] = Some(ActiveLane {
                        expected_hash: first_parent.clone(),
                        color_idx: node_color,
                    });
                    bottom_edges.push(GraphEdge {
                        from_lane: node_lane,
                        to_lane: node_lane,
                        color_idx: node_color,
                    });
                }

                // Additional parents (merge commits)
                for parent_hash in commit.parents.iter().skip(1) {
                    if let Some(existing_lane) = find_lane_for_hash(&lanes, parent_hash, None) {
                        let target_color = lanes[existing_lane].as_ref().unwrap().color_idx;
                        bottom_edges.push(GraphEdge {
                            from_lane: node_lane,
                            to_lane: existing_lane,
                            color_idx: target_color,
                        });
                    } else {
                        let new_color = next_color;
                        next_color += 1;
                        let new_lane = allocate_lane(&mut lanes);
                        lanes[new_lane] = Some(ActiveLane {
                            expected_hash: parent_hash.clone(),
                            color_idx: new_color,
                        });
                        bottom_edges.push(GraphEdge {
                            from_lane: node_lane,
                            to_lane: new_lane,
                            color_idx: new_color,
                        });
                    }
                }
            }

            // Trim trailing empty slots to keep width compact.
            while matches!(lanes.last(), Some(None)) {
                lanes.pop();
            }

            let row_width = lanes.len().max(node_lane + 1);
            if row_width > max_lanes {
                max_lanes = row_width;
            }

            rows.push(GraphRow {
                lane: node_lane,
                color_idx: node_color,
                is_merge: commit.parents.len() > 1,
                top_edges,
                bottom_edges,
            });
        }

        Self { rows, max_lanes }
    }
}

fn allocate_lane(lanes: &mut Vec<Option<ActiveLane>>) -> usize {
    if let Some(pos) = lanes.iter().position(|slot| slot.is_none()) {
        pos
    } else {
        lanes.push(None);
        lanes.len() - 1
    }
}

fn find_lane_for_hash(
    lanes: &[Option<ActiveLane>],
    hash: &str,
    exclude_lane: Option<usize>,
) -> Option<usize> {
    lanes.iter().enumerate().find_map(|(idx, slot)| {
        if exclude_lane == Some(idx) {
            return None;
        }
        slot.as_ref()
            .filter(|al| al.expected_hash == hash)
            .map(|_| idx)
    })
}
