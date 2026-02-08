# 作業タスク

1. コア型を拡張し、埋め込みソース種別を正式化する
   - 成果物: `Source.type` に `self` / `embedded` を追加し、同義として扱う仕様を反映
   - 検証方法: 型定義とCLI入力定義に `self` と `embedded` の両方が存在することを静的確認する

2. 埋め込みスキル定義モジュールを追加する（`include_str!` 相当）
   - 成果物: コンパイル時に `SKILL.md` 等を読み込む `embedded` モジュール（実コンテンツをバイナリに同梱）
   - 検証方法: ユニットテストで埋め込み定義が空でないこと、必須frontmatter（`name`, `description`）を満たすことを確認する

3. discovery に self/embedded 分岐を追加する
   - 成果物: `install-skill self|embedded` 指定時に、ファイルシステム探索ではなく埋め込み定義を返す経路
   - 検証方法: ユニットテストで `self` 指定時に外部provider呼び出しが0回であることを検証する

4. installer の canonical/symlink/copy 流れへ埋め込み経路を統合する
   - 成果物: 埋め込み由来でも通常ソースと同一の後段処理（canonical配置、symlink失敗時copyフォールバック）を実行
   - 検証方法: テンポラリディレクトリ統合テストで `install-skill self --yes` が成功し、配置結果が期待通りであることを確認する

5. lock 管理に埋め込みソースの記録ルールを追加する
   - 成果物: global インストール時に `sourceType=embedded`（または self同義）と決定的ハッシュを保存
   - 検証方法: ロックファイル検証テストで `skill_folder_hash` が保存され、外部通信不要で再現可能であることを確認する

6. CLI 経路を拡張し `my-command install-skill self` / `embedded` をサポートする
   - 成果物: `install-skill` の引数解決ロジックと非対話実行（`--yes` / `--non-interactive`）
   - 検証方法: CLIテストで `my-command install-skill self --yes` と `my-command install-skill embedded --yes` が等価に成功することを確認する

7. Introspection 出力を更新し schema に self/embedded を反映する
   - 成果物: `commands --output json` / `schema --command install-skill --output json-schema` の更新
   - 検証方法: JSON Schema 検証で source 列挙値に `self` と `embedded` が含まれることを確認する

8. mock-first 方針で end-to-end テストを整備する（外部通信なし）
   - 成果物: モックprovider + temp dir による discovery → install → lock の一連テスト
   - 検証方法: テスト実行時にネットワークアクセスなしで `cargo test` が通ることを確認する

## Future work

1. GitHub/GitLab 実通信を使ったE2E検証
   - 理由: 外部APIトークン・レート制限・CI環境差分に依存し、提案段階およびAI単独実行で再現性を担保しづらいため
   - 完了条件: 専用テスト環境で認証付き実通信E2Eを追加し、mockテストとの差分を監査可能にする
