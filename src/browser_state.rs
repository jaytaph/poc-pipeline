use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, OnceLock, RwLock};
use crate::geo::Rect;
use crate::layouter::LayoutElementId;
use crate::tiler::TileList;

/// Things that can change in the browser is stored in this structure. It keeps the current rendering pipeline (in the form of a layer_list),
/// and some things that we can control, or is controlled by the user (like current_hovered_element).
pub struct BrowserState {
    /// List of layers that will be visible are set to true
    pub visible_layer_list: Vec<bool>,
    /// If true, wireframes are drawn, otherwise complete elements are drawn
    pub wireframed: bool,
    /// Just show the hovered debug node in wireframe
    pub debug_hover: bool,
    /// Show the tile grid
    pub show_tilegrid: bool,
    /// When set, this is the element that is currently hovered upon
    pub current_hovered_element: Option<LayoutElementId>,
    /// LayerList that is currently being rendered
    pub tile_list: RwLock<TileList>,
    /// Current viewport offset + size
    pub viewport: Rect,

    pub _marker: PhantomData<*mut ()>,
}

// What could possibly go wrong??
unsafe impl Send for BrowserState {}
unsafe impl Sync for BrowserState {}


impl Debug for BrowserState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserState")
            .field("visible_layer_list", &self.visible_layer_list)
            .field("wireframed", &self.wireframed)
            .field("debug_hover", &self.debug_hover)
            .field("show_tilegrid", &self.show_tilegrid)
            .field("current_hovered_element", &self.current_hovered_element)
            .field("viewport", &self.viewport)
            .finish()
    }
}


static BROWSER_STATE: OnceLock<Arc<RwLock<BrowserState>>> = OnceLock::new();

pub fn init_browser_state(state: BrowserState) {
    BROWSER_STATE.set(Arc::new(RwLock::new(state))).expect("Failed to set browser state");
}

pub fn get_browser_state() -> Arc<RwLock<BrowserState>> {
    BROWSER_STATE.get().expect("Failed to get browser state").clone()
}