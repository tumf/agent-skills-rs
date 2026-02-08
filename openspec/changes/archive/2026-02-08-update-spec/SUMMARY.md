# 実装完了サマリ: update-spec

## 完了タスク

すべてのタスク（7/7）が完了しました：

- [x] 1. `openspec/specs/skill-installer/spec.md` から `self`/`embedded` 引数関連の記述を削除
- [x] 2. `openspec/specs/skill-installer/spec.md` の `--global` デフォルト値を `false` に更新
- [x] 3. `openspec/specs/skill-installer/spec.md` のロックファイル更新要件を「project/global 両方」に更新
- [x] 4. `openspec/specs/cli-introspection/spec.md` から `self`/`embedded` スキーマ記述を削除
- [x] 5. `openspec/specs/cli-introspection/spec.md` に `--global` デフォルト動作を追加
- [x] 6. すべての Scenario が現在の実装と一致することを確認
- [x] 7. `cargo test` で仕様書の要件が検証可能であることを確認

## 変更されたファイル

### 仕様書の更新
- `openspec/specs/skill-installer/spec.md`
  - 埋め込みスキルのインストール要件を更新（引数なしで自動解決）
  - インストールスコープのデフォルト要件を追加（project-local がデフォルト）
  - ロックファイル更新要件を更新（project/global 両方）
  
- `openspec/specs/cli-introspection/spec.md`
  - `self`/`embedded` スキーマ記述を削除
  - `--global` デフォルト動作を明記

### 変更管理ファイル
- `openspec/changes/update-spec/proposal.md` - 提案書
- `openspec/changes/update-spec/design.md` - 設計書
- `openspec/changes/update-spec/tasks.md` - タスク管理
- `openspec/changes/update-spec/SUMMARY.md` - 本サマリ

## 検証結果

### コード品質
- ✅ `cargo fmt --all --check` - フォーマット OK
- ✅ `cargo clippy --all-targets -- -D warnings` - 警告なし
- ✅ `cargo test` - 50/50 テスト成功

### 仕様の整合性
すべての Scenario が現在の実装と一致：

1. **埋め込みスキルの自動解決**
   - `my-command install-skill --yes` で外部 provider を呼ばずに埋め込みスキルをインストール

2. **デフォルトは project-local インストール**
   - `--global` なしで `./.agents/skills/` と `./.agents/.skill-lock.json` を使用

3. **--global 指定で global インストール**
   - `--global` 指定で `~/.agents/skills/` と `~/.agents/.skill-lock.json` を使用

4. **ロックファイル更新**
   - project/global 両方でスコープに応じたパスにロックファイルを記録

## 今後の対応

この変更は仕様書のみの更新であり、コードへの影響はありません。  
すべてのタスクが完了し、検証も成功しているため、アーカイブ可能な状態です。
