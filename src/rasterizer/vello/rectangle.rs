use vello::kurbo;
use vello::kurbo::{Affine, PathEl, Point, Rect, RoundedRect, Shape};
use vello::peniko::{Fill};
use crate::painter::commands::border::BorderStyle;
use crate::painter::commands::rectangle::Rectangle;
use crate::rasterizer::vello::brush::set_brush;
use crate::tiler::Tile;

pub(crate) fn do_paint_rectangle(scene: &mut vello::Scene, tile: &Tile, rect: &Rectangle) {
    // Draw background (if any background brush is defined)
    match rect.background() {
        Some(brush) => {
            let vello_rect = setup_rectangle_path(rect);
            let vello_brush = set_brush(brush, rect.rect());

            scene.fill(
                Fill::NonZero,
                Affine::translate((-tile.rect.x, -tile.rect.y)),
                &vello_brush,
                None,
                &vello_rect,
            );
        }
        None => {}
    }

    // Create border
    match rect.border().style() {
        BorderStyle::None => {},
        BorderStyle::Solid => draw_single_border(scene, rect, vec![], tile),
        BorderStyle::Dashed => draw_single_border(scene, rect, vec![50.0, 10.0, 10.0, 10.0], tile),
        BorderStyle::Dotted => draw_single_border(scene, rect, vec![10.0, 10.0], tile),
        BorderStyle::Double => draw_double_border(scene, rect, tile),
        BorderStyle::Groove => { unimplemented!() }
        BorderStyle::Ridge => { unimplemented!() }
        BorderStyle::Inset => { unimplemented!() }
        BorderStyle::Outset => { unimplemented!() }
        BorderStyle::Hidden => {
            // Don't display anything. But the border still takes up space. This is already
            // calculated in the box model by the layouter.
        }
    }
}

fn draw_single_border(scene: &mut vello::Scene, rect: &Rectangle, dashes: Vec<f64>, tile: &Tile) {
    let vello_shape = setup_rectangle_path(rect);
    let vello_brush = set_brush(&rect.border().brush(), rect.rect());
    let vello_stroke = kurbo::Stroke::new(rect.border().width() as f64).with_dashes(0.0, dashes);

    scene.stroke(
        &vello_stroke,
        Affine::translate((-tile.rect.x, -tile.rect.y)),
        &vello_brush,
        None,
        &vello_shape,
    );
}

fn draw_double_border(scene: &mut vello::Scene, rect: &Rectangle, tile: &Tile) {
    let vello_shape = setup_rectangle_path(rect);
    let vello_brush = set_brush(&rect.border().brush(), rect.rect());

    if rect.border().width() < 3.0 {
        // When the width is less than 3.0, we just draw a single line as there is no room for
        // a double border
        scene.stroke(
            &kurbo::Stroke::new(rect.border().width() as f64),
            Affine::translate((-tile.rect.x, -tile.rect.y)),
            &vello_brush,
            None,
            &vello_shape,
        );

        return;
    }

    // The formula: outer border: (N-1) / 2, 1px gap, inner border: (N-1) / 2

    // Outer border
    let width = (rect.border().width() / 2.0).floor();
    scene.stroke(
        &kurbo::Stroke::new(width as f64),
        Affine::IDENTITY,
        &vello_brush,
        None,
        &vello_shape,
    );

    let gap_size = 1.0;

    // inner border
    let inner_border_rect = Rect::new(
        rect.rect().x + width as f64 + gap_size,
        rect.rect().y + width as f64 + gap_size,
        rect.rect().width - width as f64 - gap_size,
        rect.rect().height - width as f64 - gap_size
    );
    scene.stroke(
        &kurbo::Stroke::new(width as f64),
        Affine::IDENTITY,
        &vello_brush,
        None,
        &inner_border_rect,
    );
}

enum ShapeEnum {
    Rect(Rect),
    RoundedRect(RoundedRect),
}

impl Shape for ShapeEnum {
    type PathElementsIter<'iter> = Box<dyn Iterator<Item = PathEl> + 'iter>;

    fn path_elements(&self, tolerance: f64) -> Self::PathElementsIter<'_> {
        match self {
            ShapeEnum::Rect(rect) => Box::new(rect.path_elements(tolerance)),
            ShapeEnum::RoundedRect(rounded_rect) => Box::new(rounded_rect.path_elements(tolerance)),
        }
    }

    fn area(&self) -> f64 {
        match self {
            ShapeEnum::Rect(rect) => rect.area(),
            ShapeEnum::RoundedRect(rounded_rect) => rounded_rect.area(),
        }
    }

    fn perimeter(&self, accuracy: f64) -> f64 {
        match self {
            ShapeEnum::Rect(rect) => rect.perimeter(accuracy),
            ShapeEnum::RoundedRect(rounded_rect) => rounded_rect.perimeter(accuracy),
        }
    }

    fn winding(&self, pt: Point) -> i32 {
        match self {
            ShapeEnum::Rect(rect) => rect.winding(pt),
            ShapeEnum::RoundedRect(rounded_rect) => rounded_rect.winding(pt),
        }
    }

    fn bounding_box(&self) -> Rect {
        match self {
            ShapeEnum::Rect(rect) => rect.bounding_box(),
            ShapeEnum::RoundedRect(rounded_rect) => rounded_rect.bounding_box(),
        }
    }
}

fn setup_rectangle_path(rect: &Rectangle) -> ShapeEnum {
    if rect.is_rounded() {
        let (r_tl, r_tr, r_br, r_bl) = rect.radius();

        return ShapeEnum::RoundedRect(RoundedRect::new(
            rect.rect().x,
            rect.rect().y,
            rect.rect().x + rect.rect().width,
            rect.rect().y + rect.rect().height,
            (r_tl, r_tr, r_br, r_bl)
        ))
    }

    ShapeEnum::Rect(Rect::new(
        rect.rect().x,
        rect.rect().y,
        rect.rect().x + rect.rect().width,
        rect.rect().y + rect.rect().height,
    ))
}