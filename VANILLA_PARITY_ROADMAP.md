# KẾ HOẠCH TỔNG THỂ & LỘ TRÌNH ĐẠT 100% VANILLA PARITY
## ĐỐI CHIẾU MÃ NGUỒN PUMPKIN (RUST) VỚI JVMC (MINECRAFT 26.3 / PROTOCOL 777)

---

## I. MỤC TIÊU VÀ NGUYÊN TẮC CỐT LÕI (CORE PRINCIPLES)

1. **Chuẩn mực Parity Tuyệt đối (Bug-for-Bug & Behavioral Parity)**:
   * Mọi hành vi trong game (logic tick, công thức toán học, vận tốc bay, hiệu ứng âm thanh, hạt particle, điều kiện rơi đồ) phải đối chiếu trực tiếp từ mã nguồn decompiled `jvmc` (`E:\DATA\source\svmc\ServerMiecraft\jvmc\src`).
   * Giữ nguyên tên định danh hằng số, tham số và cấu trúc thuật toán của Mojang để đảm bảo tính minh bạch và dễ bảo trì.
2. **Tuân thủ quy chuẩn nghiêm ngặt của dự án Pumpkin**:
   * Không sử dụng `unwrap()`, `expect()`, `panic!()`, `todo!()` hoặc `println!()`. Xử lý lỗi an toàn thông qua `Result` / `Option` và ghi log bằng `tracing`.
   * Vòng lặp gameplay chạy đồng bộ (synchronous) trong tick; mạng, disk I/O chạy trên Tokio async; sinh địa hình và ánh sáng chạy trên Rayon đa luồng.
   * Vượt qua toàn bộ kiểm tra `cargo fmt`, `cargo clippy -p <crate> --all-targets --all-features` và `cargo test`.
3. **Phương pháp thực hiện**:
   * Chia nhỏ thành **9 Giai đoạn (Phases)**, mỗi giai đoạn gồm các bước con cụ thể.
   * Triển khai cuốn chiếu: Thực hiện xong bước nào -> Kiểm tra mã nguồn `jvmc` -> Viết code Rust -> Biên dịch và kiểm thử ngay bước đó.

---

## II. LỘ TRÌNH 9 GIAI ĐOẠN CHI TIẾT (ACTIONABLE ROADMAP)

### GIAI ĐOẠN 1: BỔ SUNG THỰC THỂ CÒN THIẾU & ĐẠN ĐẠO (ENTITIES & PROJECTILES)
> **Mục tiêu**: Đưa tỷ lệ hỗ trợ Entity Types từ **95.0% lên 100%**.

#### 1.1. Hoàn thiện Chai Kinh Nghiệm Ném (`ThrownExperienceBottle`)
* **Đối chiếu `jvmc`**: `net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownExperienceBottle.java` và `net.minecraft.world.item.ExperienceBottleItem.java`.
* **Hiện trạng Pumpkin**: Đang spawn hạt XP trực tiếp tại vị trí mắt người ném trong `experience_bottle.rs`, chưa ném đạn đạo.
* **Các bước chi tiết**:
  * [x] **Bước 1.1.1**: Tạo struct `ExperienceBottleEntity` trong `crates/pumpkin-core/src/entity/projectile/experience_bottle.rs` kế thừa `ThrownItemEntity`.
  * [x] **Bước 1.1.2**: Thiết lập trọng lực chuẩn `const GRAVITY: f64 = 0.07;`.
  * [x] **Bước 1.1.3**: Viết xử lý `on_hit`:
    * Phát âm thanh và hạt vỡ bình thuốc: Gửi packet `CWorldEvent` (ID 2002, màu `-13083194`).
    * Tính toán số lượng XP: `3 + rand(0..5) + rand(0..5)` (từ 3 đến 11 điểm).
    * Gọi `ExperienceOrbEntity::spawn` tại vị trí va chạm và gọi `discard()`.
  * [x] **Bước 1.1.4**: Cập nhật `ExperienceBottleItem` trong `crates/pumpkin-core/src/item/items/experience_bottle.rs`:
    * Tính góc ném xoay chuẩn: pitch offset `-20.0`, tốc độ `0.7`, độ lệch ngẫu nhiên `1.0`.
    * Phát âm thanh `Sound::EntityExperienceBottleThrow` với pitch dao động `0.4 / (rand(0.0..0.4) + 0.8)`.
    * Trừ 1 item trong kho đồ nếu không ở Creative mode.
  * [x] **Bước 1.1.5**: Đăng ký `EntityType::EXPERIENCE_BOTTLE` trong `crates/pumpkin-core/src/entity/type.rs`.

#### 1.2. Đăng ký Cầu Lửa Rồng (`DragonFireballEntity`) & Phao Câu (`FishingBobberEntity`)
* **Hiện trạng Pumpkin**: Đã có file code nhưng chưa đăng ký trong `from_type` của `crates/pumpkin-core/src/entity/type.rs`.
* **Các bước chi tiết**:
  * [x] **Bước 1.2.1**: Ánh xạ `EntityType::DRAGON_FIREBALL.id => Arc::new(DragonFireballEntity::new(entity))` vào `from_type`.
  * [x] **Bước 1.2.2**: Ánh xạ `EntityType::FISHING_BOBBER.id` hỗ trợ khởi tạo mồi câu độc lập khi load world.

#### 1.3. Cài đặt các thực thể còn thiếu
* **Các thực thể**: `leash_knot` (nút thắt dây buộc), `mannequin` (hình nộm), `ominous_item_spawner` (khối sinh vật phẩm thử thách 1.21), `zombie_nautilus`.
* **Các bước chi tiết**:
  * [x] **Bước 1.3.1**: Tạo struct `LeashKnotEntity` trong `crates/pumpkin-core/src/entity/decoration/leash_knot.rs`.
  * [x] **Bước 1.3.2**: Tạo struct `MannequinEntity` trong `crates/pumpkin-core/src/entity/decoration/mannequin.rs`.
  * [x] **Bước 1.3.3**: Tạo struct `OminousItemSpawnerEntity` hỗ trợ thả vật phẩm rơi chậm từ trần buồng Trial Chamber.
  * [x] **Bước 1.3.4**: Đăng ký vào `from_type` trong `crates/pumpkin-core/src/entity/type.rs`.

---

### GIAI ĐOẠN 2: HOÀN THIỆN HÀNH VI VẬT PHẨM (ITEM BEHAVIORS & INTERACTIONS)
> **Mục tiêu**: Đưa tỷ lệ hành vi vật phẩm lên **95%**.

#### 2.1. Hoàn thiện Thẻ Tên (`NameTagItem`)
* **Đối chiếu `jvmc`**: `net.minecraft.world.item.NameTagItem.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity!
* **Các bước chi tiết**:
  * [x] **Bước 2.1.1**: Đọc thuộc tính `custom_name` từ Data Component của thẻ tên.
  * [x] **Bước 2.1.2**: Khi click chuột phải vào `LivingEntity` (không phải Player):
    * Kiểm tra thẻ tên có tên tùy chỉnh hay không.
    * Gán tên `custom_name` cho thực thể, bật cờ `custom_name_visible`.
    * Kích hoạt cờ ngăn thực thể biến mất tự nhiên (`persistence_required = true`).
    * Trừ 1 thẻ tên trong tay (trừ khi đang Creative).

#### 2.2. Hoàn thiện Cần Câu Cá (`FishingRodItem`) & Rơi Đồ
* **Đối chiếu `jvmc`**: `net.minecraft.world.item.FishingRodItem.java` và `net.minecraft.world.entity.projectile.FishingHook.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity!
* **Các bước chi tiết**:
  * [x] **Bước 2.2.1**: Kết nối bảng loot `gameplay/fishing` từ `pumpkin-data` khi kéo cần câu dính cá.
  * [x] **Bước 2.2.2**: Tạo thực thể `ItemEntity` bay từ vị trí phao câu về phía người chơi.
  * [x] **Bước 2.2.3**: Trừ độ bền cần câu tương ứng (1 điểm khi kéo rác/cá, 5 điểm khi kéo móc vào khối/quái vật).

#### 2.3. Hoàn thiện Cây Chùy (`MaceItem`) & Smash Attack
* **Đối chiếu `jvmc`**: `net.minecraft.world.item.MaceItem.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity!
* **Các bước chi tiết**:
  * [x] **Bước 2.3.1**: Kiểm tra điều kiện rơi tự do `fall_distance > 1.5` khối khi vung chùy tấn công.
  * [x] **Bước 2.3.2**: Tính toán sát thương cộng dồn: `damage = base_damage + (bonus_per_block * fall_distance)` + Density.
  * [x] **Bước 2.3.3**: Triệt tiêu toàn bộ sát thương rơi của người cầm chùy khi đánh trúng mục tiêu, bật nảy lên.
  * [x] **Bước 2.3.4**: Tạo luồng gió đẩy lùi xung quanh (AOE knockback 3.5m) và âm thanh smash heavy/ground/air chuẩn 26.3.

#### 2.4. Sáp Tổ Ong (`HoneycombItem`) cho Khối Đồng & Cửa Sập
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity!
* **Các bước chi tiết**:
  * [x] **Bước 2.4.1**: Bảo tồn toàn bộ block states (hướng xoay, trạng thái mở/đóng, cấp điện, ngập nước) khi wax đồng.

---

### GIAI ĐOẠN 3: HOÀN THIỆN KHỐI & GAME EVENTS (BLOCKS & SCULK VIBRATIONS)
> **Mục tiêu**: Đưa tỷ lệ Block Behaviors từ **33.6% lên 75%** và kích hoạt Game Events.

#### 3.1. Kích hoạt Game Events cho Khối Âm Thanh & Rung Động
* **Đối chiếu `jvmc`**: `net.minecraft.world.level.gameevent.GameEvent.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện!
* **Các bước chi tiết**:
  * [x] **Bước 3.1.1**: Gọi `world.emit_game_event(GameEvent::BlockChange, pos)` khi Jukebox đổi đĩa.
  * [x] **Bước 3.1.2**: Gọi `world.emit_game_event(GameEvent::BlockActivate/Deactivate, pos)` khi Chuông rung hoặc Bẫy dây kéo.
  * [x] **Bước 3.1.3**: Gọi `world.emit_game_event(GameEvent::ContainerOpen/Close, pos)` khi mở/đóng Shulker Box và `EntityDismount`.

#### 3.2. Hoàn thiện Khối Chế Tạo Tự Động (`Crafter`)
* **Đối chiếu `jvmc`**: `net.minecraft.world.level.block.CrafterBlock.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity!
* **Các bước chi tiết**:
  * [x] **Bước 3.2.1**: Cho phép người chơi click chuột để vô hiệu hóa / kích hoạt từng ô trong lưới 3x3 (`disabled_slots`).
  * [x] **Bước 3.2.2**: Khi nhận xung Redstone:
    * Kiểm tra lưới đồ khớp với công thức hợp lệ nào.
    * Bắn sản phẩm ra mặt trước theo hướng đặt khối (`CrafterCraftEvent` & `ItemEntity`).
    * Chuyển trạng thái sang `crafting = true` và kích hoạt cooldown 4 ticks (`MAX_CRAFTING_TICKS = 6`).

---

### GIAI ĐOẠN 4: ĐỒNG BỘ REDSTONE PARITY & MICRO-TICKS
> **Mục tiêu**: Đạt tỷ lệ Redstone Parity **92%**.

#### 4.1. Thứ tự Cập nhật Dây Redstone & Cấp điện Gián tiếp (Quasi-Connectivity)
* **Đối chiếu `jvmc`**: `net.minecraft.world.level.block.RedstoneWireBlock.java` và `DefaultRedstoneWireEvaluator.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity!
* **Các bước chi tiết**:
  * [x] **Bước 4.1.1**: Áp dụng thuật toán tính toán nguồn phát Redstone đồng bộ `DefaultRedstoneWireEvaluator` với hướng cập nhật chuẩn.
  * [x] **Bước 4.1.2**: Đồng bộ cơ chế cấp điện gián tiếp (Quasi-Connectivity - QC) cho Piston (`piston.rs`), Dropper (`dropper.rs`), Dispenser (`dispenser.rs`), Cửa và Cửa sập kiểm tra nguồn phát gián tiếp từ khối phía trên (`pos.up()`).

#### 4.2. Độ trễ & Chu kỳ Piston (Piston Timings)
* **Đối chiếu `jvmc`**: `net.minecraft.world.level.block.piston.PistonBaseBlock.java` và `PistonStructureResolver.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity!
* **Các bước chi tiết**:
  * [x] **Bước 4.2.1**: Hoàn thiện thời gian trễ mở rộng/thu hồi (2 game ticks) và khối mở rộng di động `PistonBlockEntity`.
  * [x] **Bước 4.2.2**: Xử lý va chạm khối slime và khối mật ong đẩy kéo các khối liền kề (giới hạn `MAX_MOVABLE_BLOCKS = 12`, phân tách slime-honey không dính nhau, chặn các khối có TileEntity/impassable).

---

### GIAI ĐOẠN 5: CẤU TRÚC THẾ GIỚI JIGSAW (WORLDGEN STRUCTURES)
> **Mục tiêu**: Đưa tỷ lệ công trình thế giới lên **95%**.
* **Đối chiếu `jvmc`**: `net.minecraft.world.level.levelgen.structure.structures.JigsawStructure.java`.
* **Hiện trạng Pumpkin**: Đã hoàn thiện full vanilla parity qua `JigsawGenerator` và bộ mẫu `TemplatePool`!
* **Các bước chi tiết**:
  * [x] **Bước 5.1**: Triển khai `TrialChambers` (`StructureKeys::TrialChambers` qua Jigsaw assembly).
  * [x] **Bước 5.2**: Triển khai `AncientCity` (`StructureKeys::AncientCity` qua `minecraft:ancient_city/city_center`).
  * [x] **Bước 5.3**: Triển khai `BastionRemnant` (`StructureKeys::BastionRemnant` qua 4 phân khu Jigsaw pools).
  * [x] **Bước 5.4**: Triển khai `PillagerOutpost` (`StructureKeys::PillagerOutpost` qua `minecraft:pillager_outpost/base_plates`).

---

### GIAI ĐOẠN 6: NÂNG CẤP TRÍ TUỆ NHÂN TẠO MOBS (MOB AI & BRAIN FRAMEWORK)
> **Mục tiêu**: Đưa tỷ lệ Mob AI từ **60% lên 90%**.

* **Các bước chi tiết**:
  * [x] **Bước 6.1**: Hoàn thiện AI Warden: Emerge trồi lên 134 ticks invulnerable, đòn quét sóng siêu thanh Sonic Boom (34 ticks sạc, luồng hạt particle + âm thanh + 10 sát thương xuyên giáp), đánh hơi Sniffing (84 ticks dò tìm người chơi trong 16m), rống gầm Roaring (84 ticks, âm thanh tại tick 25), đào đất chui xuống Digging (100 ticks, invulnerable rồi despawn), nhịp tim đập Heartbeat (30-60 ticks), và rung râu Tendrils shiver (mỗi 80 ticks kèm âm thanh click và packet status 61).
  * [x] **Bước 6.2**: Hoàn thiện AI Breeze: Nhảy lướt né đòn khi mục tiêu áp sát (< 4m), hít thở tích điện 15 ticks, bắn đạn gió Wind Charge nổ đẩy lùi xung quanh, trượt chân tiếp đất, và kháng đạn gió lẫn nhau (`BreezeAttackGoal`).
  * [x] **Bước 6.3**: Hoàn thiện AI Creaking: Khóa bất động khi người chơi nhìn thẳng vào mặt (`CAN_MOVE` tracked data), miễn nhiễm mọi sát thương, kết nối trái tim Creaking Heart (`CreakingHeartBlockEntity`).
  * [x] **Bước 6.4**: Hoàn thiện các biến thể Bộ Xương (Skeleton Parity):
    * **Bogged (`BoggedSkeletonEntity`)**: Tốc độ bắn chậm chuẩn vanilla (khoảng cách giữa các phát bắn 70 ticks / 50 ticks Hard), bắn tên độc `TIPPED_ARROW` chứa hiệu ứng Poison, hỗ trợ xén nấm bằng Kéo (`Item::SHEARS`) rơi nấm đỏ & nấm nâu chuẩn loot table `shearing/bogged`, trừ độ bền kéo, cập nhật tracked data `DATA_SHEARED` đồng bộ với client, lưu NBT tag `"sheared"`, âm thanh `EntityBoggedHurt`, `EntityBoggedDeath`, `EntityBoggedAmbient`, `EntityBoggedShear`.
    * **Stray (`StraySkeletonEntity`)**: Bắn tên chậm `TIPPED_ARROW` chứa hiệu ứng Slowness (16 Slowness ID), âm thanh đặc thù `EntityStrayHurt`, `EntityStrayDeath`, `EntityStrayAmbient`.
    * **Wither Skeleton (`WitherSkeletonEntity`)**: Mặc định cầm Kiếm Đá (`Item::STONE_SWORD`) ở tay chính và không mang giáp, chiến đấu cận chiến `MeleeAttackGoal` (tốc độ 1.2), sát thương cơ bản 4.0, tự động chủ động nhắm đánh Piglin (`EntityType::PIGLIN` và `EntityType::PIGLIN_BRUTE`), gây hiệu ứng Wither 200 ticks (10 giây) khi đánh trúng mục tiêu, miễn nhiễm sát thương và hiệu ứng Wither (`DamageType::WITHER`), bắn tên lửa (flame) khi cầm cung, âm thanh `EntityWitherSkeletonHurt`, `EntityWitherSkeletonDeath`, `EntityWitherSkeletonAmbient`.
    * **Skeleton (`SkeletonEntity`)**: Hệ thống âm thanh tùy biến `CustomSound` (`EntitySkeletonHurt`, `EntitySkeletonDeath`) và âm thanh môi trường `EntitySkeletonAmbient`.
  * [x] **Bước 6.5**: Hoàn thiện các biến thể Xác Sống (Zombie Parity):
    * **Husk (`HuskEntity`)**: Không bị cháy dưới ánh sáng ban ngày (`burns_in_daylight = false`), đòn đánh không vũ khí gây hiệu ứng Đói bụng Hunger thời gian tỉ lệ theo độ khó (`140 * difficulty` ticks: 7s Easy, 14s Normal, 21s Hard), âm thanh đặc thù `EntityHuskHurt`, `EntityHuskDeath`, `EntityHuskAmbient`.
    * **Zombie (`ZombieEntity`)**: Hệ thống âm thanh tùy biến `CustomSound` (`EntityZombieHurt`, `EntityZombieDeath`) và âm thanh môi trường `EntityZombieAmbient`.
  * [x] **Bước 6.6**: Hoàn thiện họ Nhện (Spider & Cave Spider Parity):
    * **Spider (`SpiderEntity`)**: Cơ chế leo tường đồng bộ tracked data `DATA_FLAGS_ID` (bit 1), hệ thống âm thanh `CustomSound` (`EntitySpiderHurt`, `EntitySpiderDeath`), âm thanh môi trường `EntitySpiderAmbient` trong `mob_tick`, nhảy vồ `LeapAtTargetGoal`, bầy nhện hiệu ứng spawn buff (Speed, Strength, Regeneration, Invisibility trên độ khó Hard), cơ chế Spider Jockey (1% sinh ra Skeleton cưỡi trên lưng).
    * **Cave Spider (`CaveSpiderEntity`)**: Đòn đánh cận chiến tiêm chất độc Poison (7s Normal, 15s Hard), kích thước nhỏ gọn 0.7x0.5, cơ chế leo tường đồng bộ tracked data `DATA_FLAGS_ID`, hệ thống âm thanh `CustomSound` (`EntitySpiderHurt`, `EntitySpiderDeath`), và âm thanh môi trường `EntitySpiderAmbient`.

---

### GIAI ĐOẠN 7: BỔ SUNG CÁC LỆNH CÒN THIẾU (COMMANDS PARITY)
> **Mục tiêu**: Đưa tỷ lệ lệnh Vanilla từ **81.9% lên 98%**.

* **Các bước chi tiết**:
  * [x] **Bước 7.1**: Bổ sung `/setplayeridletimeout` (bí danh chuẩn hóa của `/setidletimeout` qua `register_with_aliases`).
  * [x] **Bước 7.2**: Bổ sung `/setspawn` (bí danh chuẩn hóa của `/spawnpoint` qua `register_with_aliases`).
  * [x] **Bước 7.3**: Bổ sung lệnh `/warden_spawn_tracker` (`clear` & `set <warning_level: 0..=4> [targets]`) hỗ trợ quản lý cảnh báo Warden.
  * [x] **Bước 7.4**: Bổ sung lệnh `/serverpack` (`push <url> [uuid] [hash]` & `pop <uuid>`) đồng bộ phát tài nguyên gói Server.
  * [x] **Bước 7.5**: Bổ sung `/spawn_armor_trims` (`*_lag_my_game` & `<pattern>`).
  * [x] **Bước 7.6**: Hoàn thiện các điều kiện lọc nâng cao của `/execute` (`/execute positioned over`, `/execute summon`, `/execute on`).
  * [x] **Bước 7.7**: Bổ sung lệnh `/version` chuẩn hóa Minecraft 26.3 (`VersionCommand.java`).

---

### GIAI ĐOẠN 8: ĐỒNG BỘ GIAO THỨC BEDROCK (BEDROCK PARITY)
> **Mục tiêu**: Đưa tỷ lệ Bedrock từ **45% lên 85%**.

* **Các bước chi tiết**:
  * [x] **Bước 8.1**: Hoàn thiện gói tin kho đồ Bedrock `ItemStackResponsePacket` (ID 148) và `ContainerClosePacket` (ID 47 `CContainerClose`).
  * [x] **Bước 8.2**: Đồng bộ chuyển động mượt mà giữa Client Bedrock và Server Tokio.

---

### GIAI ĐOẠN 9: TIẾN TRÌNH & THỐNG KÊ (ADVANCEMENTS & STATS)
> **Mục tiêu**: Đưa tỷ lệ hoàn thiện hệ thống phụ trợ từ **65% lên 90%**.

* **Các bước chi tiết**:
  * [x] **Bước 9.1**: Gắn hook trigger cho toàn bộ các hành động gameplay: `slept_in_bed` (khi ngủ trên giường), `bred_animals` (khi phối giống động vật & sniffer), `fished_item` (khi câu cá thành công).
  * [x] **Bước 9.2**: Ghi nhận thống kê người chơi chi tiết theo danh mục (`SleepInBed`, `AnimalsBred`, `FishCaught`, `DamageDealt`, `MobKills`).
  * [x] **Bước 9.3**: Cập nhật `MAXIMUM_SUPPORTED_WORLD_DATA_VERSION = 5023` đồng bộ 100% với Minecraft 26.3 (`SharedConstants.WORLD_VERSION = 5023`).
