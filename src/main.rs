//! rmcp-xdotool: MCP server for mouse and keyboard automation via xdotool
//!
//! Gives Claude the power to interact with your desktop.
//! Use responsibly. Or don't. You're a pioneer.

use rmcp::{
    handler::server::{router::tool::ToolRouter, ServerHandler, wrapper::Parameters},
    model::*,
    ErrorData as McpError,
    ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::{env, process::{Command, Output}};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// === Parameter Types ===

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveMouseParams {
    #[schemars(description = "X coordinate")]
    pub x: i32,
    #[schemars(description = "Y coordinate")]
    pub y: i32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickParams {
    #[schemars(description = "Button to click: 1 (left), 2 (middle), 3 (right). Default: 1")]
    #[serde(default = "default_button")]
    pub button: u8,
}

fn default_button() -> u8 { 1 }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickAtParams {
    #[schemars(description = "X coordinate")]
    pub x: i32,
    #[schemars(description = "Y coordinate")]
    pub y: i32,
    #[schemars(description = "Button to click: 1 (left), 2 (middle), 3 (right). Default: 1")]
    #[serde(default = "default_button")]
    pub button: u8,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TypeTextParams {
    #[schemars(description = "Text to type")]
    pub text: String,
    #[schemars(description = "Delay between keystrokes in milliseconds. Default: 12")]
    #[serde(default = "default_delay")]
    pub delay: u32,
}

fn default_delay() -> u32 { 12 }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KeyPressParams {
    #[schemars(description = "Key(s) to press. Examples: Return, Escape, ctrl+c, alt+Tab, super+1")]
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScrollParams {
    #[schemars(description = "Scroll direction: up, down, left, right")]
    pub direction: String,
    #[schemars(description = "Number of clicks to scroll. Default: 3")]
    #[serde(default = "default_clicks")]
    pub clicks: u32,
}

fn default_clicks() -> u32 { 3 }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchWindowParams {
    #[schemars(description = "Search query (window name, class, or pattern)")]
    pub query: String,
    #[schemars(description = "Search by: 'name', 'class', 'classname', or 'any' (default: 'any')")]
    #[serde(default = "default_search_type")]
    pub search_type: String,
}

fn default_search_type() -> String { "any".to_string() }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WindowIdParams {
    #[schemars(description = "Window ID (from search_window or get_active_window)")]
    pub window_id: String,
}

// === Server ===

#[derive(Debug)]
pub struct XdotoolServer {
    pub tool_router: ToolRouter<Self>,
}

impl Default for XdotoolServer {
    fn default() -> Self {
        Self::new()
    }
}

impl XdotoolServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn button_name(button: u8) -> &'static str {
        match button {
            1 => "left",
            2 => "middle",
            3 => "right",
            _ => "unknown"
        }
    }

    fn x11_hint() -> String {
        let display = env::var("DISPLAY").unwrap_or_else(|_| "<unset>".into());
        let xauthority = env::var("XAUTHORITY").unwrap_or_else(|_| "<unset>".into());
        format!(
            "X11 env: DISPLAY={}, XAUTHORITY={}. rmcp-xdotool needs a live X11 session.",
            display, xauthority
        )
    }

    fn run_xdotool(args: &[&str]) -> Result<Output, McpError> {
        Command::new("xdotool").args(args).output().map_err(|e| {
            McpError::internal_error(format!("Failed to run xdotool: {}. {}", e, Self::x11_hint()), None)
        })
    }

    fn xdotool_error(output: &Output) -> McpError {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() { "unknown error" } else { &stderr };
        McpError::internal_error(format!("xdotool error: {}. {}", detail, Self::x11_hint()), None)
    }
}

#[rmcp::tool_router]
impl XdotoolServer {
    #[rmcp::tool(description = "Move mouse cursor to x,y coordinates on screen")]
    pub async fn move_mouse(
        &self,
        Parameters(params): Parameters<MoveMouseParams>,
    ) -> Result<CallToolResult, McpError> {
        let x = params.x.to_string();
        let y = params.y.to_string();
        let output = Self::run_xdotool(&["mousemove", &x, &y])?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                format!("Mouse moved to ({}, {})", params.x, params.y)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Click mouse button at current cursor position. Button: 1=left, 2=middle, 3=right")]
    pub async fn click(
        &self,
        Parameters(params): Parameters<ClickParams>,
    ) -> Result<CallToolResult, McpError> {
        let button = params.button.to_string();
        let output = Self::run_xdotool(&["click", &button])?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                format!("Clicked {} mouse button", Self::button_name(params.button))
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Move mouse to x,y coordinates and click. Button: 1=left, 2=middle, 3=right")]
    pub async fn click_at(
        &self,
        Parameters(params): Parameters<ClickAtParams>,
    ) -> Result<CallToolResult, McpError> {
        let x = params.x.to_string();
        let y = params.y.to_string();
        let button = params.button.to_string();
        let output = Self::run_xdotool(&["mousemove", &x, &y, "click", &button])?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                format!("Clicked {} at ({}, {})", Self::button_name(params.button), params.x, params.y)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Type text as keyboard input. Use for filling forms, search boxes, etc.")]
    pub async fn type_text(
        &self,
        Parameters(params): Parameters<TypeTextParams>,
    ) -> Result<CallToolResult, McpError> {
        let delay = params.delay.to_string();
        let output = Self::run_xdotool(&["type", "--delay", &delay, &params.text])?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                format!("Typed: \"{}\"", params.text)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Press a key or combo. Examples: Return, Escape, ctrl+c, alt+Tab, super+1, ctrl+shift+t")]
    pub async fn key_press(
        &self,
        Parameters(params): Parameters<KeyPressParams>,
    ) -> Result<CallToolResult, McpError> {
        let output = Self::run_xdotool(&["key", &params.key])?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                format!("Pressed key: {}", params.key)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Scroll mouse wheel. Direction: up, down, left, right")]
    pub async fn scroll(
        &self,
        Parameters(params): Parameters<ScrollParams>,
    ) -> Result<CallToolResult, McpError> {
        let button = match params.direction.to_lowercase().as_str() {
            "up" => "4",
            "down" => "5",
            "left" => "6",
            "right" => "7",
            _ => return Err(McpError::internal_error(
                "Invalid direction. Use: up, down, left, right",
                None
            ))
        };

        let clicks = params.clicks.to_string();
        let output = Self::run_xdotool(&["click", "--repeat", &clicks, button])?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                format!("Scrolled {} {} clicks", params.direction, params.clicks)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Get current mouse cursor position")]
    pub async fn get_mouse_position(&self) -> Result<CallToolResult, McpError> {
        let output = Self::run_xdotool(&["getmouselocation", "--shell"])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut x = 0;
            let mut y = 0;
            for line in stdout.lines() {
                if line.starts_with("X=") {
                    x = line[2..].parse().unwrap_or(0);
                } else if line.starts_with("Y=") {
                    y = line[2..].parse().unwrap_or(0);
                }
            }
            Ok(CallToolResult::success(vec![Content::text(
                format!("Mouse position: ({}, {})", x, y)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Double-click at current mouse position")]
    pub async fn double_click(&self) -> Result<CallToolResult, McpError> {
        let output = Self::run_xdotool(&["click", "--repeat", "2", "1"])?;

        if output.status.success() {
            Ok(CallToolResult::success(vec![Content::text(
                "Double-clicked".to_string()
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Show basic runtime diagnostics for xdotool and the current X11 environment")]
    pub async fn diagnostics(&self) -> Result<CallToolResult, McpError> {
        let xdotool_path = Command::new("sh")
            .args(["-c", "command -v xdotool || true"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "<not found>".into());
        let display = env::var("DISPLAY").unwrap_or_else(|_| "<unset>".into());
        let xauthority = env::var("XAUTHORITY").unwrap_or_else(|_| "<unset>".into());
        let probe = Self::run_xdotool(&["getmouselocation"]);
        let status = match probe {
            Ok(output) if output.status.success() => "ok".to_string(),
            Ok(output) => format!("error: {}", String::from_utf8_lossy(&output.stderr).trim()),
            Err(err) => err.to_string(),
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "xdotool: {}\nDISPLAY: {}\nXAUTHORITY: {}\nprobe: {}",
            xdotool_path, display, xauthority, status
        ))]))
    }

    #[rmcp::tool(description = "Search for windows by name, class, or pattern. Returns window IDs.")]
    pub async fn search_window(
        &self,
        Parameters(params): Parameters<SearchWindowParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut args = vec!["search"];

        match params.search_type.to_lowercase().as_str() {
            "name" => args.push("--name"),
            "class" => args.push("--class"),
            "classname" => args.push("--classname"),
            _ => {} // 'any' uses default behavior
        }

        args.push(&params.query);

        let output = Self::run_xdotool(&args)?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let window_ids: Vec<&str> = stdout.lines().collect();

            if window_ids.is_empty() {
                Ok(CallToolResult::success(vec![Content::text(
                    format!("No windows found matching '{}'", params.query)
                )]))
            } else {
                Ok(CallToolResult::success(vec![Content::text(
                    format!("Found {} window(s):\n{}", window_ids.len(), stdout.trim())
                )]))
            }
        } else {
            // xdotool search returns non-zero if no windows found
            Ok(CallToolResult::success(vec![Content::text(
                format!("No windows found matching '{}'", params.query)
            )]))
        }
    }

    #[rmcp::tool(description = "Get the currently focused/active window ID")]
    pub async fn get_active_window(&self) -> Result<CallToolResult, McpError> {
        let output = Self::run_xdotool(&["getactivewindow"])?;

        if output.status.success() {
            let window_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(CallToolResult::success(vec![Content::text(
                format!("Active window ID: {}", window_id)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Get window geometry (position and size) for a window ID")]
    pub async fn get_window_geometry(
        &self,
        Parameters(params): Parameters<WindowIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let output = Self::run_xdotool(&["getwindowgeometry", "--shell", &params.window_id])?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut x = 0;
            let mut y = 0;
            let mut width = 0;
            let mut height = 0;
            let mut screen = 0;

            for line in stdout.lines() {
                if line.starts_with("X=") {
                    x = line[2..].parse().unwrap_or(0);
                } else if line.starts_with("Y=") {
                    y = line[2..].parse().unwrap_or(0);
                } else if line.starts_with("WIDTH=") {
                    width = line[6..].parse().unwrap_or(0);
                } else if line.starts_with("HEIGHT=") {
                    height = line[7..].parse().unwrap_or(0);
                } else if line.starts_with("SCREEN=") {
                    screen = line[7..].parse().unwrap_or(0);
                }
            }

            Ok(CallToolResult::success(vec![Content::text(
                format!("Window {} geometry:\n  Position: ({}, {})\n  Size: {}x{}\n  Screen: {}",
                    params.window_id, x, y, width, height, screen)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }

    #[rmcp::tool(description = "Get the window title/name for a window ID")]
    pub async fn get_window_name(
        &self,
        Parameters(params): Parameters<WindowIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let output = Self::run_xdotool(&["getwindowname", &params.window_id])?;

        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(CallToolResult::success(vec![Content::text(
                format!("Window {} title: {}", params.window_id, name)
            )]))
        } else {
            Err(Self::xdotool_error(&output))
        }
    }
}

#[rmcp::tool_handler]
impl ServerHandler for XdotoolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Mouse and keyboard automation via xdotool. Move, click, type, scroll.".into()),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting rmcp-xdotool server");
    tracing::info!("{}", XdotoolServer::x11_hint());

    let server = XdotoolServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    tracing::info!("rmcp-xdotool server stopped");
    Ok(())
}
