use gtk4::cairo::Context;
use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::glib::Bytes;
use gtk4::prelude::GdkCairoContextExt;
use crate::painter::commands::border::BorderStyle;
use crate::painter::commands::brush::Brush;
use crate::painter::commands::rectangle::Rectangle;
use crate::tiler::Tile;

fn set_brush(cr: &Context, brush: &Brush) {
    match brush {
        Brush::Solid(color) => {
            cr.set_source_rgba(color.r() as f64, color.g() as f64, color.b() as f64, color.a() as f64);
        }
        Brush::Image(img) => {
            let bytes = Bytes::from(img.data());
            let pixbuf = Pixbuf::from_bytes(&bytes, Colorspace::Rgb, true, 8, img.width() as i32, img.height() as i32, img.width() as i32 * 4);
            cr.set_source_pixbuf(&pixbuf, 0.0, 0.0);
        }
    }
}

pub(crate) fn do_paint_rectangle(cr: &Context, tile: &Tile, rectangle: &Rectangle) {
    cr.translate(-tile.rect.x, -tile.rect.y);

    // Create initial rect
    match rectangle.background() {
        Some(brush) => {
            cr.rectangle(rectangle.rect().x, rectangle.rect().y, rectangle.rect().width, rectangle.rect().height);
            set_brush(cr, brush);
            _ = cr.fill();
        }
        None => {}
    }

    // Create border
    cr.rectangle(rectangle.rect().x, rectangle.rect().y, rectangle.rect().width, rectangle.rect().height);
    cr.set_line_width(rectangle.border().width() as f64);
    set_brush(cr, &rectangle.border().brush());
    match rectangle.border().style() {
        BorderStyle::None => {
            // No border to draw. Note that the border does not take up any space. This is already
            // calculated in the boxmodel by the layouter.
        }
        BorderStyle::Solid => {
            // Complete solid line
            _ = cr.stroke();
        }
        BorderStyle::Dashed => {
            // 50px dash, 10px gap, 10px dash, 10px gap
            cr.set_dash(&[50.0, 10.0, 10.0, 10.0], 0.0);
            _ = cr.stroke();
        }
        BorderStyle::Dotted => {
            // 10px dash, 10px gap
            cr.set_dash(&[10.0, 10.0], 0.0);
            _ = cr.stroke();
        }
        BorderStyle::Double => {
            if rectangle.border().width() >= 3.0 {
                // The formula  outer border: (N-1) / 2, 1px gap, inner border: (N-1) / 2

                // Outer border
                let width = (rectangle.border().width() / 2.0).floor();
                cr.set_line_width(width as f64);
                _ = cr.stroke();

                let gap_size = 1.0;

                // inner border
                let width = (rectangle.border().width() / 2.0).floor();
                cr.rectangle(
                    rectangle.rect().x + width as f64 + gap_size,
                    rectangle.rect().y + width as f64 + gap_size,
                    rectangle.rect().width - width as f64 - gap_size,
                    rectangle.rect().height - width as f64 - gap_size
                );
                cr.set_line_width(width as f64);
                _ = cr.stroke();
            } else {
                // When the width is less than 3.0, we just draw a single line as there is no room for
                // a double border
                _ = cr.stroke();
            }
        }
        BorderStyle::Groove => {}
        BorderStyle::Ridge => {}
        BorderStyle::Inset => {}
        BorderStyle::Outset => {}
        BorderStyle::Hidden => {
            // Don't display anything. But the border still takes up space. This is already
            // calculated in the boxmodel by the layouter.
        }
    }
    cr.rectangle(rectangle.rect().x, rectangle.rect().y, rectangle.rect().width, rectangle.rect().height);
}