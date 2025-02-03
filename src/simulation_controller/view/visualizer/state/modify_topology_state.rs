use wg_2024::network::NodeId;

struct ModifyTopologyState {
    id: NodeId,
    pdr: f32,
    neighbors: String,
}
