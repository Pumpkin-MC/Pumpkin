# KẾ HOẠCH TỔNG THỂ CÁC GIAI ĐOẠN HOÀN THIỆN SERVER 100% VANILLA PARITY (JVMC 26.3)

## QUY TẮC ĐIỀU HÀNH BẮT BUỘC (CONTROL & PROTOCOL ENFORCEMENT)
1. **Một bước mỗi lượt**: Mỗi lần người dùng ra lệnh `"làm tiếp theo"`, hệ thống CHỈ ĐƯỢC PHÉP THI HÀNH DUY NHẤT 1 BƯỚC VI MÔ TIẾP THEO trong danh sách.
2. **Tiêu chuẩn chất lượng**: Mỗi bước sau khi hoàn thiện code phải vượt qua `cargo check` với 0 lỗi và được đánh dấu hoàn thành `[x]`.
3. **Quy tắc cãi lại & Dừng dứt khoát (Refusal Rule)**: Khi toàn bộ các bước từ Bước 1 đến Bước 14 đã hoàn thành (`[x]`), nếu người dùng tiếp tục gõ `"làm tiếp theo"`, hệ thống **BẮT BUỘC PHẢI CÃI LẠI VÀ TỪ CHỐI**, không làm thêm bất kỳ thao tác nào vì server đã hoàn thành 100%.

---

## GIAI ĐOẠN I: CHIẾN ĐẤU & PHÒNG THỦ NÂNG CAO (COMBAT & SHIELD PARITY)
- [x] **Bước 1**: Triển khai cơ chế Axe vô hiệu hóa Khiên (Shield Disable / Axe Stun) chuẩn vanilla `Weapon.java` (5.0s / 100 ticks cooldown, phát âm thanh `ItemShieldBreak`, ngắt trạng thái đỡ `clear_active_hand`, và gửi gói cooldown tới client).
- [x] **Bước 2**: Đồng bộ kiểm tra Cooldown vào phương thức `LivingEntity::is_blocking` (nếu vật phẩm Khiên đang trong thời gian hồi chiêu `is_on_cooldown`, người chơi không thể vào tư thế đỡ đòn).
- [x] **Bước 3**: Cập nhật tính toán sát thương quét kiếm (Sweeping Edge) chuẩn xác theo tỷ lệ thuộc tính `player.sweeping_damage_ratio` (mặc định 0.0 nếu không có enchantment, tăng lên theo cấp độ Sweeping Edge: cấp 1 = 50%, cấp 2 = 66.7%, cấp 3 = 75% sát thương gốc).

## GIAI ĐOẠN II: TƯƠNG TÁC VẬT PHẨM & MÔI TRƯỜNG (ITEMS & MECHANICS PARITY)
- [x] **Bước 4**: Cơ chế dịch chuyển quả Chorus Fruit (`ChorusFruitItem`): dịch chuyển ngẫu nhiên người chơi trong bán kính 8 block và phát âm thanh `ItemChorusFruitTeleport`.
- [x] **Bước 5**: Cơ chế La bàn Lodestone (`CompassItem`): đọc dữ liệu vị trí và chiều không gian (Dimension) từ NBT `LodestoneDimension` & `LodestonePos` để định hướng kim la bàn.
- [x] **Bước 6**: Cơ chế Đồng hồ (`ClockItem`): đồng bộ góc quay kim đồng hồ theo chu kỳ thời gian trong ngày (`world_time % 24000`) và vô hiệu hóa / quay loạn xạ trong Nether/The End.

## GIAI ĐOẠN III: QUẢN LÝ KHO ĐỒ & MÀN HÌNH GUI (INVENTORY & CRAFTING PARITY)
- [x] **Bước 7**: Dọn sạch vật phẩm trong ô chế tạo (Crafting 2x2 / 3x3) khi người chơi đóng giao diện hoặc ngắt kết nối, tự động nhả (drop) đồ ra ngoài đất hoặc hoàn lại vào hòm đồ cá nhân.
- [x] **Bước 8**: Ngăn chặn người chơi cất Hộp Shulker (Shulker Box) lồng vào trong Hộp Shulker khác qua giao diện GUI hoặc kéo thả chuột.

## GIAI ĐOẠN IV: THỰC THỂ SINH VẬT & DÂN LÀNG (ENTITIES & AI PARITY)
- [x] **Bước 9**: Cập nhật hồi phục giao dịch Dân làng (Villager Restock): Dân làng làm mới số lượt mua bán tại bàn làm việc (Workstation) tối đa 2 lần mỗi ngày game.
- [x] **Bước 10**: Cơ chế sấm sét kích hoạt biến đổi Phù thủy (Witch): Dân làng bị sét đánh trực tiếp lập tức biến đổi thành `Witch` (tương tự như Heo biến thành Zombified Piglin).

## GIAI ĐOẠN V: KHỐI CHỨC NĂNG & BLOCK ENTITIES (BLOCK ENTITIES & REDSTONE PARITY)
- [x] **Bước 11**: Khối Điêu khắc Rung động Sculk Shrieker: Tăng cấp độ cảnh báo (Warning Level 1..4) cho người chơi khi nhận rung động từ Sculk Sensor và triệu hồi Warden ở cấp 4.
- [x] **Bước 12**: Khối Hầm ngục Thử thách Trial Spawner / Vault: Xử lý tương tác chìa khóa Trial Key / Ominous Trial Key để nhả vật phẩm thưởng từ Loot Table.

## GIAI ĐOẠN VI: KIỂM THỬ TOÀN DIỆN & KHÓA HỆ THỐNG (FINAL VALIDATION & ENFORCEMENT)
- [x] **Bước 13**: Biên dịch kiểm tra toàn bộ workspace `cargo check --workspace --all-targets` đảm bảo 100% không còn lỗi biên dịch.
- [x] **Bước 14**: Khóa hoàn tất hệ thống toàn diện, kích hoạt cơ chế từ chối và cãi lại bất kỳ yêu cầu "làm tiếp theo" nào từ người dùng.
