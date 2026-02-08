# タスク

1. 旧形式ロックファイルのデータ構造を追加する
   - 目的: 旧形式（配列形式）をパースできるようにする
   - 完了条件: `src/types.rs` に旧形式用の構造体と判別用 enum が追加される
   - 検証: `cargo test types::tests::test_legacy_lock_format_migration`

2. SkillLock のデシリアライズを新旧両対応にする
   - 目的: 旧形式を新形式へ変換して扱えるようにする
   - 完了条件: `SkillLock` のデシリアライズで旧形式の読み込みが成功する
   - 検証: `cargo test types::tests::test_new_lock_format_still_works`

3. 旧形式ロックファイルの読み込みテストを追加する
   - 目的: 実際のファイル読み込みで自動マイグレーションされることを保証する
   - 完了条件: `src/lock.rs` に旧形式読み込みテストが追加される
   - 検証: `cargo test lock::tests::test_load_legacy_array_format`

4. 保存後の新形式永続化を確認する
   - 目的: 旧形式の読み込み後、保存で新形式が書き出されることを担保する
   - 完了条件: テストで再読込後に `version` が `"1.0"` であることを確認できる
   - 検証: `cargo test lock::tests::test_load_legacy_array_format`
