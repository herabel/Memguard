const path = require('path');
const libPath = path.resolve(__dirname, '../target/release/libmemguard.so');
try {
    process.dlopen({ exports: {} }, libPath);
    console.error('[memguard] Native memory telemetry active');
} catch (err) {
    console.error('[memguard] Failed to load agent:', err.message);
}