# Tasks

1. install-skill のターゲットディレクトリ解決をスコープ対応に更新する
   - 対象: `resolve_target_dirs` のロジック
   - 変更内容: OpenCode は project スコープで追加 target dir を作らず、global スコープ時のみ `~/.config/opencode/skills` を返す
   - 検証: `resolve_target_dirs` のユニットテストを追加/更新して project/global の分岐を確認する

2. install-skill のインストール呼び出しを canonical + target_dirs の一回呼びに統一する
   - 対象: `install_skill_command` のインストールフロー
   - 変更内容: `InstallConfig.target_dirs` に target dir を設定し、`install_skill` を一度だけ呼ぶ
   - 検証: `InstallConfig` の利用箇所とフローをコードレビューで確認する

3. エージェント別インストールの動作テストを追加する
   - 対象: `my_command` のテスト
   - 変更内容: TempDir を使い canonical と target dir の両方が生成され、target が symlink になることを確認する（Unix 環境でのみ）
   - 検証: `cargo test` で新規テストが通ることを確認する

## Future work

- CLI 出力文言の手動確認（canonical/target の説明が誤解されないかの目視チェック）
