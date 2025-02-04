use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::render_tree::{RenderTree, RenderNodeId};
use taffy::prelude::*;
use crate::document::node::{NodeType, NodeId as DomNodeId};
use crate::document::style::{StyleValue, Unit};
use crate::layouter::{boxmodel as BoxModel, LayoutElementNode, LayoutTree, TaffyStruct, TaffyNodeId, LayoutElementId};
use crate::layouter::text::measure_text_height;
use crate::layouter::ViewportSize;

/// Generates a layout tree based on taffy. Note that the layout tree current holds taffy information (like styles)
/// that we probably want to convert to our own style system. We already do this with the taffy layout through the
/// BoxModel structure.
pub fn generate_with_taffy(render_tree: RenderTree, viewport: ViewportSize) -> LayoutTree {
    let root_id = render_tree.root_id.unwrap();
    let Some(mut layout_tree) = generate_tree(render_tree, root_id) else {
        panic!("Failed to generate root node render tree");
    };

    // Compute the layout based on the viewport
    layout_tree.taffy.tree.compute_layout(layout_tree.taffy.root_id, Size {
        width: AvailableSpace::Definite(viewport.width as f32),
        height: AvailableSpace::Definite(viewport.height as f32),
    }).unwrap();

    fn generate_boxmodel(layout_tree: &mut LayoutTree, node_id: LayoutElementId, offset: (f32, f32)) {
        let el = layout_tree.get_node_by_id(node_id).unwrap();
        let layout = layout_tree.taffy.tree.layout(el.taffy_node_id).unwrap().clone();

        let el = layout_tree.get_node_by_id_mut(node_id).unwrap();
        el.box_model = to_boxmodel(&layout, offset);
        let child_ids = el.children.clone();

        for child_id in child_ids {
            generate_boxmodel(layout_tree, child_id, (
                offset.0 + layout.location.x + layout.margin.left,
                offset.1 + layout.location.y + layout.margin.top
            ));
        }
    }

    // Generate box model for the whole layout tree
    let root_id = layout_tree.root_id;
    generate_boxmodel(&mut layout_tree, root_id, (0.0, 0.0));

    layout_tree
}

// Returns true if there is a margin on the rect (basically, if the rect is non-zero)
fn has_margin(src: Rect<LengthPercentageAuto>) -> bool {
    let is_zero = (src.top == LengthPercentageAuto::Length(0.0) || src.top == LengthPercentageAuto::Percent(0.0)) &&
    (src.right == LengthPercentageAuto::Length(0.0) || src.right == LengthPercentageAuto::Percent(0.0)) &&
    (src.bottom == LengthPercentageAuto::Length(0.0) || src.bottom == LengthPercentageAuto::Percent(0.0)) &&
    (src.left == LengthPercentageAuto::Length(0.0) || src.left == LengthPercentageAuto::Percent(0.0));

    !is_zero
}


fn generate_tree(
    render_tree: RenderTree,
    root_id: RenderNodeId,
) -> Option<LayoutTree> {
    let mut tree: TaffyTree<()> = TaffyTree::new();

    let mut layout_tree = LayoutTree {
        render_tree,
        taffy: TaffyStruct {
            tree,
            root_id: TaffyNodeId::new(0), // Will be filled in later
        },
        arena: HashMap::new(),
        root_id: LayoutElementId::new(0), // Will be filled in layer
        next_node_id: Rc::new(RefCell::new(LayoutElementId::new(0))),
    };

    let ids = {
        let temp_el = generate_node(&mut layout_tree, root_id).unwrap();
        (temp_el.taffy_node_id, temp_el.id)
    };

    layout_tree.taffy.root_id = ids.0;
    layout_tree.root_id = ids.1;

    Some(layout_tree)
}

fn generate_node(
    layout_tree: &mut LayoutTree,
    render_node_id: RenderNodeId,
) -> Option<&LayoutElementNode> {
    let mut style = Style {
        display: Display::Block,
        ..Default::default()
    };

    // Find the DOM node in the DOM document that is wrapped in the render tree
    let dom_node_id = DomNodeId::from(render_node_id);   // DOM node IDs and render node IDs are interchangeable
    let Some(dom_node) = layout_tree.render_tree.doc.get_node_by_id(dom_node_id) else {
        return None;
    };

    match &dom_node.node_type {
        NodeType::Element(data) => {
            // --- Width and Height styles ---
            if let Some(width) = data.get_style("width") {
                match width {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.size.width = Dimension::Length(*value),
                            Unit::Percent => style.size.width = Dimension::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(height) = data.get_style("height") {
                match height {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.size.height = Dimension::Length(*value),
                            Unit::Percent => style.size.height = Dimension::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // --- Margin ---
            if let Some(margin_block_start) = data.get_style("margin-top") {
                match margin_block_start {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.margin.top = LengthPercentageAuto::Length(*value),
                            Unit::Percent => style.margin.top = LengthPercentageAuto::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(margin_block_end) = data.get_style("margin-bottom") {
                match margin_block_end {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.margin.bottom = LengthPercentageAuto::Length(*value),
                            Unit::Percent => style.margin.bottom = LengthPercentageAuto::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(margin_inline_start) = data.get_style("margin-left") {
                match margin_inline_start {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.margin.left = LengthPercentageAuto::Length(*value),
                            Unit::Percent => style.margin.left = LengthPercentageAuto::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(margin_inline_end) = data.get_style("margin-right") {
                match margin_inline_end {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.margin.right = LengthPercentageAuto::Length(*value),
                            Unit::Percent => style.margin.right = LengthPercentageAuto::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            // --- Padding ---
            if let Some(padding_block_start) = data.get_style("padding-top") {
                match padding_block_start {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.padding.top = LengthPercentage::Length(*value),
                            Unit::Percent => style.padding.top = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(padding_block_end) = data.get_style("padding-bottom") {
                match padding_block_end {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.padding.bottom = LengthPercentage::Length(*value),
                            Unit::Percent => style.padding.bottom = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(padding_inline_start) = data.get_style("padding-left") {
                match padding_inline_start {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.padding.left = LengthPercentage::Length(*value),
                            Unit::Percent => style.padding.left = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(padding_inline_end) = data.get_style("padding-right") {
                match padding_inline_end {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.padding.right = LengthPercentage::Length(*value),
                            Unit::Percent => style.padding.right = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            // --- Border ---
            if let Some(border_top_width) = data.get_style("border-top-width") {
                match border_top_width {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.border.top = LengthPercentage::Length(*value),
                            Unit::Percent => style.border.top = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(border_bottom_width) = data.get_style("border-bottom-width") {
                match border_bottom_width {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.border.bottom = LengthPercentage::Length(*value),
                            Unit::Percent => style.border.bottom = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(border_left_width) = data.get_style("border-left-width") {
                match border_left_width {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.border.left = LengthPercentage::Length(*value),
                            Unit::Percent => style.border.left = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if let Some(border_right_width) = data.get_style("border-right-width") {
                match border_right_width {
                    StyleValue::Unit(value, unit) => {
                        match unit {
                            Unit::Px => style.border.right = LengthPercentage::Length(*value),
                            Unit::Percent => style.border.right = LengthPercentage::Percent(*value),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        NodeType::Text(text) => {
            let font_size = 32.0;
            let line_height = 36.0;
            style.size.height = Dimension::Length(
                measure_text_height(text, font_size, line_height) as f32
            );

        }
    }

    if dom_node.children.is_empty() {
        match layout_tree.taffy.tree.new_leaf(style) {
            Ok(leaf_id) => {
                let el = LayoutElementNode {
                    id: layout_tree.next_node_id(),
                    dom_node_id,
                    render_node_id,
                    taffy_node_id: leaf_id,
                    box_model: BoxModel::BoxModel::ZERO,
                    children: vec![],
                };

                let id = el.id;
                layout_tree.arena.insert(id, el);
                return layout_tree.arena.get(&id);
            },
            Err(_) => {},
        }

        return None
    }

    let mut children_taffy_ids = Vec::new();
    let mut children_el_ids = Vec::new();

    let render_node = layout_tree.render_tree.get_node_by_id(render_node_id).unwrap();
    let children = render_node.children.clone();

    for child_render_node_id in &children {
        match generate_node(layout_tree, *child_render_node_id) {
            Some(el) => {
                children_taffy_ids.push(el.taffy_node_id);
                children_el_ids.push(el.id);
            },
            None => continue,
        }
    }

    match layout_tree.taffy.tree.new_with_children(style, &children_taffy_ids) {
        Ok(leaf_id) => {
            let el = LayoutElementNode {
                id: layout_tree.next_node_id(),
                dom_node_id,
                render_node_id,
                taffy_node_id: leaf_id,
                box_model: BoxModel::BoxModel::ZERO,
                children: children_el_ids,
            };

            let id = el.id;
            layout_tree.arena.insert(id, el);
            layout_tree.arena.get(&id)
        }
        Err(_) => None,
    }
}

/// Converts a taffy layout to our own BoxModel structure
pub fn to_boxmodel(layout: &Layout, offset: (f32, f32)) -> BoxModel::BoxModel {
    BoxModel::BoxModel {
        margin_box: BoxModel::Rect {
            x: offset.0 as f64 + layout.location.x as f64,
            y: offset.1  as f64 + layout.location.y as f64,
            width: layout.size.width as f64 + layout.margin.left as f64 + layout.margin.right as f64,
            height: layout.size.height as f64 + layout.margin.top as f64 + layout.margin.bottom as f64,
        },
        padding: BoxModel::Edges {
            top: layout.padding.top as f64,
            right: layout.padding.right as f64,
            bottom: layout.padding.bottom as f64,
            left: layout.padding.left as f64,
        },
        border: BoxModel::Edges {
            top: layout.border.top as f64,
            right: layout.border.right as f64,
            bottom: layout.border.bottom as f64,
            left: layout.border.left as f64,
        },
        margin: BoxModel::Edges {
            top: layout.margin.top as f64,
            right: layout.margin.right as f64,
            bottom: layout.margin.bottom as f64,
            left: layout.margin.left as f64,
        }
    }
}
