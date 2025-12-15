# Data Model: Copy Capture to Clipboard on Save

**Branch**: `001-copy-capture-clipboard` | **Date**: 2025-12-15

## Entities

### ClipboardSetting

A boolean configuration option controlling clipboard behavior on save.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `clipboardCopyEnabled` | `bool` | `true` | Whether to copy screenshots to clipboard on save |

**Storage**: `settings.json` via tauri-plugin-store
**Persistence**: Automatic via store plugin

### SaveData (Extended)

Request payload for save operation - extends existing structure.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `filename` | `String` | Yes | Target filename without extension |
| `timestampOptions` | `TimestampOptions` | Yes | Timestamp overlay configuration |
| `imageFormat` | `String` | Yes | Output format ("jpg" or "png") |
| `clipboardCopyEnabled` | `bool` | Yes | Whether to copy to clipboard |

### ClipboardCopyResult

Result status returned from save operation.

| Field | Type | Description |
|-------|------|-------------|
| `fileSaved` | `bool` | Whether file was saved successfully |
| `clipboardCopied` | `bool` | Whether clipboard copy succeeded |
| `clipboardError` | `Option<String>` | Error message if clipboard failed |

### ClipboardErrorPayload

Event payload for clipboard failure notification.

| Field | Type | Description |
|-------|------|-------------|
| `message` | `String` | User-friendly error message |
| `reason` | `String` | Technical error reason |

## State Transitions

### Save Operation Flow

```
SaveRequested
    │
    ├─► FileSaveAttempted
    │       │
    │       ├─► FileSaveSuccess
    │       │       │
    │       │       └─► ClipboardCopyAttempted (if enabled)
    │       │               │
    │       │               ├─► ClipboardCopySuccess ─► Complete
    │       │               │
    │       │               └─► ClipboardCopyFailed
    │       │                       │
    │       │                       └─► EmitErrorEvent ─► Complete
    │       │
    │       └─► FileSaveFailed ─► Error (no clipboard attempt)
```

## Validation Rules

- `clipboardCopyEnabled` must be serializable as JSON boolean
- Setting default `true` applied when key missing from store
- Clipboard copy only attempted when `clipboardCopyEnabled == true` AND file save succeeded

## Relationships

```
AllSettings
    └─► clipboardCopyEnabled (new field)

SaveData
    └─► clipboardCopyEnabled (passed from frontend or settings)

save_screenshot()
    ├─► reads clipboardCopyEnabled
    ├─► writes file (existing)
    └─► conditionally copies to clipboard (new)
```
