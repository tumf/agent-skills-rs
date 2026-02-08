# 設計メモ

## 方針
- 読み込み時に旧形式と新形式の両方を受け付ける
- 旧形式は読み込み時に新形式へ変換する
- 変換後は既存の保存処理を通じて新形式が永続化される

## 旧形式の想定
```json
{
  "skills": [
    {"name": "skill-a", "path": "/path/to/a", "source_type": "github"}
  ]
}
```

## 変換ルール
- `skills` 配列を `HashMap<skillName, LockEntry>` に変換
- `path` が空の要素はインストール済みとみなせないため除外
- `source` と `sourceType` は `source_type` から設定
- `skillFolderHash` は空文字で初期化（後続更新で補完）
- `installedAt` / `updatedAt` は変換時刻で初期化
- `version` は `"1.0"` とする

## 互換性
- 新形式（version + skills map）は従来通りデシリアライズ可能
- 旧形式は読み込み時に自動的に新形式へ昇格
