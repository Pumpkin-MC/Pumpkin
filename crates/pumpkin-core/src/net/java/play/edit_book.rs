#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_config::BookSizeConfig;

impl JavaClient {
    pub fn handle_edit_book(&self, player: &Player, packet: &SEditBook<'_>) {
        if let Some(server) = player.world().server.upgrade() {
            let config = &server.advanced_config.networking.java.book_size;
            if config.enabled {
                let (size, allowed) = book_size_and_allowance(&packet.pages, config);
                if size > allowed {
                    warn!(
                        "{} tried to send a book too large. Book size: {size} - Allowed: {allowed} - Pages: {}",
                        player.gameprofile.name,
                        packet.pages.len()
                    );
                    self.try_kick(&TextComponent::text("Book too large!"));
                    return;
                }
            }
        }

        let held_stack = player.inventory().held_item();
        if held_stack.item.id != Item::WRITABLE_BOOK.id {
            return;
        }

        let mut pages: Vec<String> = packet.pages.iter().map(|p| (*p).to_string()).collect();
        let mut title = packet.title.map(std::string::ToString::to_string);
        let signing = title.is_some();
        let slot = player.inventory().get_selected_slot() as u32;

        if let Some(player_arc) = player.world().get_player_by_uuid(player.gameprofile.id)
            && let Some(server) = player.world().server.upgrade()
        {
            let mut event =
                crate::plugin::api::events::player::player_edit_book::PlayerEditBookEvent {
                    player: player_arc,
                    slot,
                    pages: pages.clone(),
                    title: title.clone(),
                    signing,
                    cancelled: false,
                };
            event = server.plugin_manager.fire_blocking(&server, event);
            if event.cancelled {
                return;
            }
            pages = event.pages;
            title = event.title;
        }

        if let Some(title) = title {
            let mut written_book = ItemStack::new(1, &Item::WRITTEN_BOOK);
            let content = WrittenBookContentImpl {
                title,
                author: player.gameprofile.name.clone(),
                generation: 0,
                pages: pages.into_iter().map(TextComponent::text).collect(),
                resolved: false,
            };
            written_book
                .patch
                .push((DataComponent::WrittenBookContent, Some(content.to_dyn())));
            player.inventory().set_held_item(written_book);
        } else {
            let mut writable_book = held_stack;
            let content = WritableBookContentImpl { pages };
            writable_book
                .patch
                .retain(|(component, _)| *component != DataComponent::WritableBookContent);
            writable_book
                .patch
                .push((DataComponent::WritableBookContent, Some(content.to_dyn())));
            player.inventory().set_held_item(writable_book);
        }
    }
}

/// Returns the UTF-8 size of `pages` and the byte budget they are allowed, as Paper computes it.
///
/// Each page adds a share of `page_max` proportional to its length (at least a tenth, at
/// most a full page), and pages with several multi-byte characters lose one byte each.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "Page lengths are bounded by the packet decoder, and the f64 to i64 cast mirrors Java's compound assignment"
)]
fn book_size_and_allowance(pages: &[&str], config: &BookSizeConfig) -> (i64, i64) {
    let page_max = f64::from(config.page_max);
    let multiplier = config.total_multiplier.clamp(0.3, 1.0);
    let mut size = 0i64;
    let mut allowed = i64::from(config.page_max);
    for page in pages {
        size += page.len() as i64;
        // Java's `String.length()` counts UTF-16 code units.
        let length = page.encode_utf16().count();
        let multi_byte_characters = page.encode_utf16().filter(|&unit| unit > 127).count();
        allowed = (allowed as f64 + page_max * (length as f64 / 255.0).clamp(0.1, 1.0) * multiplier)
            as i64;
        if multi_byte_characters > 1 {
            allowed -= multi_byte_characters as i64;
        }
    }
    (size, allowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowance(page: &str, count: usize) -> (i64, i64) {
        book_size_and_allowance(&vec![page; count], &BookSizeConfig::default())
    }

    #[test]
    fn rejects_full_pages_of_multi_byte_characters() {
        let page = "\u{d55c}".repeat(1024);
        assert_eq!(allowance(&page, 100), (307_200, 150_960));
    }

    #[test]
    fn accepts_full_pages_of_ascii() {
        let page = "a".repeat(1024);
        assert_eq!(allowance(&page, 100), (102_400, 253_360));
    }

    #[test]
    fn short_pages_get_at_least_a_tenth_of_the_budget() {
        assert_eq!(allowance("", 1), (0, 2_810));
    }
}
