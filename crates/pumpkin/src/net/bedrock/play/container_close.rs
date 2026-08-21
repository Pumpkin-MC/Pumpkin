#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_container_close(&self, player: &Arc<Player>, packet: SContainerClose) {
        if packet.container_id == 0 || packet.container_id == 0xff {
            self.inventory_opened.store(false, Ordering::Relaxed);
        }
        player.on_handled_screen_closed().await;

        self.enqueue_client_packet(&SContainerClose {
            container_id: packet.container_id,
            container_type: packet.container_type,
            server_initiated: false,
        })
        .await;

        // Sync the cursor (make it empty) to Bedrock client
        self.enqueue_client_packet(&CInventoryContent {
            container_id: VarUInt(59), // Cursor container ID
            slots: vec![NetworkItemStackDescriptor::default()],
            full_container_name: FullContainerName {
                container_name: ContainerName::Cursor,
                dynamic_id: None,
            },
            storage_item: NetworkItemStackDescriptor::default(),
        })
        .await;

        // Sync the inventory content to Bedrock client
        self.enqueue_client_packet(&CInventoryContent {
            container_id: VarUInt(0), // player inventory
            slots: player
                .inventory()
                .main_inventory
                .read()
                .await
                .iter()
                .map(NetworkItemStackDescriptor::from)
                .collect(),
            full_container_name: FullContainerName {
                container_name: ContainerName::Inventory,
                dynamic_id: None,
            },
            storage_item: NetworkItemStackDescriptor::default(),
        })
        .await;
    }
}
