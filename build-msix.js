const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const makeappx = 'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.26100.0\\x64\\makeappx.exe';
const inputDir = path.join(__dirname, 'dist', 'win-unpacked');
const outputDir = path.join(__dirname, 'dist', 'appx');
const outputFile = path.join(outputDir, 'SmartPrtScr.appx');

// Create output directory
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

// Create AppxManifest.xml
const manifest = `<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities">
  <Identity Name="Pleymor.smart-prtscr"
            Publisher="CN=B79D1225-9D8B-4940-8036-D7FDBD5EA2D3"
            Version="1.0.0.0"
            ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>smart-prtscr</DisplayName>
    <PublisherDisplayName>Pleymor</PublisherDisplayName>
    <Logo>Assets\\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.22621.0" />
  </Dependencies>
  <Resources>
    <Resource Language="en-us" />
    <Resource Language="fr-fr" />
  </Resources>
  <Applications>
    <Application Id="SmartPrtScr"
                 Executable="Smart PrtScr.exe"
                 EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="smart-prtscr"
                          Description="Simple and fast screenshot capture tool for Windows"
                          BackgroundColor="#667eea"
                          Square150x150Logo="Assets\\Square150x150Logo.png"
                          Square44x44Logo="Assets\\Square44x44Logo.png">
        <uap:DefaultTile Wide310x150Logo="Assets\\Wide310x150Logo.png" />
      </uap:VisualElements>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>`;

// Create Assets folder and placeholder images
const assetsDir = path.join(inputDir, 'Assets');
if (!fs.existsSync(assetsDir)) {
  fs.mkdirSync(assetsDir, { recursive: true });
}

// Copy icon as placeholder (we'll use the existing icon.ico converted)
const sharp = require('sharp');

async function createAssets() {
  const iconPath = path.join(__dirname, 'icon.ico');

  // Create placeholder PNGs from a simple colored square
  const sizes = [
    { name: 'StoreLogo.png', size: 50 },
    { name: 'Square150x150Logo.png', size: 150 },
    { name: 'Square44x44Logo.png', size: 44 },
    { name: 'Wide310x150Logo.png', width: 310, height: 150 }
  ];

  for (const s of sizes) {
    const width = s.width || s.size;
    const height = s.height || s.size;
    await sharp({
      create: {
        width,
        height,
        channels: 4,
        background: { r: 102, g: 126, b: 234, alpha: 1 }
      }
    }).png().toFile(path.join(assetsDir, s.name));
    console.log(`Created ${s.name}`);
  }
}

async function build() {
  console.log('Creating assets...');
  await createAssets();

  console.log('Writing AppxManifest.xml...');
  fs.writeFileSync(path.join(inputDir, 'AppxManifest.xml'), manifest);

  console.log('Building APPX package...');
  try {
    execSync(`"${makeappx}" pack /d "${inputDir}" /p "${outputFile}" /o`, { stdio: 'inherit' });
    console.log(`\\nSuccess! Package created: ${outputFile}`);
  } catch (error) {
    console.error('Build failed:', error.message);
    process.exit(1);
  }
}

build();
