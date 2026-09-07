#[allow(clippy::wildcard_imports)]
use super::*;
use std::sync::atomic::Ordering;

impl PendingConnection {
    pub(in crate::net::java) async fn send_bedrock_skin_pack(&mut self, server: &Server) {
        let skin_config = &server.advanced_config.networking.bedrock.skins;
        if !skin_config.java_resource_pack
            || !server.bedrock_skin_pack_endpoint.load(Ordering::Acquire)
            || self.version.load() < JavaMinecraftVersion::V_26_1
        {
            return;
        }
        let Some(pack) = server.bedrock_skin_packs.current().await else {
            return;
        };
        if self.pending_bedrock_skin_pack.is_some()
            || self.last_bedrock_skin_pack_offer == Some(pack.id)
            || self
                .bedrock_skin_pack
                .as_ref()
                .is_some_and(|loaded| loaded.id == pack.id)
        {
            return;
        }
        let port = server
            .advanced_config
            .networking
            .bedrock
            .nethernet
            .address
            .port();
        let url = crate::net::bedrock::skin_pack::resource_url(
            &self.server_address,
            port,
            skin_config.resource_pack_url.as_deref(),
            pack.id,
        );
        self.send_packet_now(&CConfigAddResourcePack::new(
            &pack.id, &url, &pack.hash, false, None,
        ))
        .await;
        self.pending_resource_packs.insert(pack.id);
        self.last_bedrock_skin_pack_offer = Some(pack.id);
        self.pending_bedrock_skin_pack = Some(pack);
    }

    pub async fn handle_login_acknowledged(
        &mut self,
        server: &Server,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling login acknowledgement");
        if !self.version.load().supports_configuration_state() {
            self.kick(TextComponent::text(
                "Configuration state not supported for this version",
            ))
            .await;
            return Some(PacketHandlerResult::Stop);
        }
        self.connection_state.store(ConnectionState::Config);
        self.send_packet_now(&server.get_branding()).await;

        if server.advanced_config.server_links.enabled
            && self.version.load() >= JavaMinecraftVersion::V_1_21
        {
            let mut links: Vec<Link> = Vec::new();

            let bug_report = &server.advanced_config.server_links.bug_report;
            if !bug_report.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::BugReport), bug_report));
            }

            let support = &server.advanced_config.server_links.support;
            if !support.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Support), support));
            }

            let status = &server.advanced_config.server_links.status;
            if !status.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Status), status));
            }

            let feedback = &server.advanced_config.server_links.feedback;
            if !feedback.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Feedback), feedback));
            }

            let community = &server.advanced_config.server_links.community;
            if !community.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Community), community));
            }

            let website = &server.advanced_config.server_links.website;
            if !website.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Website), website));
            }

            let forums = &server.advanced_config.server_links.forums;
            if !forums.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::Forums), forums));
            }

            let news = &server.advanced_config.server_links.news;
            if !news.is_empty() {
                links.push(Link::new(Label::BuiltIn(LinkType::News), news));
            }

            let announcements = &server.advanced_config.server_links.announcements;
            if !announcements.is_empty() {
                links.push(Link::new(
                    Label::BuiltIn(LinkType::Announcements),
                    announcements,
                ));
            }

            for (key, value) in &server.advanced_config.server_links.custom {
                links.push(Link::new(
                    Label::TextComponent(TextComponent::text(key.clone()).into()),
                    value,
                ));
            }

            self.send_packet_now(&CConfigServerLinks::new(&links)).await;
        }

        let resource_config = &server.advanced_config.resource_pack.java;
        if resource_config.enabled {
            let uuid = Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, resource_config.url.as_bytes());
            let resource_pack = CConfigAddResourcePack::new(
                &uuid,
                &resource_config.url,
                &resource_config.sha1,
                resource_config.force,
                if resource_config.prompt_message.is_empty() {
                    None
                } else {
                    Some(TextComponent::text(resource_config.prompt_message.clone()))
                },
            );

            self.send_packet_now(&resource_pack).await;
            self.pending_resource_packs.insert(uuid);
        }
        self.send_bedrock_skin_pack(server).await;
        if self.pending_resource_packs.is_empty() {
            self.continue_configuration_after_resource_packs().await;
        }
        debug!("login acknowledged");
        None
    }

    pub(in crate::net::java) async fn continue_configuration_after_resource_packs(&mut self) {
        if self.version.load() >= JavaMinecraftVersion::V_1_20_5 {
            self.send_known_packs().await;
        } else {
            self.handle_known_packs().await;
        }
    }

    pub async fn send_known_packs(&mut self) {
        let version_str = self.version.load().to_string();
        self.send_packet_now(&CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: &version_str,
        }]))
        .await;
    }
}
