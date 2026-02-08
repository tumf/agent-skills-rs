# 作業タスク

- [x] 1. コア型を拡張し、埋め込みソース種別を正式化する
   - 成果物: `Source.type` に `self` / `embedded` を追加し、同義として扱う仕様を反映
   - 検証方法: 型定義とCLI入力定義に `self` と `embedded` の両方が存在することを静的確認する

- [x] 2. 埋め込みスキル定義モジュールを追加する（`include_str!` 相当）
   - 成果物: コンパイル時に `SKILL.md` 等を読み込む `embedded` モジュール（実コンテンツをバイナリに同梱）
   - 検証方法: ユニットテストで埋め込み定義が空でないこと、必須frontmatter（`name`, `description`）を満たすことを確認する

- [x] 3. discovery に self/embedded 分岐を追加する
   - 成果物: `install-skill self|embedded` 指定時に、ファイルシステム探索ではなく埋め込み定義を返す経路
   - 検証方法: ユニットテストで `self` 指定時に外部provider呼び出しが0回であることを検証する

- [x] 4. installer の canonical/symlink/copy 流れへ埋め込み経路を統合する
   - 成果物: 埋め込み由来でも通常ソースと同一の後段処理（canonical配置、symlink失敗時copyフォールバック）を実行
   - 検証方法: テンポラリディレクトリ統合テストで `install-skill self --yes` が成功し、配置結果が期待通りであることを確認する

- [x] 5. lock 管理に埋め込みソースの記録ルールを追加する
   - 成果物: global インストール時に `sourceType=embedded`（または self同義）と決定的ハッシュを保存
   - 検証方法: ロックファイル検証テストで `skill_folder_hash` が保存され、外部通信不要で再現可能であることを確認する

- [x] 6. CLI 経路を拡張し `my-command install-skill self` / `embedded` をサポートする
   - 成果物: `install-skill` の引数解決ロジックと非対話実行（`--yes` / `--non-interactive`）
   - 検証方法: CLIテストで `my-command install-skill self --yes` と `my-command install-skill embedded --yes` が等価に成功することを確認する

- [x] 7. Introspection 出力を更新し schema に self/embedded を反映する
   - 成果物: `commands --output json` / `schema --command install-skill --output json-schema` の更新
   - 検証方法: JSON Schema 検証で source 列挙値に `self` と `embedded` が含まれることを確認する

- [x] 8. mock-first 方針で end-to-end テストを整備する（外部通信なし）
   - 成果物: モックprovider + temp dir による discovery → install → lock の一連テスト
   - 検証方法: テスト実行時にネットワークアクセスなしで `cargo test` が通ることを確認する

## Future work

1. GitHub/GitLab 実通信を使ったE2E検証
   - 理由: 外部APIトークン・レート制限・CI環境差分に依存し、提案段階およびAI単独実行で再現性を担保しづらいため
   - 完了条件: 専用テスト環境で認証付き実通信E2Eを追加し、mockテストとの差分を監査可能にする

## Acceptance #1 Failure Follow-up

- [x] `discover_skills` が非埋め込みソースで常に空配列を返しており、優先探索ディレクトリ走査（`skills/`, `.agents/skills/`, `.claude/skills/` など）と `metadata.internal` フィルタ要件を満たしていないため、ファイルシステム探索と frontmatter 検証を実装する（根拠: `src/discovery.rs:27`）。
- [x] `commands --output json` の `type` が `commands.list` ではなく `commands` になっているため、要件どおり `commands.list` を返すように修正し、テストを更新する（根拠: `src/cli.rs:131`、実行出力）。
- [x] `schema --command install-skill --output json-schema` が `--agent` / `--skill` / `--global` など仕様上の引数を公開していないため、`install-skill` の引数定義と JSON Schema 出力を仕様に合わせて拡張する（根拠: `src/cli.rs:59`、実行出力）。
- [x] ロック記録が常に `~/.agents/.skill-lock.json` に行われ、`global` のみ記録する要件とローカル非記録要件を満たせていないため、スコープ（global/local）を CLI とインストールフローに導入して条件分岐を実装する（根拠: `src/bin/my_command.rs:98`、`src/bin/my_command.rs:139`）。
- [x] 埋め込みソースの `source_type` が `embedded`/`self` ではなく `self_` で記録されるため、ロック保存時の source type 正規化を実装する（根拠: `src/lock.rs:60`、`/tmp/skill-installer-accept-home/.agents/.skill-lock.json:6`）。
- [x] symlink 失敗時 fallback は実装されているが `symlinkFailed = true` を返す結果モデルが存在せず要件シナリオを満たせないため、インストール結果型に `symlink_failed` を追加して返却・利用する（根拠: `src/installer.rs:34`、`src/installer.rs:91`）。
