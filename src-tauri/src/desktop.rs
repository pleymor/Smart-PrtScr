//! Virtual-desktop geometry: composing per-monitor captures into one image
//! and mapping a selection drawn in the selection window back to image pixels.
//!
//! All monitor rectangles are in physical pixels, in Windows virtual-screen
//! coordinates (the primary monitor's top-left is 0,0; monitors placed left of
//! or above it have negative coordinates).

use image::RgbaImage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn right(&self) -> i64 {
        self.x as i64 + self.width as i64
    }

    fn bottom(&self) -> i64 {
        self.y as i64 + self.height as i64
    }

    pub fn contains(&self, x: i64, y: i64) -> bool {
        x >= self.x as i64 && x < self.right() && y >= self.y as i64 && y < self.bottom()
    }
}

/// One monitor's capture, positioned in virtual-screen coordinates.
pub struct MonitorCapture {
    pub rect: Rect,
    pub is_primary: bool,
    pub image: RgbaImage,
}

/// A monitor region inside the composed desktop image (image pixels, origin
/// at the composed image's top-left corner).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MonitorRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
}

/// The whole virtual desktop captured as a single image.
pub struct ComposedDesktop {
    /// Position and size of the image in virtual-screen coordinates.
    pub bounds: Rect,
    pub image: RgbaImage,
    pub monitors: Vec<MonitorRegion>,
}

/// Smallest rectangle containing every monitor.
pub fn virtual_bounds(rects: &[Rect]) -> Option<Rect> {
    let first = rects.first()?;
    let (mut left, mut top) = (first.x as i64, first.y as i64);
    let (mut right, mut bottom) = (first.right(), first.bottom());
    for r in &rects[1..] {
        left = left.min(r.x as i64);
        top = top.min(r.y as i64);
        right = right.max(r.right());
        bottom = bottom.max(r.bottom());
    }
    Some(Rect::new(
        left as i32,
        top as i32,
        (right - left) as u32,
        (bottom - top) as u32,
    ))
}

/// Paste every monitor capture at its place in the virtual desktop. Areas not
/// covered by any monitor (non-rectangular layouts) stay transparent black.
pub fn compose(captures: &[MonitorCapture]) -> Option<ComposedDesktop> {
    let rects: Vec<Rect> = captures.iter().map(|c| c.rect).collect();
    let bounds = virtual_bounds(&rects)?;
    let mut image = RgbaImage::new(bounds.width, bounds.height);
    let mut monitors = Vec::with_capacity(captures.len());

    for capture in captures {
        let off_x = (capture.rect.x as i64 - bounds.x as i64) as u32;
        let off_y = (capture.rect.y as i64 - bounds.y as i64) as u32;
        // The capture should match the monitor rectangle; never write outside it.
        let w = capture.image.width().min(capture.rect.width);
        let h = capture.image.height().min(capture.rect.height);
        for y in 0..h {
            let src_start = (y * capture.image.width() * 4) as usize;
            let src = &capture.image.as_raw()[src_start..src_start + (w * 4) as usize];
            let dst_start = (((off_y + y) * bounds.width + off_x) * 4) as usize;
            let dst: &mut [u8] = &mut image;
            dst[dst_start..dst_start + (w * 4) as usize].copy_from_slice(src);
        }
        monitors.push(MonitorRegion {
            x: off_x,
            y: off_y,
            width: capture.rect.width,
            height: capture.rect.height,
            primary: capture.is_primary,
        });
    }

    Some(ComposedDesktop {
        bounds,
        image,
        monitors,
    })
}

/// Map a selection expressed in the selection window's CSS pixels to a crop
/// rectangle in image pixels. The selection window displays the composed image
/// stretched over its whole viewport, so the ratio image/viewport converts CSS
/// pixels to physical pixels whatever the window's DPI scale factor is.
///
/// Returns `(x, y, width, height)` clamped to the image, or `None` when the
/// selection is empty or entirely outside the image.
/// `selection` is `(x, y, width, height)`, `viewport` and `image` are `(width, height)`.
pub fn selection_to_image_rect(
    selection: (f64, f64, f64, f64),
    viewport: (f64, f64),
    image: (u32, u32),
) -> Option<(u32, u32, u32, u32)> {
    let (sel_x, sel_y, sel_width, sel_height) = selection;
    let (viewport_width, viewport_height) = viewport;
    let (image_width, image_height) = image;
    if viewport_width <= 0.0 || viewport_height <= 0.0 {
        return None;
    }
    let sx = image_width as f64 / viewport_width;
    let sy = image_height as f64 / viewport_height;

    let clamp_x = |v: f64| v.max(0.0).min(image_width as f64);
    let clamp_y = |v: f64| v.max(0.0).min(image_height as f64);

    let left = clamp_x((sel_x * sx).round());
    let top = clamp_y((sel_y * sy).round());
    let right = clamp_x(((sel_x + sel_width) * sx).round());
    let bottom = clamp_y(((sel_y + sel_height) * sy).round());

    if right <= left || bottom <= top {
        return None;
    }
    Some((
        left as u32,
        top as u32,
        (right - left) as u32,
        (bottom - top) as u32,
    ))
}

/// Index of the monitor region containing the point (image pixels), if any.
pub fn monitor_at(monitors: &[MonitorRegion], x: f64, y: f64) -> Option<usize> {
    monitors.iter().position(|m| {
        x >= m.x as f64
            && x < (m.x + m.width) as f64
            && y >= m.y as f64
            && y < (m.y + m.height) as f64
    })
}

/// Monitor (virtual-screen coordinates) on which the options dialog should
/// appear: the one containing the centre of the selection.
pub fn monitor_for_selection(
    desktop: &Rect,
    monitors: &[MonitorRegion],
    crop: (u32, u32, u32, u32),
) -> Option<Rect> {
    let (x, y, w, h) = crop;
    let cx = x as f64 + w as f64 / 2.0;
    let cy = y as f64 + h as f64 / 2.0;
    let m = &monitors[monitor_at(monitors, cx, cy)?];
    Some(Rect::new(
        desktop.x + m.x as i32,
        desktop.y + m.y as i32,
        m.width,
        m.height,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(w: u32, h: u32, v: u8) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([v, v, v, 255]))
    }

    #[test]
    fn virtual_bounds_spans_monitors_with_negative_coordinates() {
        // Secondary 1920x1080 monitor left of a 2560x1440 primary, top-aligned
        // 200px higher; a third portrait monitor right of the primary.
        let rects = [
            Rect::new(0, 0, 2560, 1440),
            Rect::new(-1920, -200, 1920, 1080),
            Rect::new(2560, 0, 1080, 1920),
        ];
        assert_eq!(
            virtual_bounds(&rects),
            Some(Rect::new(-1920, -200, 1920 + 2560 + 1080, 1920 + 200))
        );
    }

    #[test]
    fn compose_places_every_monitor_including_negative_ones() {
        let captures = vec![
            MonitorCapture {
                rect: Rect::new(0, 0, 4, 2),
                is_primary: true,
                image: solid(4, 2, 10),
            },
            MonitorCapture {
                rect: Rect::new(-3, -1, 3, 2),
                is_primary: false,
                image: solid(3, 2, 200),
            },
        ];
        let desktop = compose(&captures).expect("composed desktop");

        assert_eq!(desktop.bounds, Rect::new(-3, -1, 7, 3));
        assert_eq!(desktop.image.dimensions(), (7, 3));
        // Secondary (negative) monitor occupies x 0..3, y 0..2 of the image.
        assert_eq!(desktop.image.get_pixel(0, 0).0[0], 200);
        assert_eq!(desktop.image.get_pixel(2, 1).0[0], 200);
        // Primary occupies x 3..7, y 1..3.
        assert_eq!(desktop.image.get_pixel(3, 1).0[0], 10);
        assert_eq!(desktop.image.get_pixel(6, 2).0[0], 10);
        // Gap not covered by a monitor stays empty.
        assert_eq!(desktop.image.get_pixel(0, 2).0, [0, 0, 0, 0]);

        assert_eq!(
            desktop.monitors,
            vec![
                MonitorRegion {
                    x: 3,
                    y: 1,
                    width: 4,
                    height: 2,
                    primary: true
                },
                MonitorRegion {
                    x: 0,
                    y: 0,
                    width: 3,
                    height: 2,
                    primary: false
                },
            ]
        );
    }

    #[test]
    fn selection_is_scaled_from_css_pixels_to_image_pixels() {
        // Window at 150% scale: a 2560x1440 physical desktop shows as 1706.67x960 CSS px.
        let rect = selection_to_image_rect(
            (100.0, 50.0, 200.0, 100.0),
            (2560.0 / 1.5, 960.0),
            (2560, 1440),
        );
        assert_eq!(rect, Some((150, 75, 300, 150)));
    }

    #[test]
    fn selection_is_clamped_to_the_image() {
        let rect = selection_to_image_rect((-10.0, 90.0, 50.0, 50.0), (100.0, 100.0), (100, 100));
        assert_eq!(rect, Some((0, 90, 40, 10)));
        assert_eq!(
            selection_to_image_rect((200.0, 0.0, 10.0, 10.0), (100.0, 100.0), (100, 100)),
            None
        );
    }

    #[test]
    fn dialog_goes_to_the_monitor_holding_the_selection() {
        // Composed desktop starting at (-1920, -200): secondary on the left.
        let desktop = Rect::new(-1920, -200, 1920 + 2560, 1440 + 200);
        let monitors = vec![
            MonitorRegion {
                x: 1920,
                y: 200,
                width: 2560,
                height: 1440,
                primary: true,
            },
            MonitorRegion {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                primary: false,
            },
        ];
        assert_eq!(
            monitor_for_selection(&desktop, &monitors, (100, 100, 400, 300)),
            Some(Rect::new(-1920, -200, 1920, 1080))
        );
        assert_eq!(
            monitor_for_selection(&desktop, &monitors, (2000, 300, 200, 200)),
            Some(Rect::new(0, 0, 2560, 1440))
        );
    }

    #[test]
    fn monitor_at_finds_the_monitor_under_a_point() {
        let monitors = vec![
            MonitorRegion {
                x: 1920,
                y: 0,
                width: 2560,
                height: 1440,
                primary: true,
            },
            MonitorRegion {
                x: 0,
                y: 200,
                width: 1920,
                height: 1080,
                primary: false,
            },
        ];
        assert_eq!(monitor_at(&monitors, 2000.0, 10.0), Some(0));
        assert_eq!(monitor_at(&monitors, 10.0, 300.0), Some(1));
        assert_eq!(monitor_at(&monitors, 10.0, 10.0), None);
    }
}
