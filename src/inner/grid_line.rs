use super::{InnerLayout, InnerOption, UseInner};
use crate::{
    chart::Attr,
    colours::{Colour, LIGHTER_GREY},
    debug::DebugRect,
    projection::Projection,
    series::UseSeries,
    ticks::{short_format_fn, Ticks, UseTicks},
    use_watched_node::UseWatchedNode,
    TickLabels,
};
use leptos::*;
use std::{borrow::Borrow, rc::Rc};

#[derive(Clone)]
pub struct GridLine<Tick: Clone> {
    width: MaybeSignal<f64>,
    colour: MaybeSignal<Colour>,
    ticks: TickLabels<Tick>,
}

#[derive(Clone)]
pub struct HorizontalGridLine<X: Clone>(GridLine<X>);
#[derive(Clone)]
pub struct VerticalGridLine<Y: Clone>(GridLine<Y>);

#[derive(Clone)]
struct GridLineAttr<Tick> {
    width: MaybeSignal<f64>,
    colour: MaybeSignal<Colour>,
    ticks: Ticks<Tick>,
}

#[derive(Clone)]
struct HorizontalGridLineAttr<X>(GridLineAttr<X>);
#[derive(Clone)]
struct VerticalGridLineAttr<Y>(GridLineAttr<Y>);

#[derive(Clone)]
struct UseGridLine<Tick: 'static> {
    width: MaybeSignal<f64>,
    colour: MaybeSignal<Colour>,
    ticks: UseTicks<Tick>,
}

#[derive(Clone)]
struct UseHorizontalGridLine<X: 'static>(UseGridLine<X>);
#[derive(Clone)]
struct UseVerticalGridLine<Y: 'static>(UseGridLine<Y>);

impl<Tick: Clone> GridLine<Tick> {
    fn new(ticks: impl Borrow<TickLabels<Tick>>) -> Self {
        Self {
            width: 1.0.into(),
            colour: Into::<Colour>::into(LIGHTER_GREY).into(),
            ticks: ticks.borrow().clone(),
        }
    }

    /// Vertical grid lines running parallel to the y-axis. These run from top to bottom at each tick.
    pub fn vertical(ticks: impl Borrow<TickLabels<Tick>>) -> HorizontalGridLine<Tick> {
        HorizontalGridLine(Self::new(ticks))
    }
    /// Horizontal grid lines running parallel to the x-axis. These run from left to right at each tick.
    pub fn horizontal(ticks: impl Borrow<TickLabels<Tick>>) -> VerticalGridLine<Tick> {
        VerticalGridLine(Self::new(ticks))
    }

    fn apply_attr(self, attr: &Attr) -> GridLineAttr<Tick> {
        GridLineAttr {
            width: self.width,
            colour: self.colour,
            ticks: self.ticks.apply_attr(attr, short_format_fn()),
        }
    }
}

impl<X: Clone + PartialEq + 'static, Y: 'static> InnerLayout<X, Y> for HorizontalGridLine<X> {
    fn apply_attr(self, attr: &Attr) -> Rc<dyn InnerOption<X, Y>> {
        Rc::new(HorizontalGridLineAttr(self.0.apply_attr(attr)))
    }
}

impl<X: 'static, Y: Clone + PartialEq + 'static> InnerLayout<X, Y> for VerticalGridLine<Y> {
    fn apply_attr(self, attr: &Attr) -> Rc<dyn InnerOption<X, Y>> {
        Rc::new(VerticalGridLineAttr(self.0.apply_attr(attr)))
    }
}

impl<X: Clone + PartialEq, Y> InnerOption<X, Y> for HorizontalGridLineAttr<X> {
    fn to_use(
        self: Rc<Self>,
        series: &UseSeries<X, Y>,
        proj: Signal<Projection>,
    ) -> Box<dyn UseInner> {
        let avail_width = Projection::derive_width(proj);
        Box::new(UseHorizontalGridLine(UseGridLine {
            width: self.0.width,
            colour: self.0.colour,
            ticks: self.0.ticks.clone().generate_x(series.data, avail_width),
        }))
    }
}

impl<X, Y: Clone + PartialEq> InnerOption<X, Y> for VerticalGridLineAttr<Y> {
    fn to_use(
        self: Rc<Self>,
        series: &UseSeries<X, Y>,
        proj: Signal<Projection>,
    ) -> Box<dyn UseInner> {
        let avail_height = Projection::derive_height(proj);
        Box::new(UseVerticalGridLine(UseGridLine {
            width: self.0.width,
            colour: self.0.colour,
            ticks: self.0.ticks.clone().generate_y(series.data, avail_height),
        }))
    }
}

impl<X> UseInner for UseHorizontalGridLine<X> {
    fn render(self: Box<Self>, proj: Signal<Projection>, _: &UseWatchedNode) -> View {
        view! {
            <ViewHorizontalGridLine line=self.0 projection=proj />
        }
    }
}

impl<X> UseInner for UseVerticalGridLine<X> {
    fn render(self: Box<Self>, proj: Signal<Projection>, _: &UseWatchedNode) -> View {
        view! {
            <ViewVerticalGridLine line=self.0 projection=proj />
        }
    }
}

#[component]
fn ViewHorizontalGridLine<X: 'static>(
    line: UseGridLine<X>,
    projection: Signal<Projection>,
) -> impl IntoView {
    let ticks = Signal::derive(move || {
        let ticks = line.ticks.ticks; // Ticky ticky tick tick
        with!(|ticks, projection| {
            (ticks.ticks.iter())
                .map(|tick| {
                    let x = ticks.state.position(tick);
                    let x = projection.data_to_svg(x, 0.0).0;
                    let bounds = projection.bounds();
                    view! {
                        <line
                            x1=x
                            y1=move || bounds.top_y()
                            x2=x
                            y2=move || bounds.bottom_y()
                            stroke=move || line.colour.get().to_string()
                            stroke-width=line.width />
                    }
                })
                .collect_view()
        })
    });
    view! {
        <g class="_chartistry_grid_line_x">
            <DebugRect label="XGridLine" debug=line.ticks.debug />
            {ticks}
        </g>
    }
}

#[component]
fn ViewVerticalGridLine<Y: 'static>(
    line: UseGridLine<Y>,
    projection: Signal<Projection>,
) -> impl IntoView {
    let ticks = Signal::derive(move || {
        let ticks = line.ticks.ticks;
        with!(|ticks, projection| {
            (ticks.ticks.iter())
                .map(|tick| {
                    let y = ticks.state.position(tick);
                    let y = projection.data_to_svg(0.0, y).1;
                    let bounds = projection.bounds();
                    view! {
                        <line
                            x1=move || bounds.left_x()
                            y1=y
                            x2=move || bounds.right_x()
                            y2=y
                            stroke=move || line.colour.get().to_string()
                            stroke-width=line.width />
                    }
                })
                .collect_view()
        })
    });
    view! {
        <g class="_chartistry_grid_line_y">
            <DebugRect label="YGridLine" debug=line.ticks.debug />
            {ticks}
        </g>
    }
}
