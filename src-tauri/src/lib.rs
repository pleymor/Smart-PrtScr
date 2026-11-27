use ab_glyph::{Font, FontArc, PxScale};
use chrono::Local;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Mutex;
use std::env;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, Manager, Runtime, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;
use arboard::{Clipboard, ImageData};

mod autostart;
mod desktop;

use desktop::{MonitorCapture, MonitorRegion, Rect};


#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

// Global hotkey handler for Windows (Win+Shift+PrintScreen works in fullscreen games)
#[cfg(target_os = "windows")]
mod keyboard_hook {
    use super::*;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, GetMessageW, RegisterClassW,
        TranslateMessage, DispatchMessageW, DestroyWindow, PostMessageW,
        WNDCLASSW, MSG, WM_HOTKEY, WM_QUIT, WINDOW_EX_STYLE, WINDOW_STYLE,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, MOD_WIN, MOD_SHIFT, MOD_NOREPEAT,
    };
    use windows::core::PCWSTR;
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicIsize, Ordering};

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static HOOK_HWND: AtomicIsize = AtomicIsize::new(0);

    const VK_SNAPSHOT: u32 = 0x2C;
    const HOTKEY_WIN_SHIFT_PRTSCR: i32 = 1;
    const HOTKEY_PRTSCR: i32 = 2;

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if msg == WM_HOTKEY {
            let hotkey_id = wparam.0 as i32;
            if hotkey_id == HOTKEY_WIN_SHIFT_PRTSCR || hotkey_id == HOTKEY_PRTSCR {
                println!("[HOTKEY] Capture hotkey pressed (id={})", hotkey_id);
                if let Some(app) = APP_HANDLE.get() {
                    let _ = app.emit("trigger-capture", ());
                }
            }
            return LRESULT(0);
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    pub fn start_hook(app: AppHandle) {
        let _ = APP_HANDLE.set(app);

        std::thread::spawn(|| {
            unsafe {
                let class_name: Vec<u16> = "SmartPrtScrHotkey\0".encode_utf16().collect();
                let wc = WNDCLASSW {
                    lpfnWndProc: Some(wnd_proc),
                    lpszClassName: PCWSTR(class_name.as_ptr()),
                    ..Default::default()
                };
                RegisterClassW(&wc);

                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    PCWSTR(class_name.as_ptr()),
                    PCWSTR::null(),
                    WINDOW_STYLE::default(),
                    0, 0, 0, 0,
                    HWND(-3isize as *mut std::ffi::c_void), // HWND_MESSAGE
                    None,
                    None,
                    None,
                ).unwrap();

                // Store HWND for cleanup
                HOOK_HWND.store(hwnd.0 as isize, Ordering::SeqCst);

                // Win+Shift+PrintScreen (works in fullscreen games)
                if RegisterHotKey(hwnd, HOTKEY_WIN_SHIFT_PRTSCR, MOD_WIN | MOD_SHIFT | MOD_NOREPEAT, VK_SNAPSHOT).is_ok() {
                    println!("[HOTKEY] Win+Shift+PrintScreen registered");
                }

                // PrintScreen alone (for normal desktop use)
                if RegisterHotKey(hwnd, HOTKEY_PRTSCR, MOD_NOREPEAT, VK_SNAPSHOT).is_ok() {
                    println!("[HOTKEY] PrintScreen registered");
                }

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                let _ = UnregisterHotKey(hwnd, HOTKEY_WIN_SHIFT_PRTSCR);
                let _ = UnregisterHotKey(hwnd, HOTKEY_PRTSCR);
                let _ = DestroyWindow(hwnd);
            }
        });
    }

    pub fn stop_hook() {
        let hwnd_val = HOOK_HWND.load(Ordering::SeqCst);
        if hwnd_val != 0 {
            unsafe {
                let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
                let _ = PostMessageW(hwnd, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
    }
}

/// What a click on the tray icon does (the options stay reachable from the
/// right-click menu, "Options").
#[derive(Debug, PartialEq, Eq)]
enum TrayClickAction {
    StartCapture,
}

fn tray_click_action(button: MouseButton, state: MouseButtonState) -> Option<TrayClickAction> {
    match (button, state) {
        (MouseButton::Left, MouseButtonState::Up) => Some(TrayClickAction::StartCapture),
        _ => None,
    }
}

// State structures
#[derive(Clone)]
pub struct RawScreenshot {
    pub data: Vec<u8>,  // RGBA raw data
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct AppState {
    pub current_screenshot: Mutex<Option<RawScreenshot>>,
    pub current_screenshot_path: Mutex<Option<String>>, // Temp file path for quick loading via asset protocol
    pub pending_screenshot: Mutex<Option<PendingScreenshot>>,
    /// Last zone selected in the selection window (None until the user selects one).
    pub selection_bounds: Mutex<Option<SelectionBounds>>,
    /// Virtual-desktop rectangle covered by the current capture and by the
    /// selection window (physical pixels, may have negative coordinates).
    pub desktop_bounds: Mutex<Option<Rect>>,
    /// Monitors inside the current capture (image pixels).
    pub monitors: Mutex<Vec<MonitorRegion>>,
}

#[derive(Clone)]
pub struct PendingScreenshot {
    pub image_data: Vec<u8>,
    pub default_filename: String,
}

// Serializable structures
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TimestampOptions {
    pub enabled: bool,
    #[serde(rename = "fontSize")]
    pub font_size: u32,
    #[serde(rename = "type")]
    pub display_type: String,
    #[serde(rename = "textColor")]
    pub text_color: String,
    #[serde(rename = "textAlign")]
    pub text_align: String,
    pub position: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TimestampOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            font_size: 14,
            display_type: "banner-dark".to_string(),
            text_color: "white".to_string(),
            text_align: "center".to_string(),
            position: "bottom".to_string(),
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

/// Zone selected in the selection window, in its CSS pixels, with the size of
/// its viewport (needed to convert to image pixels whatever the DPI scale).
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SelectionBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(rename = "viewportWidth")]
    pub viewport_width: f64,
    #[serde(rename = "viewportHeight")]
    pub viewport_height: f64,
}

impl SelectionBounds {
    /// Crop rectangle in the captured image's pixels.
    fn to_image_rect(&self, image_width: u32, image_height: u32) -> Option<(u32, u32, u32, u32)> {
        desktop::selection_to_image_rect(
            (self.x, self.y, self.width, self.height),
            (self.viewport_width, self.viewport_height),
            (image_width, image_height),
        )
    }
}

/// Returned to the selection window: the screenshot to display and where each
/// monitor is inside it.
#[derive(Clone, Serialize, Debug)]
pub struct CaptureInfo {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub monitors: Vec<MonitorRegion>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub filename: String,
    #[serde(rename = "timestampOptions")]
    pub timestamp_options: TimestampOptions,
    #[serde(rename = "imageFormat")]
    pub image_format: String,
}

// Event payload for clipboard copy failure notification
#[derive(Clone, Serialize)]
pub struct ClipboardErrorPayload {
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AllSettings {
    #[serde(rename = "savePath")]
    pub save_path: String,
    #[serde(rename = "autoStart")]
    pub auto_start: bool,
    #[serde(rename = "imageFormat")]
    pub image_format: String,
    #[serde(rename = "windowsPrtScrDisabled")]
    pub windows_prtscr_disabled: bool,
    #[serde(rename = "timestampOptions")]
    pub timestamp_options: TimestampOptions,
    #[serde(rename = "clipboardCopyEnabled")]
    pub clipboard_copy_enabled: bool,
}

// Get default screenshot path
fn get_default_screenshot_path() -> PathBuf {
    dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Screenshots")
}

// Generate default filename
fn generate_default_filename() -> String {
    Local::now().format("%Y-%m-%dT%H-%M-%S_capture").to_string()
}

// Get text color from name
fn get_text_color(color_name: &str) -> Rgba<u8> {
    match color_name {
        "white" => Rgba([255, 255, 255, 255]),
        "black" => Rgba([0, 0, 0, 255]),
        "gray" => Rgba([128, 128, 128, 255]),
        "red" => Rgba([255, 0, 0, 255]),
        "green" => Rgba([0, 255, 0, 255]),
        "blue" => Rgba([0, 0, 255, 255]),
        "yellow" => Rgba([255, 255, 0, 255]),
        "cyan" => Rgba([0, 255, 255, 255]),
        "magenta" => Rgba([255, 0, 255, 255]),
        _ => Rgba([255, 255, 255, 255]),
    }
}

// Get banner color
fn get_banner_color(display_type: &str) -> Rgba<u8> {
    if display_type == "banner-light" {
        Rgba([245, 245, 245, 255])
    } else {
        Rgba([51, 51, 51, 255])
    }
}

// Result of timestamp processing: encoded bytes for file + RGBA for clipboard
struct ProcessedImage {
    encoded_bytes: Vec<u8>,
    rgba_pixels: Vec<u8>,
    width: u32,
    height: u32,
}

// Add timestamp to image - returns both encoded format and raw RGBA
fn add_timestamp_to_image(
    image_data: &[u8],
    options: &TimestampOptions,
    format: &str,
) -> Result<ProcessedImage, String> {
    let img = image::load_from_memory(image_data).map_err(|e| e.to_string())?;
    let mut rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();

    if !options.enabled {
        let rgba_pixels = rgba_img.as_raw().clone();
        let encoded_bytes = encode_image(&DynamicImage::ImageRgba8(rgba_img), format)?;
        return Ok(ProcessedImage { encoded_bytes, rgba_pixels, width, height });
    }

    let timestamp = Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
    let banner_height = (options.font_size + 20).max(30);
    let text_color = get_text_color(&options.text_color);

    // Load embedded font using ab_glyph
    let font_data = include_bytes!("../fonts/arial.ttf");
    let font = FontArc::try_from_slice(font_data).map_err(|e| e.to_string())?;
    let scale = PxScale::from(options.font_size as f32);

    let is_overlay = options.display_type == "overlay";

    if is_overlay {
        // Overlay mode: draw text directly on image
        let text_y = if options.position == "top" {
            10
        } else {
            (height as i32) - (options.font_size as i32) - 10
        };

        let text_width = get_text_width(&font, scale, &timestamp);
        let text_x = match options.text_align.as_str() {
            "left" => 10,
            "right" => width as i32 - text_width - 10,
            _ => (width as i32 - text_width) / 2,
        };

        draw_text_mut(&mut rgba_img, text_color, text_x, text_y, scale, &font, &timestamp);
        let rgba_pixels = rgba_img.as_raw().clone();
        let encoded_bytes = encode_image(&DynamicImage::ImageRgba8(rgba_img), format)?;
        Ok(ProcessedImage { encoded_bytes, rgba_pixels, width, height })
    } else {
        // Banner mode: add banner above or below image
        let new_height = height + banner_height;
        let mut new_img = RgbaImage::new(width, new_height);

        let banner_color = get_banner_color(&options.display_type);
        let (img_y, banner_y) = if options.position == "top" {
            (banner_height, 0)
        } else {
            (0, height)
        };

        // Draw banner background
        for y in banner_y..(banner_y + banner_height) {
            for x in 0..width {
                new_img.put_pixel(x, y, banner_color);
            }
        }

        // Copy original image
        for y in 0..height {
            for x in 0..width {
                new_img.put_pixel(x, y + img_y, *rgba_img.get_pixel(x, y));
            }
        }

        // Calculate text position
        let text_width = get_text_width(&font, scale, &timestamp);
        let padding = 20;
        let text_x = match options.text_align.as_str() {
            "left" => padding,
            "right" => width as i32 - text_width - padding,
            _ => (width as i32 - text_width) / 2,
        };
        let text_y = banner_y as i32 + (banner_height as i32 - options.font_size as i32) / 2;

        draw_text_mut(&mut new_img, text_color, text_x, text_y, scale, &font, &timestamp);
        let rgba_pixels = new_img.as_raw().clone();
        let encoded_bytes = encode_image(&DynamicImage::ImageRgba8(new_img), format)?;
        Ok(ProcessedImage { encoded_bytes, rgba_pixels, width: width, height: new_height })
    }
}

fn get_text_width(font: &FontArc, scale: PxScale, text: &str) -> i32 {
    let mut width = 0.0;
    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let h_advance = font.h_advance_unscaled(glyph_id);
        width += h_advance * scale.x / font.height_unscaled();
    }
    width as i32
}

fn encode_image(img: &DynamicImage, format: &str) -> Result<Vec<u8>, String> {
    let mut buffer = Cursor::new(Vec::new());
    let img_format = if format == "png" {
        ImageFormat::Png
    } else {
        ImageFormat::Jpeg
    };
    img.write_to(&mut buffer, img_format).map_err(|e| e.to_string())?;
    Ok(buffer.into_inner())
}

// Copy image to system clipboard from raw RGBA data
fn copy_rgba_to_clipboard(pixels: Vec<u8>, width: u32, height: u32) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let img_data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(pixels),
    };
    clipboard.set_image(img_data).map_err(|e| e.to_string())?;
    Ok(())
}

// Tauri commands
#[tauri::command]
async fn get_all_settings(app: AppHandle) -> Result<AllSettings, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let default_path = get_default_screenshot_path();

    // Save path
    let save_path = store.get("screenshotPath")
        .and_then(|p| p.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| default_path.to_string_lossy().to_string());

    // Auto start
    let auto_start = autostart::is_enabled().unwrap_or(false);

    // Image format
    let image_format = store.get("imageFormat")
        .and_then(|f| f.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "jpg".to_string());

    // Windows PrtScr disabled
    #[cfg(target_os = "windows")]
    let windows_prtscr_disabled = {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        hkcu.open_subkey("Control Panel\\Keyboard")
            .and_then(|key| key.get_value::<u32, _>("PrintScreenKeyForSnippingEnabled"))
            .map(|v| v == 0)
            .unwrap_or(false)
    };
    #[cfg(not(target_os = "windows"))]
    let windows_prtscr_disabled = false;

    // Timestamp options
    let timestamp_options = store.get("timestampOptions")
        .and_then(|o| serde_json::from_value(o.clone()).ok())
        .unwrap_or_default();

    // Clipboard copy enabled (default: true)
    let clipboard_copy_enabled = store.get("clipboardCopyEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(AllSettings {
        save_path,
        auto_start,
        image_format,
        windows_prtscr_disabled,
        timestamp_options,
        clipboard_copy_enabled,
    })
}

#[tauri::command]
async fn get_save_path(app: AppHandle) -> Result<String, String> {
    println!("[LOG] {} get_save_path called", Local::now().format("%H:%M:%S%.3f"));
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let default_path = get_default_screenshot_path();

    if let Some(path) = store.get("screenshotPath") {
        if let Some(path_str) = path.as_str() {
            return Ok(path_str.to_string());
        }
    }

    Ok(default_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn set_save_path(app: AppHandle) -> Result<Option<String>, String> {
    // Get the appropriate parent window (filename-dialog during capture, main otherwise)
    let parent_window = app.get_webview_window("filename-dialog")
        .or_else(|| app.get_webview_window("main"));

    // Open folder picker dialog (blocking)
    let mut builder = app.dialog()
        .file()
        .set_title("Choisir le dossier de destination");

    // Set parent window if available
    if let Some(ref window) = parent_window {
        builder = builder.set_parent(window);
    }

    let folder = builder.blocking_pick_folder();

    if let Some(path) = folder {
        let path_str = path.to_string();
        let store = app.store("settings.json").map_err(|e| e.to_string())?;
        store.set("screenshotPath", serde_json::json!(path_str));
        store.save().map_err(|e| e.to_string())?;
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn reset_save_path(app: AppHandle) -> Result<String, String> {
    let default_path = get_default_screenshot_path();
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let path_str = default_path.to_string_lossy().to_string();
    store.set("screenshotPath", serde_json::json!(path_str));
    store.save().map_err(|e| e.to_string())?;
    Ok(path_str)
}

#[tauri::command]
async fn get_timestamp_options(app: AppHandle) -> Result<TimestampOptions, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    if let Some(options) = store.get("timestampOptions") {
        if let Ok(opts) = serde_json::from_value(options.clone()) {
            return Ok(opts);
        }
    }

    Ok(TimestampOptions::default())
}

#[tauri::command]
async fn set_timestamp_options(app: AppHandle, options: TimestampOptions) -> Result<TimestampOptions, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("timestampOptions", serde_json::to_value(&options).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(options)
}

#[tauri::command]
async fn reset_timestamp_options(app: AppHandle) -> Result<TimestampOptions, String> {
    let defaults = TimestampOptions::default();
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("timestampOptions", serde_json::to_value(&defaults).unwrap());
    store.save().map_err(|e| e.to_string())?;
    Ok(defaults)
}

#[tauri::command]
async fn get_image_format(app: AppHandle) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    if let Some(format) = store.get("imageFormat") {
        if let Some(fmt) = format.as_str() {
            return Ok(fmt.to_string());
        }
    }

    Ok("jpg".to_string())
}

#[tauri::command]
async fn set_image_format(app: AppHandle, format: String) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("imageFormat", serde_json::json!(format));
    store.save().map_err(|e| e.to_string())?;
    Ok(format)
}

#[tauri::command]
async fn get_clipboard_copy_enabled(app: AppHandle) -> Result<bool, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    Ok(store.get("clipboardCopyEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)) // Default: enabled (FR-003)
}

#[tauri::command]
async fn set_clipboard_copy_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("clipboardCopyEnabled", serde_json::json!(enabled));
    store.save().map_err(|e| e.to_string())?;
    Ok(enabled)
}

#[tauri::command]
async fn get_auto_start() -> Result<bool, String> {
    autostart::is_enabled()
}

#[tauri::command]
async fn set_auto_start(enabled: bool) -> Result<bool, String> {
    // Returns the effective state, so the checkbox reflects what Windows will do.
    autostart::set_enabled(enabled)
}

#[cfg(target_os = "windows")]
#[tauri::command]
async fn get_windows_prtscr_disabled() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("Control Panel\\Keyboard") {
        Ok(key) => {
            match key.get_value::<u32, _>("PrintScreenKeyForSnippingEnabled") {
                Ok(value) => Ok(value == 0),
                Err(_) => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
async fn set_windows_prtscr_disabled(disabled: bool) -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags("Control Panel\\Keyboard", KEY_SET_VALUE)
        .map_err(|e| e.to_string())?;

    let value: u32 = if disabled { 0 } else { 1 };
    key.set_value("PrintScreenKeyForSnippingEnabled", &value)
        .map_err(|e| e.to_string())?;

    Ok(disabled)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
async fn get_windows_prtscr_disabled() -> Result<bool, String> {
    Ok(false)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
async fn set_windows_prtscr_disabled(_disabled: bool) -> Result<bool, String> {
    Ok(false)
}

#[tauri::command]
async fn capture_screen(_app: AppHandle, state: State<'_, AppState>) -> Result<CaptureInfo, String> {
    println!("[PERF] {} capture_screen command called", Local::now().format("%H:%M:%S%.3f"));
    // Return the temp file path for asset protocol loading
    let path = state
        .current_screenshot_path
        .lock()
        .unwrap()
        .clone()
        .ok_or("No screenshot available. Capture was not performed before opening selection window.")?;
    let (width, height) = state
        .current_screenshot
        .lock()
        .unwrap()
        .as_ref()
        .map(|r| (r.width, r.height))
        .ok_or("No screenshot available")?;
    let monitors = state.monitors.lock().unwrap().clone();
    println!("[PERF] {} Returning path: {} ({}x{}, {} monitor(s))", Local::now().format("%H:%M:%S%.3f"), path, width, height, monitors.len());
    Ok(CaptureInfo { path, width, height, monitors })
}

#[tauri::command]
async fn process_selection(
    _app: AppHandle,
    state: State<'_, AppState>,
    bounds: SelectionBounds,
) -> Result<(), String> {
    println!("[LOG] {} process_selection called with bounds: {:?}", Local::now().format("%H:%M:%S%.3f"), bounds);
    let current = state.current_screenshot.lock().unwrap();
    let raw = current.as_ref().ok_or("No screenshot available")?;

    // Create image from raw RGBA data
    let img = RgbaImage::from_raw(raw.width, raw.height, raw.data.clone())
        .ok_or("Failed to create image from raw data")?;
    let dynamic_img = DynamicImage::ImageRgba8(img);

    // Crop the image (selection is in CSS pixels of the selection window)
    let (crop_x, crop_y, crop_w, crop_h) = bounds
        .to_image_rect(raw.width, raw.height)
        .ok_or("Empty selection")?;
    let cropped = dynamic_img.crop_imm(crop_x, crop_y, crop_w, crop_h);

    // Convert to PNG bytes for final storage (quality matters here)
    let mut buffer = Cursor::new(Vec::new());
    cropped.write_to(&mut buffer, ImageFormat::Png).map_err(|e| e.to_string())?;
    let cropped_data = buffer.into_inner();

    // Store pending screenshot
    drop(current); // Release lock before acquiring another
    let mut pending = state.pending_screenshot.lock().unwrap();
    *pending = Some(PendingScreenshot {
        image_data: cropped_data,
        default_filename: generate_default_filename(),
    });
    drop(pending); // Release lock

    // filename-dialog is already open (opened simultaneously with selection window)
    // Save will be triggered from the dialog

    Ok(())
}

#[tauri::command]
async fn save_screenshot(
    app: AppHandle,
    state: State<'_, AppState>,
    data: SaveData,
) -> Result<String, String> {
    println!("[LOG] {} save_screenshot called", Local::now().format("%H:%M:%S%.3f"));
    let pending = {
        let mut lock = state.pending_screenshot.lock().unwrap();
        lock.take()
    };

    let screenshot = pending.ok_or("No pending screenshot")?;

    // Get save path
    let save_path = get_save_path(app.clone()).await?;
    let save_dir = PathBuf::from(&save_path);

    // Create directory if needed
    if !save_dir.exists() {
        fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;
    }

    // Get timestamp options and format
    let extension = if data.image_format == "png" { "png" } else { "jpg" };
    let filename = format!("{}.{}", data.filename, extension);
    let full_path = save_dir.join(&filename);

    // Apply timestamp
    let processed = add_timestamp_to_image(
        &screenshot.image_data,
        &data.timestamp_options,
        &data.image_format,
    )?;

    // Save file
    fs::write(&full_path, &processed.encoded_bytes).map_err(|e| e.to_string())?;

    // Copy to clipboard if enabled (after successful file save per FR-006)
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let clipboard_enabled = store.get("clipboardCopyEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default: enabled (FR-003)

    if clipboard_enabled {
        // Use raw RGBA directly - no re-decoding needed
        if let Err(e) = copy_rgba_to_clipboard(processed.rgba_pixels, processed.width, processed.height) {
            // FR-005: File save succeeded, emit error event but don't fail
            println!("[LOG] {} Clipboard copy failed: {}", Local::now().format("%H:%M:%S%.3f"), e);
            let _ = app.emit("clipboard-copy-failed", ClipboardErrorPayload {
                message: "Screenshot saved but clipboard copy failed".to_string(),
            });
        } else {
            println!("[LOG] {} Screenshot copied to clipboard", Local::now().format("%H:%M:%S%.3f"));
        }
    }

    // Save settings
    set_timestamp_options(app.clone(), data.timestamp_options).await?;
    set_image_format(app.clone(), data.image_format).await?;

    // Show file in explorer
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .args(["/select,", &full_path.to_string_lossy()])
            .spawn();
    }

    Ok(full_path.to_string_lossy().to_string())
}

// Copy to clipboard only (without saving to disk)
#[tauri::command]
async fn copy_to_clipboard_only(
    app: AppHandle,
    state: State<'_, AppState>,
    timestamp_options: TimestampOptions,
    image_format: String,
) -> Result<(), String> {
    println!("[LOG] {} copy_to_clipboard_only called", Local::now().format("%H:%M:%S%.3f"));

    // Get pending screenshot (don't take ownership yet in case of failure)
    let image_data = {
        let pending = state.pending_screenshot.lock().unwrap();
        let screenshot = pending.as_ref().ok_or("No pending screenshot")?;
        screenshot.image_data.clone()
    };

    // Apply timestamp if enabled (FR-009)
    let processed = add_timestamp_to_image(&image_data, &timestamp_options, &image_format)?;

    // Copy to clipboard - use raw RGBA directly
    copy_rgba_to_clipboard(processed.rgba_pixels, processed.width, processed.height)?;

    // Success: clear pending screenshot
    {
        let mut pending = state.pending_screenshot.lock().unwrap();
        *pending = None;
    }

    // Save timestamp options for next time
    set_timestamp_options(app.clone(), timestamp_options).await?;
    set_image_format(app, image_format).await?;

    println!("[LOG] {} Screenshot copied to clipboard (no file saved)", Local::now().format("%H:%M:%S%.3f"));
    Ok(())
}

#[tauri::command]
async fn get_default_filename(state: State<'_, AppState>) -> Result<String, String> {
    println!("[LOG] {} get_default_filename called", Local::now().format("%H:%M:%S%.3f"));
    let pending = state.pending_screenshot.lock().unwrap();
    if let Some(p) = pending.as_ref() {
        Ok(p.default_filename.clone())
    } else {
        Ok(generate_default_filename())
    }
}

#[tauri::command]
async fn get_preview_image(state: State<'_, AppState>) -> Result<String, String> {
    println!("[LOG] {} get_preview_image called", Local::now().format("%H:%M:%S%.3f"));
    let pending = state.pending_screenshot.lock().unwrap();
    let screenshot = pending.as_ref().ok_or("No pending screenshot")?;

    use base64::{Engine as _, engine::general_purpose};
    Ok(general_purpose::STANDARD.encode(&screenshot.image_data))
}

#[tauri::command]
async fn cancel_screenshot(state: State<'_, AppState>) -> Result<(), String> {
    println!("[LOG] {} cancel_screenshot called", Local::now().format("%H:%M:%S%.3f"));
    let mut pending = state.pending_screenshot.lock().unwrap();
    *pending = None;
    drop(pending);
    let mut current = state.current_screenshot.lock().unwrap();
    *current = None;
    drop(current);
    let mut path_cache = state.current_screenshot_path.lock().unwrap();
    *path_cache = None;
    drop(path_cache);
    *state.selection_bounds.lock().unwrap() = None;
    Ok(())
}

/// Open the options dialog, centred on `monitor` (virtual-screen physical
/// coordinates) when given, otherwise on the primary monitor.
fn open_filename_dialog<R: Runtime>(app: &AppHandle<R>, monitor: Option<Rect>) -> Result<(), String> {
    println!("[LOG] {} open_filename_dialog called", Local::now().format("%H:%M:%S%.3f"));
    let existing = app.get_webview_window("filename-dialog");
    if existing.is_some() {
        return Ok(());
    }

    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = Image::from_bytes(icon_bytes).map_err(|e| e.to_string())?;

    let dialog = WebviewWindowBuilder::new(app, "filename-dialog", WebviewUrl::App("filename-dialog.html".into()))
        .title("Smart PrtScr - Options")
        .icon(icon).map_err(|e| e.to_string())?
        .inner_size(480.0, 400.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .center()
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(m) = monitor {
        let size = dialog.outer_size().map_err(|e| e.to_string())?;
        let x = m.x + (m.width as i32 - size.width as i32) / 2;
        let y = m.y + (m.height as i32 - size.height as i32) / 2;
        dialog
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())?;
    }
    dialog.show().map_err(|e| e.to_string())?;
    dialog.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

/// Capture every connected monitor and compose them into one image covering
/// the whole virtual desktop.
fn capture_desktop() -> Result<desktop::ComposedDesktop, String> {
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let mut captures = Vec::with_capacity(screens.len());
    for screen in &screens {
        let info = screen.display_info;
        let capture = screen.capture().map_err(|e| e.to_string())?;
        let (w, h) = (capture.width(), capture.height());
        let image = RgbaImage::from_raw(w, h, capture.as_raw().to_vec())
            .ok_or("Failed to create image from raw data")?;
        println!(
            "[CAPTURE] monitor id={} at ({}, {}) {}x{} scale={} primary={}",
            info.id, info.x, info.y, w, h, info.scale_factor, info.is_primary
        );
        captures.push(MonitorCapture {
            // Position from the monitor rectangle, size from the actual capture
            // (both physical pixels for a per-monitor DPI aware process).
            rect: Rect::new(info.x, info.y, w, h),
            is_primary: info.is_primary,
            image,
        });
    }
    desktop::compose(&captures).ok_or_else(|| "No screen found".to_string())
}

fn open_selection_window(app: &AppHandle, state: &State<'_, AppState>) -> Result<(), String> {
    use std::time::Instant;
    let start = Instant::now();
    println!("[PERF] {} open_selection_window START", Local::now().format("%H:%M:%S%.3f"));

    let existing = app.get_webview_window("selection");
    if existing.is_some() {
        return Ok(());
    }

    // Capture all screens BEFORE opening the selection window
    let t1 = Instant::now();
    let composed = capture_desktop()?;
    println!("[PERF] {} capture_desktop() took {:?}", Local::now().format("%H:%M:%S%.3f"), t1.elapsed());

    let width = composed.image.width();
    let height = composed.image.height();

    // Write BMP to temp file for fast loading via asset protocol
    let t4 = Instant::now();
    let temp_path = std::env::temp_dir().join("smart-prtscr-preview.bmp");
    composed.image.save(&temp_path).map_err(|e| e.to_string())?;
    let temp_path_str = temp_path.to_string_lossy().to_string();
    println!("[PERF] {} BMP save to temp file took {:?}", Local::now().format("%H:%M:%S%.3f"), t4.elapsed());

    // Store the screenshot data in state
    {
        let mut current = state.current_screenshot.lock().unwrap();
        *current = Some(RawScreenshot {
            data: composed.image.into_raw(),
            width,
            height,
        });
    }
    {
        let mut path_cache = state.current_screenshot_path.lock().unwrap();
        *path_cache = Some(temp_path_str);
    }
    *state.monitors.lock().unwrap() = composed.monitors;
    println!("[PERF] {} Total before window creation: {:?}", Local::now().format("%H:%M:%S%.3f"), start.elapsed());

    let t6 = Instant::now();
    create_selection_window(app, composed.bounds)?;
    println!("[PERF] {} Window creation took {:?}", Local::now().format("%H:%M:%S%.3f"), t6.elapsed());

    println!("[PERF] {} TOTAL open_selection_window: {:?}", Local::now().format("%H:%M:%S%.3f"), start.elapsed());

    // Window will be shown by frontend after image is loaded (via show_selection_window command)

    Ok(())
}

/// Place a window exactly over `bounds` (physical pixels). Size is applied
/// after the position: moving onto a monitor with another DPI makes Windows
/// rescale the window, which the second call corrects.
fn place_window<R: Runtime>(window: &tauri::WebviewWindow<R>, bounds: Rect) -> Result<(), String> {
    window
        .set_position(tauri::PhysicalPosition::new(bounds.x, bounds.y))
        .map_err(|e| e.to_string())?;
    window
        .set_size(tauri::PhysicalSize::new(bounds.width, bounds.height))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Create the (hidden) selection window; the frontend shows it once the
/// screenshot is loaded (show_selection_window).
/// The window spans the whole virtual desktop (all monitors), so it is a
/// borderless window placed in physical pixels rather than a fullscreen
/// window, which would only cover one monitor.
fn create_selection_window<R: Runtime>(app: &AppHandle<R>, bounds: Rect) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        *state.selection_bounds.lock().unwrap() = None;
        *state.desktop_bounds.lock().unwrap() = Some(bounds);
    }
    let window = WebviewWindowBuilder::new(app, "selection", WebviewUrl::App("selection.html".into()))
        .title("Selection")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;
    place_window(&window, bounds)?;

    // The options dialog is NOT opened here: it appears once the user has
    // selected a zone (on_selection_ready).
    Ok(())
}

/// Called each time the user selects / moves / resizes the zone. The first
/// call opens the options dialog; later calls update it.
fn on_selection_ready<R: Runtime>(app: &AppHandle<R>, bounds: SelectionBounds) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        *state.selection_bounds.lock().unwrap() = Some(bounds.clone());
    }
    if app.get_webview_window("filename-dialog").is_none() {
        // The dialog reads the stored bounds at startup (get_selection_bounds).
        open_filename_dialog(app, dialog_monitor(app, &bounds))?;
    } else {
        let _ = app.emit_to("filename-dialog", "selection-bounds-changed", bounds);
    }
    Ok(())
}

/// Monitor holding the centre of the selection (virtual-screen coordinates).
fn dialog_monitor<R: Runtime>(app: &AppHandle<R>, bounds: &SelectionBounds) -> Option<Rect> {
    let state = app.state::<AppState>();
    let desktop = (*state.desktop_bounds.lock().unwrap())?;
    let monitors = state.monitors.lock().unwrap();
    let crop = bounds.to_image_rect(desktop.width, desktop.height)?;
    desktop::monitor_for_selection(&desktop, &monitors, crop)
}

#[tauri::command]
async fn selection_ready(app: AppHandle, bounds: SelectionBounds) -> Result<(), String> {
    on_selection_ready(&app, bounds)
}

#[tauri::command]
async fn get_selection_bounds(state: State<'_, AppState>) -> Result<Option<SelectionBounds>, String> {
    Ok(state.selection_bounds.lock().unwrap().clone())
}

fn open_main_window(app: &AppHandle) -> Result<(), String> {
    println!("[LOG] {} open_main_window called", Local::now().format("%H:%M:%S%.3f"));
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = Image::from_bytes(icon_bytes).map_err(|e| e.to_string())?;

    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Smart PrtScr")
        .icon(icon).map_err(|e| e.to_string())?
        .inner_size(450.0, 300.0)
        .resizable(true)
        .decorations(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn start_capture(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    println!("[LOG] {} start_capture called", Local::now().format("%H:%M:%S%.3f"));
    open_selection_window(&app, &state)
}

#[tauri::command]
async fn close_window(app: AppHandle, label: String) -> Result<(), String> {
    println!("[LOG] {} close_window called for: {}", Local::now().format("%H:%M:%S%.3f"), label);
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_window(app: AppHandle, label: String) -> Result<(), String> {
    println!("[LOG] {} hide_window called for: {}", Local::now().format("%H:%M:%S%.3f"), label);
    if let Some(window) = app.get_webview_window(&label) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn log_message(message: String) -> Result<(), String> {
    println!("[FRONTEND] {} {}", Local::now().format("%H:%M:%S%.3f"), message);
    Ok(())
}

#[tauri::command]
async fn show_selection_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    println!("[LOG] {} show_selection_window called", Local::now().format("%H:%M:%S%.3f"));
    if let Some(window) = app.get_webview_window("selection") {
        let bounds = *state.desktop_bounds.lock().unwrap();
        if let Some(bounds) = bounds {
            place_window(&window, bounds)?;
        }
        window.show().map_err(|e| e.to_string())?;
        // Showing can trigger a DPI change; make sure every monitor is still covered.
        if let Some(bounds) = bounds {
            place_window(&window, bounds)?;
        }
        window.set_focus().map_err(|e| e.to_string())?;
    }
    // Bring filename-dialog to front after selection window is shown
    if let Some(dialog) = app.get_webview_window("filename-dialog") {
        dialog.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn focus_dialog(app: AppHandle) -> Result<(), String> {
    if let Some(dialog) = app.get_webview_window("filename-dialog") {
        dialog.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = open_main_window(app);
        }))
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_all_settings,
            get_save_path,
            set_save_path,
            reset_save_path,
            get_timestamp_options,
            set_timestamp_options,
            reset_timestamp_options,
            get_image_format,
            set_image_format,
            get_clipboard_copy_enabled,
            set_clipboard_copy_enabled,
            get_auto_start,
            set_auto_start,
            get_windows_prtscr_disabled,
            set_windows_prtscr_disabled,
            capture_screen,
            process_selection,
            save_screenshot,
            copy_to_clipboard_only,
            get_default_filename,
            get_preview_image,
            cancel_screenshot,
            start_capture,
            close_window,
            hide_window,
            open_folder,
            log_message,
            show_selection_window,
            focus_dialog,
            selection_ready,
            get_selection_bounds,
        ])
        .setup(|app| {
            // Setup system tray
            let open_item = MenuItemBuilder::with_id("open", "Options").build(app)?;
            let capture_item = MenuItemBuilder::with_id("capture", "Capturer (PrtScr)").build(app)?;
            let folder_item = MenuItemBuilder::with_id("folder", "Dossier de sauvegarde").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quitter").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&open_item)
                .item(&capture_item)
                .separator()
                .item(&folder_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let icon_bytes = include_bytes!("../icons/icon.ico");
            let icon = Image::from_bytes(icon_bytes).unwrap();

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Smart PrtScr - Clic : capturer | Clic droit : menu")
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "open" => {
                            let _ = open_main_window(app);
                        }
                        "capture" => {
                            let state: State<'_, AppState> = app.state();
                            let _ = open_selection_window(app, &state);
                        }
                        "folder" => {
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Ok(path) = get_save_path(app_clone).await {
                                    #[cfg(target_os = "windows")]
                                    let _ = std::process::Command::new("explorer")
                                        .arg(&path)
                                        .spawn();
                                }
                            });
                        }
                        "quit" => {
                            #[cfg(target_os = "windows")]
                            keyboard_hook::stop_hook();
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, button_state, .. } = event {
                        let app = tray.app_handle();
                        if let Some(TrayClickAction::StartCapture) = tray_click_action(button, button_state) {
                            let state: State<'_, AppState> = app.state();
                            let _ = open_selection_window(app, &state);
                        }
                    }
                })
                .build(app)?;

            // Event listener for capture hotkey
            let app_handle = app.handle().clone();
            app.listen("trigger-capture", move |_event| {
                println!("[EVENT] trigger-capture received");
                let state: State<'_, AppState> = app_handle.state();
                if let Some(window) = app_handle.get_webview_window("selection") {
                    let _ = window.emit("capture-full-screen", ());
                } else {
                    let _ = open_selection_window(&app_handle, &state);
                }
            });

            // Start global hotkey handler
            #[cfg(target_os = "windows")]
            keyboard_hook::start_hook(app.handle().clone());

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // Prevent app from exiting when all windows are closed
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .manage(AppState::default())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("mock app")
    }

    /// Real capture on this machine, compared with Windows' virtual-screen
    /// metrics. Run explicitly: `cargo test --lib -- --ignored capture_desktop`
    #[cfg(target_os = "windows")]
    #[test]
    #[ignore = "captures the real screens"]
    fn capture_desktop_covers_the_whole_virtual_screen() {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CMONITORS, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
            SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
        };
        // Same DPI awareness as the app (set by Tauri at startup).
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let expected = unsafe {
            Rect::new(
                GetSystemMetrics(SM_XVIRTUALSCREEN),
                GetSystemMetrics(SM_YVIRTUALSCREEN),
                GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32,
                GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32,
            )
        };
        let monitor_count = unsafe { GetSystemMetrics(SM_CMONITORS) } as usize;

        let desktop = capture_desktop().expect("capture");
        println!("virtual screen {:?}, monitors {:?}", desktop.bounds, desktop.monitors);
        assert_eq!(desktop.bounds, expected);
        assert_eq!(desktop.image.dimensions(), (expected.width, expected.height));
        assert_eq!(desktop.monitors.len(), monitor_count);
        assert_eq!(desktop.monitors.iter().filter(|m| m.primary).count(), 1);
    }

    #[test]
    fn options_dialog_appears_only_after_the_zone_is_selected() {
        let app = mock_app();
        let handle = app.handle();

        create_selection_window(handle, Rect::new(0, 0, 1920, 1080)).unwrap();
        assert!(handle.get_webview_window("selection").is_some());
        assert!(
            handle.get_webview_window("filename-dialog").is_none(),
            "options dialog must not be open while the user is still selecting the zone"
        );

        let bounds = SelectionBounds {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 200.0,
            viewport_width: 1920.0,
            viewport_height: 1080.0,
        };
        on_selection_ready(handle, bounds).unwrap();
        assert!(handle.get_webview_window("filename-dialog").is_some());
        let stored = handle.state::<AppState>().selection_bounds.lock().unwrap().clone();
        assert_eq!(stored.map(|b| (b.x, b.y, b.width, b.height)), Some((10.0, 20.0, 300.0, 200.0)));
    }

    #[test]
    fn tray_left_click_starts_a_capture() {
        assert_eq!(
            tray_click_action(MouseButton::Left, MouseButtonState::Up),
            Some(TrayClickAction::StartCapture)
        );
    }

    #[test]
    fn tray_other_clicks_do_nothing_so_the_right_click_menu_shows() {
        assert_eq!(tray_click_action(MouseButton::Left, MouseButtonState::Down), None);
        assert_eq!(tray_click_action(MouseButton::Right, MouseButtonState::Up), None);
        assert_eq!(tray_click_action(MouseButton::Middle, MouseButtonState::Up), None);
    }
}
