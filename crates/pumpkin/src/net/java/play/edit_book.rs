#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_edit_book(&self, player: &Player, packet: SEditBook<'_>) {
        let held_stack = player.inventory().held_item().await;
        if held_stack.item.id != Item::WRITABLE_BOOK.id {
            return;
        }

        let pages: Vec<String> = packet.pages.iter().map(|p| (*p).to_string()).collect();

        if let Some(title) = packet.title {
            let mut written_book = ItemStack::new(1, &Item::WRITTEN_BOOK);
            let content = WrittenBookContentImpl {
                title: title.to_string(),
                author: player.gameprofile.name.clone(),
                pages,
            };
            written_book
                .patch
                .push((DataComponent::WrittenBookContent, Some(content.to_dyn())));
            player.inventory().set_held_item(written_book).await;
        } else {
            let mut writable_book = held_stack;
            let content = WritableBookContentImpl { pages };
            writable_book
                .patch
                .retain(|(component, _)| *component != DataComponent::WritableBookContent);
            writable_book
                .patch
                .push((DataComponent::WritableBookContent, Some(content.to_dyn())));
            player.inventory().set_held_item(writable_book).await;
        }
    }
}
