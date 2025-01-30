use wg_2024::network::NodeId;

pub struct TestSectionState {
    pub send_default_fragment_node_id: NodeId,
    pub send_default_flood_request_node_id: NodeId,
    pub send_default_ack_node_id: NodeId,
    pub send_default_nack_node_id: NodeId,
    pub routing_path_string: String,
    pub msg_frag_data_string: String,
    pub packet_sender_status_flag: Option<Result<String, String>>,
    pub channel_modifier_status_flag: Option<Result<String, String>>,
}

impl Default for TestSectionState {
    fn default() -> Self {
        Self {
            send_default_fragment_node_id: 1,
            send_default_flood_request_node_id: 1,
            send_default_ack_node_id: 1,
            send_default_nack_node_id: 1,
            routing_path_string: String::new(),
            msg_frag_data_string: String::new(),
            packet_sender_status_flag: None,
            channel_modifier_status_flag: None,
        }
    }
}
