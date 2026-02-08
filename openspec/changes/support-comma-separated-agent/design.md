# 設計

## 方針
- CLI の `--agent` は **カンマ区切り** と **複数回指定** を両方許可する。
- 受け取った agent は `split(',')` + `trim()` で正規化し、空要素を除外する。
- 重複は順序保持で除外する（最初に出現した順を維持）。
- 未知の agent 名は fail-fast でエラーにする。

## 仕様マッピング
- 解析済み agent の一覧から `target_dirs` を解決する。
- canonical ディレクトリ（`.agents/skills/<skill>`）は単一の真実の源として維持する。
- `target_dirs` には canonical と同一の `.agents/skills` を含めない。

## 例
- 入力: `--agent claude,opencode`
  - 解決: `["claude", "opencode"]`
  - 配布先: `.claude/skills`, `.config/opencode/skills`
- 入力: `--agent claude --agent opencode,claude`
  - 解決: `["claude", "opencode"]`（重複除外）

## トレードオフ
- JSON Schema で「複数回指定」を厳密に表現しづらい点があるため、説明文で明示する。
