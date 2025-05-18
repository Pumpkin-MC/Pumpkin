/*
#[derive(Debug, Default)]
pub struct DragHandler(RwLock<HashMap<u64, Arc<Mutex<Drag>>>>);

impl DragHandler {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub async fn new_drag(
        &self,
        container_id: u64,
        player: i32,
        drag_type: MouseDragType,
    ) -> Result<(), InventoryError> {
        let drag = Drag {
            player,
            drag_type,
            slots: vec![],
        };
        let mut drags = self.0.write().await;
        drags.insert(container_id, Arc::new(Mutex::new(drag)));
        Ok(())
    }

    pub async fn add_slot(
        &self,
        container_id: u64,
        player: i32,
        slot: usize,
    ) -> Result<(), InventoryError> {
        let drags = self.0.read().await;
        match drags.get(&container_id) {
            Some(drag) => {
                let mut drag = drag.lock().await;
                if drag.player != player {
                    Err(InventoryError::MultiplePlayersDragging)?
                }
                if !drag.slots.contains(&slot) {
                    drag.slots.push(slot);
                }
            }
            None => Err(InventoryError::OutOfOrderDragging)?,
        }
        Ok(())
    }

    pub async fn apply_drag<T: Container>(
        &self,
        maybe_carried_item: &mut Option<ItemStack>,
        container: &mut T,
        container_id: &u64,
        player: i32,
    ) -> Result<(), InventoryError> {
        // The Minecraft client does still send dragging packets when not carrying an item!
        if maybe_carried_item.is_none() {
            return Ok(());
        }

        let mut drags = self.0.write().await;
        let Some((_, drag)) = drags.remove_entry(container_id) else {
            Err(InventoryError::OutOfOrderDragging)?
        };
        let drag = drag.lock().await;

        if player != drag.player {
            Err(InventoryError::MultiplePlayersDragging)?
        }
        let mut slots = container.all_slots();
        let Some(carried_item) = maybe_carried_item else {
            return Ok(());
        };
        match drag.drag_type {
            // This is only valid in the Creative gamemode.
            // Checked in any function that uses this function.
            MouseDragType::Middle => {
                for slot in &drag.slots {
                    *slots[*slot] = maybe_carried_item.clone();
                }
            }
            MouseDragType::Right => {
                let changing_slots =
                    drag.possibly_changing_slots(slots.as_ref(), carried_item.item.id);
                changing_slots.into_iter().for_each(|slot| {
                    if carried_item.item_count != 0 {
                        carried_item.item_count -= 1;
                        if let Some(stack) = &mut slots[slot] {
                            // TODO: Check for stack max here
                            if stack.item_count + 1 < stack.item.components.max_stack_size {
                                stack.item_count += 1;
                            } else {
                                carried_item.item_count += 1;
                            }
                        } else {
                            *slots[slot] = Some(ItemStack {
                                item: carried_item.item.clone(),
                                item_count: 1,
                            })
                        }
                    }
                });

                if carried_item.item_count == 0 {
                    *maybe_carried_item = None
                }
            }
            MouseDragType::Left => {
                // TODO: Handle dragging a stack with a greater amount than the item allows as max unstackable.
                // In that specific case, follow `MouseDragType::Right` behaviours instead!

                let changing_slots = drag.possibly_changing_slots(&slots, carried_item.item.id);
                let amount_of_slots = changing_slots.len();
                let (amount_per_slot, remainder) = if amount_of_slots == 0 {
                    // TODO: please work lol
                    (1, 0)
                } else {
                    (
                        carried_item.item_count.div_euclid(amount_of_slots as u8),
                        carried_item.item_count.rem_euclid(amount_of_slots as u8),
                    )
                };
                changing_slots.into_iter().for_each(|slot| {
                    if let Some(stack) = slots[slot].as_mut() {
                        debug_assert!(stack.item.id == carried_item.item.id);
                        // TODO: Handle max stack size
                        stack.item_count += amount_per_slot;
                    }
                });

                if remainder > 0 {
                    carried_item.item_count = remainder;
                } else {
                    *maybe_carried_item = None
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug)]
struct Drag {
    player: i32,
    drag_type: MouseDragType,
    slots: Vec<usize>,
}

impl Drag {
    fn possibly_changing_slots(
        &self,
        slots: &[&mut Option<ItemStack>],
        carried_item_id: u16,
    ) -> Box<[usize]> {
        self.slots
            .iter()
            .filter_map(move |slot_index| {
                let slot = &slots[*slot_index];

                match slot {
                    Some(item_slot) => {
                        if item_slot.item.id == carried_item_id {
                            Some(*slot_index)
                        } else {
                            None
                        }
                    }
                    None => Some(*slot_index),
                }
            })
            .collect()
    }
}
 */
