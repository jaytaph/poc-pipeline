pub mod commands;

use std::fs::File;
use std::io::BufReader;
use std::ops::AddAssign;
use rand::Rng;
use crate::layering::layer::LayerList;
use crate::layouter::LayoutContext;
use crate::painter::commands::border::{Border, BorderStyle};
use crate::painter::commands::brush::Brush;
use crate::painter::commands::color::Color;
use crate::painter::commands::rectangle::Rectangle;
use crate::painter::commands::PaintCommand;
use crate::tiler::Tile;

pub struct Painter {}

impl Painter {
    // Generate paint commands for the given tile
    pub(crate) fn paint(tile: &Tile, layer_list: &LayerList) -> Vec<PaintCommand> {
        let mut commands = Vec::new();

        // @TODO: Since we might paint the element partially on this tile, we want to
        // make sure the painting is correct. If we have a bordered rectangle that spans two tiles,
        // we must make sure that this tile (the left side) does not have a border on the right side.
        // The next tile, should have no border on the left side so stitched together, they form a
        // single element.

        // I hope we can draw "outside" the texture surface.. So if the surface is 100x100, and we
        // want to draw a rectangle at 50x50 with a width of 100, it should draw a rectangle that is
        // 50x50 to 100x100. This way we can draw the border on the next tile as well, but we use a
        // negative offset.  This kind of clipping makes it easier to draw elements

        for tle in &tile.elements {
            let Some(element) = layer_list.layout_tree.get_node_by_id(tle.id) else {
                continue;
            };

            match &element.context {
                LayoutContext::Text(text) => {
                    // // @TODO No need to load them over and over again
                    // let font = Font::new("Arial", 24.0);
                    // let layout = Layout::new(&font, &text.layout);
                    // let brush = Brush::solid(Color::BLACK);
                    // let r = Rectangle::new(element.box_model.border_box()).with_background(brush);
                    // commands.push(PaintCommand::text(r, layout));
                }
                LayoutContext::Image(image) => {
                    // @TODO No need to load them over and over again. We need some image store of some kind
                    let file = File::open("sub.png").unwrap();
                    let reader = BufReader::new(file);
                    let img = image::load(reader, image::ImageFormat::from_path("sub.png").unwrap()).unwrap().to_rgba8();

                    let brush = Brush::image(img.as_raw().clone(), img.width(), img.height());
                    let border = Border::new(3.0, BorderStyle::Double, Brush::Solid(Color::BLACK));
                    let r = Rectangle::new(element.box_model.border_box()).with_background(brush).with_border(border);
                    commands.push(PaintCommand::rectangle(r));

                }
                LayoutContext::None => {
                    let c = Color::new(0.75, 0.0, 0.73, 0.15);
                    let brush = Brush::Solid(c);
                    let border = Border::new(1.0, BorderStyle::Double, Brush::Solid(Color::BLACK));
                    let r = Rectangle::new(element.box_model.border_box()).with_background(brush).with_border(border);
                    commands.push(PaintCommand::rectangle(r));
                }
            }
        }

        commands
    }
}
