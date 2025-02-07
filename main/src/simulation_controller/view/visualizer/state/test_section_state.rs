use wg_2024::network::NodeId;

use crate::simulation_controller::util::StatusFlag;

pub struct TestSectionState {
    pub send_default_fragment_node_id: NodeId,
    pub send_default_flood_request_node_id: NodeId,
    pub send_default_ack_node_id: NodeId,
    pub send_default_nack_node_id: NodeId,

    pub msg_fragment_routing_path_string: String,
    pub msg_frag_data_string: String,
    pub packet_sender_status_flag: StatusFlag,

    pub ack_nack_routing_path_string: String,
    pub ack_nack_sender_status_flag: StatusFlag,

    pub flood_req_initiator_id: NodeId,
    pub flood_req_sender_status_flag: StatusFlag,
    pub flood_latest_used_id: u64,
    pub channel_modifier_status_flag: StatusFlag,
}

impl Default for TestSectionState {
    fn default() -> Self {
        Self {
            send_default_fragment_node_id: 1,
            send_default_flood_request_node_id: 1,
            send_default_ack_node_id: 1,
            send_default_nack_node_id: 1,
            msg_fragment_routing_path_string: String::new(),
            msg_frag_data_string: String::new(),
            packet_sender_status_flag: None,
            channel_modifier_status_flag: None,
            flood_req_sender_status_flag: None,
            ack_nack_routing_path_string: String::new(),
            ack_nack_sender_status_flag: None,
            flood_req_initiator_id: 1,
            flood_latest_used_id: 0,
        }
    }
}
