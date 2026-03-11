# CShip (pronounced "sea ship")

**Beautiful, Blazing-fast, Customizable Claude Code Statusline.**

`cship` renders a live statusline for [Claude Code](https://claude.ai/code) sessions, showing session cost, context window usage, model name, API usage limits, and more — all configurable via a simple TOML file.

## Install

### Method 1: curl installer (recommended)

Auto-detects your OS and architecture (macOS arm64/x86_64, Linux x86_64/aarch64), downloads the binary to `~/.local/bin/cship`, creates a starter config at `~/.config/cship.toml`, wires the `statusLine` entry in `~/.claude/settings.json`, and optionally installs [Starship](https://starship.rs) (needed for passthrough modules) and, on Linux, `libsecret-tools` (needed for usage limits).

```sh
curl -fsSL https://raw.githubusercontent.com/stephenleo/cship/main/install.sh | bash
```

### Method 2: cargo install

Requires the Rust toolchain.

```sh
cargo install cship
```

After installing with `cargo`, wire the statusline manually in `~/.claude/settings.json`:

```json
{
  "statusLine": { "type": "command", "command": "cship" }
}
```

## Configuration

The default config file is `~/.config/cship.toml`. You can also place a `cship.toml` in your project root for per-project overrides. A minimal working example:

```toml
[cship]
lines = ["$cship.model $cship.cost $cship.context_bar"]
```

The `lines` array defines the rows of your statusline. Each element is a format string mixing `$cship.<module>` tokens (native cship modules) with Starship module tokens (e.g. `$git_branch`).

### Styling example

```toml
[cship]
lines = ["$cship.model $cship.cost $cship.context_bar"]

[cship.cost]
warn_threshold = 1.0
warn_style = "bold yellow"
critical_threshold = 5.0
critical_style = "bold red"
```

### Available modules

| Token | Description |
|-------|-------------|
| `$cship.model` | Claude model name |
| `$cship.cost` | Session cost in USD ($X.XX) |
| `$cship.context_bar` | Visual progress bar of context window usage |
| `$cship.context_window` | Context window tokens (used/total) |
| `$cship.usage_limits` | API usage limits (5hr / 7-day) |
| `$cship.vim` | Vim mode indicator |
| `$cship.agent` | Sub-agent name |
| `$cship.session` | Session identity info |
| `$cship.workspace` | Workspace/project directory |

Full configuration reference: **https://cship.dev**

## Debugging

Run `cship explain` to inspect what cship sees from Claude Code's context JSON — useful when a module shows nothing or behaves unexpectedly.

```sh
cship explain
```

## Showcase

Six ready-to-use configurations — from minimal to full-featured. Each can be dropped into `~/.config/cship.toml`.

---

### 1. Minimal

One clean row. Model, cost with colour thresholds, context bar.

<!-- image -->

<details>
<summary>View config</summary>

```toml
[cship]
lines = ["$cship.model  $cship.cost  $cship.context_bar"]

[cship.cost]
style              = "green"
warn_threshold     = 2.0
warn_style         = "yellow"
critical_threshold = 5.0
critical_style     = "bold red"

[cship.context_bar]
width              = 10
warn_threshold     = 75.0
warn_style         = "yellow"
critical_threshold = 90.0
critical_style     = "bold red"
```

</details>

---

### 2. Git-Aware Developer

Two rows: Starship git status on top, Claude session below. Starship passthrough (`$directory`, `$git_branch`, `$git_status`) requires [Starship](https://starship.rs) to be installed.

<!-- image -->

<details>
<summary>View config</summary>

```toml
[cship]
lines = [
  "$directory  $git_branch  $git_status",
  "$cship.model  $cship.cost  $cship.context_bar",
]

[cship.model]
symbol = "◆ "
style  = "bold cyan"

[cship.cost]
warn_threshold     = 2.0
warn_style         = "yellow"
critical_threshold = 8.0
critical_style     = "bold red"

[cship.context_bar]
width              = 12
warn_threshold     = 70.0
warn_style         = "yellow"
critical_threshold = 85.0
critical_style     = "bold red"
```

</details>

---

### 3. Cost Guardian

Shows cost, lines changed, and rolling API usage limits all at once. Colour escalates as budgets fill.

<!-- image -->

<details>
<summary>View config</summary>

```toml
[cship]
lines = [
  "$cship.model  $cship.cost  +$cship.cost.total_lines_added -$cship.cost.total_lines_removed",
  "$cship.context_bar  $cship.usage_limits",
]

[cship.model]
style = "bold purple"

[cship.cost]
symbol             = "$ "
warn_threshold     = 1.0
warn_style         = "bold yellow"
critical_threshold = 3.0
critical_style     = "bold red"

[cship.context_bar]
width              = 14
warn_threshold     = 60.0
warn_style         = "yellow"
critical_threshold = 80.0
critical_style     = "bold red"

[cship.usage_limits]
five_hour_format   = "5h {pct}%"
seven_day_format   = "7d {pct}%"
separator          = "  "
warn_threshold     = 70.0
warn_style         = "bold yellow"
critical_threshold = 90.0
critical_style     = "bold red"
```

</details>

---

### 4. Material Hex

Every style value is a `fg:#rrggbb` hex colour — no named colours anywhere. Amber warns, coral criticals.

<!-- image -->

<details>
<summary>View config</summary>

```toml
[cship]
lines = [
  "$cship.model  $cship.cost  $cship.context_bar",
  "$cship.usage_limits",
]

[cship.model]
style = "fg:#c3e88d"

[cship.cost]
style              = "fg:#82aaff"
warn_threshold     = 2.0
warn_style         = "fg:#ffcb6b"
critical_threshold = 6.0
critical_style     = "bold fg:#f07178"

[cship.context_bar]
width              = 14
style              = "fg:#89ddff"
warn_threshold     = 65.0
warn_style         = "fg:#ffcb6b"
critical_threshold = 85.0
critical_style     = "bold fg:#f07178"

[cship.usage_limits]
five_hour_format   = "5h {pct}%"
seven_day_format   = "7d {pct}%"
separator          = "  "
warn_threshold     = 70.0
warn_style         = "fg:#ffcb6b"
critical_threshold = 90.0
critical_style     = "bold fg:#f07178"
```

</details>

---

### 5. Tokyo Night

Three-row layout for polyglot developers. Starship handles language runtimes and git; cship handles session data. Styled with the [Tokyo Night](https://github.com/folke/tokyonight.nvim) colour palette.

<!-- image -->

<details>
<summary>View config</summary>

```toml
[cship]
lines = [
  "$directory  $git_branch  $git_status  $python  $nodejs  $rust",
  "$cship.model  $cship.agent",
  "$cship.context_bar  $cship.cost  $cship.usage_limits",
]

[cship.model]
symbol = "◈ "
style  = "bold fg:#7aa2f7"

[cship.agent]
symbol = "↳ "
style  = "fg:#9ece6a"

[cship.context_bar]
width              = 16
style              = "fg:#7dcfff"
warn_threshold     = 60.0
warn_style         = "fg:#e0af68"
critical_threshold = 80.0
critical_style     = "bold fg:#f7768e"

[cship.cost]
symbol             = "$ "
style              = "fg:#a9b1d6"
warn_threshold     = 2.0
warn_style         = "fg:#e0af68"
critical_threshold = 8.0
critical_style     = "bold fg:#f7768e"

[cship.usage_limits]
five_hour_format   = "5h {pct}%"
seven_day_format   = "7d {pct}%"
separator          = " · "
warn_threshold     = 70.0
warn_style         = "fg:#e0af68"
critical_threshold = 90.0
critical_style     = "bold fg:#f7768e"
```

</details>

---

### 6. Nerd Fonts

Requires a [Nerd Font](https://www.nerdfonts.com) in your terminal. Icons are embedded as `symbol` values on each module and as literal characters in the format string for Starship passthrough rows.

<!-- image -->

<details>
<summary>View config</summary>

```toml
[cship]
lines = [
  " $directory   $git_branch  $git_status",
  "$cship.model  $cship.cost  $cship.context_bar  $cship.usage_limits",
]

[cship.model]
symbol = " "
style  = "bold cyan"

[cship.cost]
symbol             = " "
style              = "green"
warn_threshold     = 2.0
warn_style         = "yellow"
critical_threshold = 6.0
critical_style     = "bold red"

[cship.context_bar]
symbol             = " "
width              = 12
warn_threshold     = 70.0
warn_style         = "yellow"
critical_threshold = 88.0
critical_style     = "bold red"

[cship.usage_limits]
five_hour_format   = "󱐋  5h {pct}%"
seven_day_format   = "7d {pct}%"
separator          = "  "
warn_threshold     = 70.0
warn_style         = "yellow"
critical_threshold = 90.0
critical_style     = "bold red"
```

</details>

---

## Full documentation

→ **[cship.dev](https://cship.dev)**

Complete configuration reference, format string syntax, all module options, and examples.

---

## License

Apache-2.0
