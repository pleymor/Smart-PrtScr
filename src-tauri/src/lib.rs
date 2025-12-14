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
    AppHandle, Emitter, Listener, Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;


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
        TranslateMessage, DispatchMessageW, DestroyWindow,
        WNDCLASSW, MSG, WM_HOTKEY, WINDOW_EX_STYLE, WINDOW_STYLE,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, MOD_WIN, MOD_SHIFT, MOD_NOREPEAT,
    };
    use windows::core::PCWSTR;
    use std::sync::OnceLock;

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SelectionBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub filename: String,
    #[serde(rename = "timestampOptions")]
    pub timestamp_options: TimestampOptions,
    #[serde(rename = "imageFormat")]
    pub image_format: String,
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

// Add timestamp to image
fn add_timestamp_to_image(
    image_data: &[u8],
    options: &TimestampOptions,
    format: &str,
) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(image_data).map_err(|e| e.to_string())?;
    let mut rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();

    if !options.enabled {
        return encode_image(&DynamicImage::ImageRgba8(rgba_img), format);
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
        encode_image(&DynamicImage::ImageRgba8(rgba_img), format)
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
        encode_image(&DynamicImage::ImageRgba8(new_img), format)
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

// Tauri commands
#[tauri::command]
async fn get_all_settings(app: AppHandle) -> Result<AllSettings, String> {
    use tauri_plugin_autostart::ManagerExt;

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let default_path = get_default_screenshot_path();

    // Save path
    let save_path = store.get("screenshotPath")
        .and_then(|p| p.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| default_path.to_string_lossy().to_string());

    // Auto start
    let auto_start = app.autolaunch().is_enabled().unwrap_or(false);

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

    Ok(AllSettings {
        save_path,
        auto_start,
        image_format,
        windows_prtscr_disabled,
        timestamp_options,
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
async fn get_auto_start(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    autostart.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_auto_start(app: AppHandle, enabled: bool) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    Ok(enabled)
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
async fn capture_screen(_app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    println!("[PERF] {} capture_screen command called", Local::now().format("%H:%M:%S%.3f"));
    // Return the temp file path for asset protocol loading
    let path_cache = state.current_screenshot_path.lock().unwrap();
    let path = path_cache.as_ref().ok_or("No screenshot available. Capture was not performed before opening selection window.")?;
    println!("[PERF] {} Returning path: {}", Local::now().format("%H:%M:%S%.3f"), path);
    Ok(path.clone())
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

    // Crop the image
    let cropped = dynamic_img.crop_imm(
        bounds.x as u32,
        bounds.y as u32,
        bounds.width as u32,
        bounds.height as u32,
    );

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
    let final_image = add_timestamp_to_image(
        &screenshot.image_data,
        &data.timestamp_options,
        &data.image_format,
    )?;

    // Save file
    fs::write(&full_path, &final_image).map_err(|e| e.to_string())?;

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
    Ok(())
}

fn open_filename_dialog(app: &AppHandle) -> Result<(), String> {
    println!("[LOG] {} open_filename_dialog called", Local::now().format("%H:%M:%S%.3f"));
    let existing = app.get_webview_window("filename-dialog");
    if existing.is_some() {
        return Ok(());
    }

    let icon_bytes = include_bytes!("../icons/32x32.png");
    let icon = Image::from_bytes(icon_bytes).map_err(|e| e.to_string())?;

    WebviewWindowBuilder::new(app, "filename-dialog", WebviewUrl::App("filename-dialog.html".into()))
        .title("Smart PrtScr - Options")
        .icon(icon).map_err(|e| e.to_string())?
        .inner_size(480.0, 400.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn open_selection_window(app: &AppHandle, state: &State<'_, AppState>) -> Result<(), String> {
    use std::time::Instant;
    let start = Instant::now();
    println!("[PERF] {} open_selection_window START", Local::now().format("%H:%M:%S%.3f"));

    let existing = app.get_webview_window("selection");
    if existing.is_some() {
        return Ok(());
    }

    // Capture screen BEFORE opening the selection window
    let t1 = Instant::now();
    let screens = Screen::all().map_err(|e| e.to_string())?;
    let screen = screens.first().ok_or("No screen found")?;
    println!("[PERF] {} Screen::all() took {:?}", Local::now().format("%H:%M:%S%.3f"), t1.elapsed());

    let t2 = Instant::now();
    let capture = screen.capture().map_err(|e| e.to_string())?;
    println!("[PERF] {} screen.capture() took {:?}", Local::now().format("%H:%M:%S%.3f"), t2.elapsed());

    // Get screen info for window size
    let info = screen.display_info;

    // Store raw RGBA data
    let t3 = Instant::now();
    let rgba_data = capture.as_raw().to_vec();
    let width = capture.width();
    let height = capture.height();
    println!("[PERF] {} as_raw().to_vec() took {:?}", Local::now().format("%H:%M:%S%.3f"), t3.elapsed());

    // Write BMP to temp file for fast loading via asset protocol
    let t4 = Instant::now();
    let img = RgbaImage::from_raw(width, height, rgba_data.clone())
        .ok_or("Failed to create image from raw data")?;
    let temp_path = std::env::temp_dir().join("smart-prtscr-preview.bmp");
    img.save(&temp_path).map_err(|e| e.to_string())?;
    let temp_path_str = temp_path.to_string_lossy().to_string();
    println!("[PERF] {} BMP save to temp file took {:?}", Local::now().format("%H:%M:%S%.3f"), t4.elapsed());

    // Store the screenshot data in state
    {
        let mut current = state.current_screenshot.lock().unwrap();
        *current = Some(RawScreenshot {
            data: rgba_data,
            width,
            height,
        });
    }
    {
        let mut path_cache = state.current_screenshot_path.lock().unwrap();
        *path_cache = Some(temp_path_str);
    }
    println!("[PERF] {} Total before window creation: {:?}", Local::now().format("%H:%M:%S%.3f"), start.elapsed());

    let t6 = Instant::now();
    let _window = WebviewWindowBuilder::new(app, "selection", WebviewUrl::App("selection.html".into()))
        .title("Selection")
        .inner_size(info.width as f64, info.height as f64)
        .position(info.x as f64, info.y as f64)
        .fullscreen(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;
    println!("[PERF] {} Window creation took {:?}", Local::now().format("%H:%M:%S%.3f"), t6.elapsed());

    // Open filename-dialog simultaneously (centered, always on top)
    let t7 = Instant::now();
    open_filename_dialog(app)?;
    println!("[PERF] {} filename-dialog creation took {:?}", Local::now().format("%H:%M:%S%.3f"), t7.elapsed());

    println!("[PERF] {} TOTAL open_selection_window: {:?}", Local::now().format("%H:%M:%S%.3f"), start.elapsed());

    // Window will be shown by frontend after image is loaded (via show_selection_window command)

    Ok(())
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
async fn show_selection_window(app: AppHandle) -> Result<(), String> {
    println!("[LOG] {} show_selection_window called", Local::now().format("%H:%M:%S%.3f"));
    if let Some(window) = app.get_webview_window("selection") {
        window.show().map_err(|e| e.to_string())?;
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
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
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
            get_auto_start,
            set_auto_start,
            get_windows_prtscr_disabled,
            set_windows_prtscr_disabled,
            capture_screen,
            process_selection,
            save_screenshot,
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
        ])
        .setup(|app| {
            // Setup system tray
            let open_item = MenuItemBuilder::with_id("open", "Ouvrir").build(app)?;
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
                .tooltip("Smart PrtScr - Ready")
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
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let _ = open_main_window(tray.app_handle());
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
