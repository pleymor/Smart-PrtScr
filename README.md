# Smart PrtScr

A lightweight screenshot tool for Windows with real-time timestamp preview, multi-monitor support, and system tray integration.

## Features

- **Selection Rectangle**: Draw a custom area to capture with resize handles
- **Real-time Timestamp Preview**: See exactly how your screenshot will look before saving
- **Live Options Editing**: Modify timestamp settings and see changes instantly on the selection
- **Multi-monitor Support**: Works seamlessly across multiple displays
- **Customizable Timestamp**: Banner (dark/light) or overlay mode, position, font size, color, alignment, and text styles
- **System Tray**: Runs quietly in the background with quick access menu
- **Auto-start**: Optional launch at Windows startup
- **Configurable Output**: PNG or JPEG format, custom save folder

## Demo

[![Demo Video](https://img.youtube.com/vi/dIJptb2p3BU/maxresdefault.jpg)](https://www.youtube.com/watch?v=dIJptb2p3BU)

## Installation

### Option 1: Installer (Recommended)

1. Download `Smart PrtScr Setup.exe` from the releases
2. Run the installer and follow the instructions
3. The application starts automatically after installation

### Option 2: Portable Version

1. Download `Smart PrtScr Portable.exe` from the releases
2. Place it anywhere you like
3. Double-click to run

### Option 3: Build from Source

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

### Option 4: Build MSIX for Microsoft Store

#### Prerequisites

- [Windows SDK](https://developer.microsoft.com/windows/downloads/windows-sdk/) (provides `makeappx.exe`)
- Store icons in `src-tauri/icons/`:
  - `StoreLogo.png` (50x50)
  - `Square44x44Logo.png` (44x44)
  - `Square71x71Logo.png` (71x71)
  - `Square150x150Logo.png` (150x150)
  - `Square310x310Logo.png` (310x310)

#### Build Steps

```powershell
# Build and create MSIX package
.\build-msix.ps1

# Skip Tauri build if already built
.\build-msix.ps1 -SkipBuild
```

The MSIX package will be created in `msix-output/`.

#### Submission

1. Upload the unsigned MSIX to [Partner Center](https://partner.microsoft.com/dashboard)
2. Microsoft will sign it with your Store certificate

#### Local Testing (Optional)

```powershell
# Create a self-signed certificate
New-SelfSignedCertificate -Type Custom -Subject "CN=TestCert" -KeyUsage DigitalSignature -FriendlyName "Test Cert" -CertStoreLocation "Cert:\CurrentUser\My"

# Export to PFX and sign
signtool sign /fd SHA256 /a /f cert.pfx /p password .\msix-output\SmartPrtScr_*.msix
```

## Usage

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `PrintScreen` | Open selection window with options dialog |
| `Win+Shift+PrintScreen` | Full screen capture (selection covers entire screen) |
| `Escape` | Cancel capture |

### Capture Workflow

1. Press `PrintScreen` to start a capture
2. Both the selection window and options dialog open simultaneously
3. Draw a rectangle on the screen to select the area
4. The timestamp preview appears on your selection in real-time
5. Adjust timestamp options in the dialog - changes reflect instantly
6. Click **Save** to capture, or press `Escape` to cancel

### System Tray Menu

Right-click the tray icon to:
- Open settings window
- Start a new capture
- Open the screenshots folder
- Quit the application

## Timestamp Options

| Option | Values |
|--------|--------|
| Type | Banner Dark, Banner Light, Overlay |
| Position | Top, Bottom |
| Font Size | 8-72px |
| Text Color | White, Black, Gray, Red, Green, Blue, Yellow, Cyan, Magenta |
| Alignment | Left, Center, Right |
| Style | Bold, Italic, Underline |

## Configuration

- **Save Folder**: Default is `Pictures/Screenshots`, changeable in settings
- **Image Format**: PNG (best quality) or JPEG (smaller files)
- **Filename**: Auto-generated with timestamp, or enter a custom name

## Requirements

- Windows 10 or 11

## Tech Stack

- **Tauri v2**: Lightweight desktop application framework
- **Rust**: Backend for screen capture and image processing
- **HTML/CSS/JS**: Simple frontend UI

## License

ISC

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.
