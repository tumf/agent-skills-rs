# 変更提案: install-skill のシンボリックリンク運用と OpenCode パス修正

## 背景
現状の `install-skill` 実装では、canonical へのインストール後に各エージェント向けディレクトリへ再度インストールを行っており、シンボリックリンクではなくコピーが作成されます。また OpenCode のパスがスコープ（project/global）に応じて正しく扱われていません。

## 目的
- canonical（`.agents/skills/<skill>` または `~/.agents/skills/<skill>`）を単一の実体として扱い、エージェント向けディレクトリにはシンボリックリンクを作成する。
- OpenCode のインストール先をスコープに応じて正しく解決する。

## 対象範囲
- `install-skill` コマンドのインストールフロー
- エージェント別のターゲットディレクトリ解決

## 変更方針
- canonical へのインストールは 1 回のみ行い、`InstallConfig.target_dirs` にエージェント向けディレクトリを渡してリンク作成を行う。
- OpenCode の project スコープでは `.agents/skills` が universal であるため、追加の target dir を作らない。
- OpenCode の global スコープでは `~/.config/opencode/skills` を target dir として扱う。

## 影響とリスク
- 既存のコピー方式でインストール済みのディレクトリが上書きされる可能性があるため、事前に削除・置換の挙動を明確化する。
- シンボリックリンク作成失敗時は既存仕様どおり copy fallback とする。

## 非目的
- 新規エージェントの追加
- 既存ロックファイル形式の変更
