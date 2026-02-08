# タスク

1. CLI の `--agent` をカンマ区切り + 複数回指定に対応
   - 変更箇所: `src/bin/my_command.rs`
   - 完了確認: `cargo run --bin my-command -- schema --command install-skill --output json-schema` の説明文に複数指定が記載される

2. agent 文字列の正規化と `target_dirs` 解決を追加
   - 変更箇所: `src/bin/my_command.rs`
   - 完了確認: `--agent claude,opencode` で `.claude/skills` と `.config/opencode/skills` への配布ログが出ること

3. 未知 agent の fail-fast エラーを追加
   - 変更箇所: `src/bin/my_command.rs`
   - 完了確認: `--agent unknown` 実行時にエラーが返ること

4. イントロスペクションの説明文を更新
   - 変更箇所: `src/cli.rs`
   - 完了確認: `cargo run --bin my-command -- schema --command install-skill --output json-schema` に反映される

5. 単体テスト追加（正規化・重複除外・複数指定）
   - 変更箇所: `src/bin/my_command.rs` または `src/cli.rs` のテストモジュール
   - 完了確認: `cargo test` が成功する
