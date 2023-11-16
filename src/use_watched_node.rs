use crate::{bounds::Bounds, projection::Projection};
use leptos::{html::Div, *};
use leptos_use::{
    use_mouse_with_options, use_resize_observer, UseMouseCoordType, UseMouseEventExtractorDefault,
    UseMouseOptions, UseMouseSourceType,
};

#[derive(Clone, Debug)]
pub struct UseWatchedNode {
    pub bounds: Signal<Option<Bounds>>,
    pub mouse_hover: Signal<bool>,
    pub mouse_abs: Signal<(f64, f64)>,
    pub mouse_rel: Signal<(f64, f64)>,
}

fn scroll_position() -> (f64, f64) {
    let window = window();
    let x = window.scroll_x().unwrap_or_default();
    let y = window.scroll_y().unwrap_or_default();
    (x, y)
}

pub fn use_watched_node(node: NodeRef<Div>) -> UseWatchedNode {
    // SVG bounds -- dimensions for our root <svg> element inside the document
    let (bounds, set_bounds) = create_signal::<Option<Bounds>>(None);
    use_resize_observer(node, move |entries, _| {
        let rect = &entries[0].target().get_bounding_client_rect();
        let rect = Bounds::new(rect.width(), rect.height());
        log::info!("set_bounds: {:?}", rect);
        set_bounds.set(Some(rect))
    });
    let bounds: Signal<Option<Bounds>> = bounds.into();

    // Mouse position
    let mouse = use_mouse_with_options(
        UseMouseOptions::default()
            .target(node)
            .coord_type(UseMouseCoordType::<UseMouseEventExtractorDefault>::Page)
            .reset_on_touch_ends(true),
    );

    // Mouse absolute coords on page
    let mouse_abs = Signal::derive(move || {
        let x = mouse.x.get();
        let y = mouse.y.get();
        (x, y)
    });

    // Mouse relative to SVG
    let mouse_rel: Signal<_> = create_memo(move |_| {
        let (x, y) = mouse_abs.get();
        let (left, top) = node
            .get()
            .map(|target| {
                let rect = target.get_bounding_client_rect();
                (rect.left(), rect.top())
            })
            .unwrap_or_default();
        let x = x - left;
        let y = y - top;
        (x, y)
    })
    .into();

    // Mouse inside SVG?
    let mouse_hover = create_memo(move |_| {
        let (x, y) = mouse_rel.get();
        log::info!("mouse_hover: {:?} -> {:?}", (x, y), bounds.get());
        mouse.source_type.get() != UseMouseSourceType::Unset
            && bounds
                .get()
                .map(|bounds| bounds.contains(x, y))
                .unwrap_or(false)
    })
    .into();

    UseWatchedNode {
        bounds,
        mouse_hover,
        mouse_abs,
        mouse_rel,
    }
}

impl UseWatchedNode {
    // Mouse inside inner chart?
    pub fn mouse_hover_inner(&self, proj: Signal<Projection>) -> Signal<bool> {
        let (mouse_rel, hover) = (self.mouse_rel, self.mouse_hover);
        create_memo(move |_| {
            let (x, y) = mouse_rel.get();
            hover.get() && proj.with(|proj| proj.bounds().contains(x, y))
        })
        .into()
    }
}
