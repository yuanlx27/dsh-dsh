# Local icon provenance

`icon.png` and `icon.icns` are raster/container conversions of the pinned R
`@deepseek-ai/dsh-web-frontend/dist/favicon.svg` (0.2.0-rc.2), generated with the
installed locked Tauri CLI's `icon` command. Only macOS outputs are retained;
other-platform output was temporary. The upstream source remains in the frozen
runtime closure. This local experimental desktop is not an official release;
About explicitly identifies the local ad-hoc experimental build.

Regeneration: run `node_modules/.bin/tauri icon
runtime/node_modules/@deepseek-ai/dsh-web-frontend/dist/favicon.svg --output
<temporary-directory>`, then copy only `icon.png` and `icon.icns` here. Follow
upstream licensing/provenance in the bundled package closure. No new logo or
JavaScript source is generated or committed.
