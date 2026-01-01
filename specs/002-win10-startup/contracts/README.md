# API Contracts: Windows 10 Startup Compatibility

**Feature**: 002-win10-startup

## Overview

This feature does not expose external APIs. All functionality is internal to the application.

## Internal Tauri Commands

The following Tauri IPC commands are modified by this feature:

### get_auto_start

**Direction**: Frontend → Backend
**Request**: None
**Response**: `Result<bool, String>`

```typescript
// Frontend usage
const isEnabled: boolean = await invoke('get_auto_start');
```

### set_auto_start

**Direction**: Frontend → Backend
**Request**: `{ enabled: boolean }`
**Response**: `Result<bool, String>`

```typescript
// Frontend usage
await invoke('set_auto_start', { enabled: true });
```

## No External Contracts

This feature:
- Does not expose REST/GraphQL APIs
- Does not define new data formats
- Does not integrate with external services
- Only modifies internal Tauri IPC commands

The existing frontend code continues to work without changes.
