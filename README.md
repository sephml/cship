# CShip (pronounced "sea ship")

**Beautiful, Blazing-fast, Customizable Claude Code Statusline.**

`cship` is a custom statusline rendered by [Claude Code](https://claude.ai/code) that shows real-time session data — model name, cost, context window usage, usage limits, and more — directly in your terminal prompt.

---

## Install

### Option A — curl installer (recommended)

Downloads a pre-built binary for your platform (macOS arm64/x86_64, Linux x86_64/aarch64):

```sh
curl -fsSL https://raw.githubusercontent.com/stephenleo/cship/main/install.sh | bash
```

The installer:
- Places the binary at `~/.local/bin/cship`
- Creates a starter config at `~/.config/cship.toml`
- Wires the statusline into `~/.claude/settings.json`
- Optionally installs [Starship](https://starship.rs) (needed for passthrough modules)
- On Linux, optionally installs `libsecret-tools` (needed for usage limits)

### Option B — cargo install

```sh
cargo install cship
```

Then add `cship` as the Claude Code statusline in `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "cship"
  }
}
```

---

## Configuration

Create `~/.config/cship.toml` (or a `cship.toml` at your project root):

```toml
[cship]
lines = ["$cship.model $cship.cost $cship.context_bar"]
```

Each entry in `lines` is one row of the statusline. Module tokens follow `$cship.<module>` syntax. [Starship](https://starship.rs) modules (e.g. `$git_branch`) can be mixed in freely.

### Available modules

| Module | Token | Shows |
|--------|-------|-------|
| `model` | `$cship.model` | Claude model name |
| `cost` | `$cship.cost` | Session cost, duration, lines changed |
| `context_bar` | `$cship.context_bar` | Context window usage bar |
| `context_window` | `$cship.context_window` | Context window token count |
| `vim` | `$cship.vim` | Vim mode (Normal / Insert) |
| `agent` | `$cship.agent` | Sub-agent name |
| `session` | `$cship.session` | Session identity |
| `workspace` | `$cship.workspace` | Project directory |
| `usage_limits` | `$cship.usage_limits` | API usage limits (5 hr / 7 day) |

### Styling example

```toml
[cship]
lines = ["$cship.model $cship.cost $cship.context_bar"]

[cship.cost]
warn_threshold    = 1.0
warn_style        = "bold yellow"
critical_threshold = 5.0
critical_style    = "bold red"
```

### Debug your config

```sh
cship explain
```

Prints each module's live value, the config file in use, and any warnings — useful when something looks wrong.

---

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
