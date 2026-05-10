const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const svgPath = path.join(__dirname, '../src-tauri/icons/icon.svg');
const iconsDir = path.join(__dirname, '../src-tauri/icons');

const sizes = [
  { name: '32x32.png', size: 32 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: 'icon.png', size: 512 },
  { name: 'Square30x30Logo.png', size: 30 },
  { name: 'Square44x44Logo.png', size: 44 },
  { name: 'Square71x71Logo.png', size: 71 },
  { name: 'Square89x89Logo.png', size: 89 },
  { name: 'Square107x107Logo.png', size: 107 },
  { name: 'Square142x142Logo.png', size: 142 },
  { name: 'Square150x150Logo.png', size: 150 },
  { name: 'Square284x284Logo.png', size: 284 },
  { name: 'Square310x310Logo.png', size: 310 },
  { name: 'StoreLogo.png', size: 50 },
];

async function convert() {
  const svgBuffer = fs.readFileSync(svgPath);
  
  // Create PNGs
  for (const { name, size } of sizes) {
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(path.join(iconsDir, name));
    console.log(`Created ${name}`);
  }
  
  // Create .ico (use smaller sizes)
  try {
    const toIco = (await import('to-ico')).default;
    const ico = await toIco([
      fs.readFileSync(path.join(iconsDir, '32x32.png')),
      fs.readFileSync(path.join(iconsDir, '128x128.png'))
    ]);
    fs.writeFileSync(path.join(iconsDir, 'icon.ico'), ico);
    console.log('Created icon.ico');
  } catch (e) {
    console.log('ICO error:', e.message);
  }
  
  // Create .icns
  try {
    const pngToIcns = (await import('png-to-icns')).default;
    const icns = await pngToIcns(path.join(iconsDir, 'icon.png'));
    fs.writeFileSync(path.join(iconsDir, 'icon.icns'), icns);
    console.log('Created icon.icns');
  } catch (e) {
    console.log('ICNS error:', e.message);
  }
  
  console.log('Done!');
}

convert();