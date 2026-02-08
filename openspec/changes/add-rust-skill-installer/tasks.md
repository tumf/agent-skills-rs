# 作業タスク

1. ライブラリのコア型とエージェント設定の雛形を追加する
   - 成果物: `types` と `agents` モジュール（Skill/Agent/Source/LockEntry）
   - 検証: `cargo check` が通る、または対象ファイルに型定義が存在する

2. スキル発見ロジックを実装する（探索ディレクトリ + SKILL.md パース）
   - 成果物: `discovery` モジュール、frontmatter パース、internal スキルのフィルタ
   - 検証: ユニットテストで `skills/` と `.agents/skills/` を優先検索し、`metadata.internal` が除外される

3. インストールロジックを実装する（canonical + symlink/copy）
   - 成果物: `installer` モジュール、path traversal 対策、symlink フォールバック
   - 検証: テンポラリディレクトリを使うテストで `symlink` と `copy` の両方が動作する

4. ロックファイル管理を実装する（globalのみ記録）
   - 成果物: `.skill-lock.json` の read/write と version 管理
   - 検証: ロックファイル更新時に `skill_folder_hash` が保存される

5. 外部依存の抽象化とモック実装を追加する
   - 成果物: `providers` トレイト（clone/fetch/hash）、モック実装
   - 検証: モックのみで discovery → install → lock の一連テストが通る

6. CLIの `install-skill` を実装しライブラリに接続する
   - 成果物: `my-command install-skill` の実行経路
   - 検証: `--yes` で非対話実行でき、`--agent`/`--skill` で絞り込み可能

7. Introspection コマンドを実装する
   - 成果物: `my-command commands --output json` / `schema --command install-skill --output json-schema`
   - 検証: JSON出力が `schemaVersion`, `type`, `ok` を含む

8. 最小限の統合テストを追加する（外部API無し）
   - 成果物: モックprovider + temp dir による end-to-end テスト
   - 検証: `cargo test` が通る
