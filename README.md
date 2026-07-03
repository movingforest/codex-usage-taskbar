# Codex Usage Taskbar

Windows taskbar widget for OpenAI Codex usage limits.

This project is a Codex-focused fork of
[CodeZeno/Claude-Code-Usage-Monitor](https://github.com/CodeZeno/Claude-Code-Usage-Monitor).
The original project is MIT licensed; its license is kept in this repository.

## What It Shows

- `5h` Codex remaining percentage
- `7d` Codex remaining percentage
- local reset time for the 5-hour window
- tray tooltip with remaining percentage, used percentage, reset time, data source, and stale status

The compact clock indicators show used percentage in 10% steps. Codex uses blue by default, Claude Code uses orange by default, and `0%` is empty.

## Data Sources

The app reads Codex usage in this order:

1. Remote usage endpoint: `https://chatgpt.com/backend-api/wham/usage`
2. Local fallback: latest `token_count.rate_limits` payload under `%USERPROFILE%\.codex\sessions`

For the remote endpoint, the app reads `%USERPROFILE%\.codex\auth.json` or `%CODEX_HOME%\auth.json` to get the same access token used by the Codex CLI. Tokens are only used for the request and are not logged.

The local fallback reads only session JSONL records and extracts rate-limit metadata. It does not copy prompts, messages, or tool output into this repository.

## Build From Source

Requirements:

- Windows 10 or Windows 11
- Rust toolchain
- Visual Studio C++ Build Tools
- Codex CLI signed in with ChatGPT

Build:

```powershell
cargo test
cargo build --release
```

Run:

```powershell
.\target\release\codex-usage-taskbar.exe
```

Diagnostics:

```powershell
.\target\release\codex-usage-taskbar.exe --diagnose
```

Diagnostic log:

```text
%TEMP%\codex-usage-taskbar.log
```

Settings:

```text
%APPDATA%\CodexUsageTaskbar\settings.json
```

## UI Settings

Right-click the widget to open the menu.

- `Settings > Language` includes Simplified Chinese.
- `Settings > Colors` lets you choose the fill color, clock base color, and text color with the Windows color picker.
- `Settings > Screen` lets you choose which taskbar/monitor hosts the widget.

## Taskbar Placement

The primary Windows taskbar is screen 1 and is used by default. The default `tray_offset` is intentionally large, so on startup the app clamps itself to the leftmost available area before the Windows tray icons. Drag the left divider if you want to reposition it, or use `Settings > Screen` to move it to another monitor.

## GitHub Publishing

Recommended first publish flow:

```powershell
git status
git add .
git commit -m "Initial Codex usage taskbar fork"
gh auth login
gh repo create codex-usage-taskbar --public --source . --remote origin --push
```

Before publishing, verify:

```powershell
rg -n "access_token|auth.json|refresh_token|session_id|\.codex|BEGIN" .
git status --short
```

Do not commit `%USERPROFILE%\.codex`, local logs, `.env` files, build output, or generated session data.

## Upstream Attribution

Based on `CodeZeno/Claude-Code-Usage-Monitor` v1.4.8 by Code Zeno Pty Ltd.

Changes in this fork:

- Codex is enabled by default
- Claude Code and Antigravity are disabled by default
- taskbar text displays remaining Codex usage
- the 5-hour row shows local reset time
- progress bars were replaced with compact clock indicators
- Simplified Chinese localization, custom colors, and screen selection were added
- local Codex session JSONL fallback is available
- settings, startup key, diagnostics, and binary metadata use `Codex Usage Taskbar`
