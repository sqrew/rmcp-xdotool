# rmcp-xdotool

MCP server for mouse and keyboard automation via xdotool. Gives Claude (or any MCP client) the ability to control your desktop.

## Installation

```bash
cargo install rmcp-xdotool
```

Or build from source:
```bash
git clone https://github.com/sqrew/rmcp-xdotool
cd rmcp-xdotool
cargo build --release
```

## Requirements

- Linux with X11
- xdotool installed (`sudo pacman -S xdotool` or `sudo apt install xdotool`)

## Tools

| Tool | Description |
|------|-------------|
| `move_mouse` | Move cursor to x,y coordinates |
| `click` | Click at current position (1=left, 2=middle, 3=right) |
| `click_at` | Move to x,y and click |
| `type_text` | Type text as keyboard input |
| `key_press` | Press key/combo (e.g., `ctrl+c`, `alt+Tab`, `Return`) |
| `scroll` | Scroll up/down/left/right |
| `get_mouse_position` | Get current cursor position |
| `double_click` | Double-click at current position |

## Claude Code Configuration

Add to your `~/.claude.json`:

```json
{
  "mcpServers": {
    "xdotool": {
      "type": "stdio",
      "command": "/path/to/rmcp-xdotool",
      "args": [],
      "env": {}
    }
  }
}
```

## Usage Examples

```
Claude, move my mouse to (500, 300)
Claude, click the button at position (800, 450)
Claude, type "hello world" into the search box
Claude, press ctrl+s to save
Claude, scroll down 5 clicks
```

## Warning

This gives Claude full control of your mouse and keyboard. Use responsibly. Or don't. You're a pioneer.

## Related Projects

- [claude-sensors](https://crates.io/crates/claude-sensors) - Environmental awareness for AI
- [rmcp-i3](https://crates.io/crates/rmcp-i3) - i3 window manager control
- [rmcp-breakrs](https://crates.io/crates/rmcp-breakrs) - Desktop break reminders

## License

MIT

---

Built with Claude. For Claude. By sqrew.
