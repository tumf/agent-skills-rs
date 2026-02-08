# 提案: 仕様書を実装に合わせて更新

## 概要
最近のコード変更（埋め込みスキル専用化、project-local デフォルト化）を反映し、`openspec/specs/` 配下の仕様書を最新の実装に合わせて更新する。

## 背景
以下の実装変更が行われたが、仕様書がまだ古い内容を参照している：

1. **`install-skill` コマンドから `source` 引数を削除**
   - 常に埋め込みスキル（`SourceType::Self_`）をインストールするように変更
   - `self` / `embedded` の選択肢は不要になった

2. **`--global` のデフォルト値を変更**
   - 旧: `--global` のデフォルトは `true`（global インストール）
   - 新: `--global` のデフォルトは `false`（project-local インストール）
   - `--global` フラグを明示的に指定したときのみ global スコープになる

3. **ロックファイル更新の動作変更**
   - 旧: global スコープのときのみロックファイルを更新
   - 新: project / global 両方でロックファイルを更新（パスが異なる）
     - project: `./.agents/.skill-lock.json`
     - global: `~/.agents/.skill-lock.json`

## 提案内容
`openspec/specs/skill-installer/spec.md` と `openspec/specs/cli-introspection/spec.md` を以下のように更新する：

### skill-installer/spec.md の変更
- **削除**: `self` / `embedded` 引数に関する記述（現在は引数なしで常に埋め込みスキルをインストール）
- **更新**: `--global` のデフォルト値を `false`（project-local）に変更
- **更新**: ロックファイル更新の要件を「global のみ」から「project/global 両方（パスが異なる）」に変更

### cli-introspection/spec.md の変更
- **削除**: `self` / `embedded` が有効値として含まれるという記述
- **追加**: `--global` のデフォルト動作に関する記述

## 受入基準
- [ ] `openspec/specs/skill-installer/spec.md` が最新の実装を正確に反映している
- [ ] `openspec/specs/cli-introspection/spec.md` が最新の実装を正確に反映している
- [ ] すべての Scenario が現在の動作と一致している
- [ ] 仕様書に記載された要件が `cargo test` で検証可能である
