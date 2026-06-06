# Third-Party Notices

This document lists the third-party software and assets used by Bludio, along with their respective licenses.

## Dependencies

The following Rust crates are used by Bludio. All are compatible with the MIT license under which Bludio is distributed.

| Crate | License | MIT Compatible |
|-------|---------|----------------|
| `gpui-unofficial` | Apache-2.0 | Yes |
| `gpui-platform-gpui-unofficial` | Apache-2.0 | Yes |
| `bluer` | BSD-2-Clause | Yes |
| `tokio` | MIT | Yes |
| `futures` | MIT / Apache-2.0 | Yes |
| `libpulse-binding` | MIT / Apache-2.0 | Yes |
| `libpulse-sys` | MIT / Apache-2.0 | Yes |
| `serde` | MIT / Apache-2.0 | Yes |
| `serde_json` | MIT / Apache-2.0 | Yes |
| `unicode-segmentation` | MIT / Apache-2.0 | Yes |

**Note:** The underlying PulseAudio C library (`libpulse`) is LGPL-2.1+. Bludio links to it dynamically at runtime via the MIT/Apache-2.0 licensed Rust bindings above. This does not impose copyleft requirements on Bludio itself.

## Assets

The following fonts and icons are bundled with Bludio and distributed under their respective open-source licenses.

| Asset | Author | License | License File |
|-------|--------|---------|--------------|
| Noto Sans | The Noto Project Authors | SIL Open Font License 1.1 | [`licenses/LICENSE-NOTO-SANS`](licenses/LICENSE-NOTO-SANS) |
| OpenDyslexic | Abbie Gonzalez | SIL Open Font License 1.1 | [`licenses/LICENSE-OPENDYSLEXIC`](licenses/LICENSE-OPENDYSLEXIC) |
| Lucide Icons | Lucide Contributors | ISC License | [`licenses/LICENSE-LUCIDE`](licenses/LICENSE-LUCIDE) |
