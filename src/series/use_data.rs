use crate::{bounds::Bounds, series::UseLine, state::State, Series};
use chrono::prelude::*;
use leptos::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct UseData<X: 'static, Y: 'static> {
    pub series: Memo<Vec<UseLine>>,

    pub data_x: Memo<Vec<X>>,
    data_y: Memo<Vec<HashMap<usize, Y>>>,

    pub range_x: Memo<Option<(X, X)>>,
    /// Yields the min / max Y values. Still returns a range if min / max are set and no data.
    pub range_y: Memo<Option<(Y, Y)>>,

    pub positions_x: Memo<Vec<f64>>,
    positions_y: Memo<Vec<HashMap<usize, f64>>>,
    pub position_range: Memo<Bounds>,
}

impl<X: Clone + PartialEq + 'static, Y: Clone + PartialEq + 'static> UseData<X, Y> {
    pub fn new<T: 'static>(series: Series<T, X, Y>, data: Signal<Vec<T>>) -> UseData<X, Y>
    where
        X: PartialOrd + Position,
        Y: PartialOrd + Position,
    {
        let lines = series.to_lines();
        let Series {
            get_x,
            min_x,
            max_x,
            min_y,
            max_y,
            ..
        } = series;

        // Sort series by name
        let series = {
            let (lines, _): (Vec<_>, Vec<_>) = lines.clone().into_iter().unzip();
            create_memo(move |_| {
                let mut lines = lines.clone();
                lines.sort_by_key(|line| line.name.get());
                lines
            })
        };

        // Data signals
        let data_x = create_memo(move |_| {
            data.with(|data| data.iter().map(|datum| (get_x)(datum)).collect::<Vec<_>>())
        });
        let y_maker = |which: bool| {
            let lines = lines.clone();
            create_memo(move |_| {
                data.with(|data| {
                    data.iter()
                        .map(|datum| {
                            lines
                                .iter()
                                .map(|(line, get_y)| {
                                    let y = if which {
                                        get_y.value(datum)
                                    } else {
                                        get_y.cumulative_value(datum)
                                    };
                                    (line.id, y)
                                })
                                .collect::<HashMap<_, _>>()
                        })
                        .collect::<Vec<_>>()
                })
            })
        };
        // Generate two sets of Ys: original and cumulative value. They can differ when stacked
        let data_y = y_maker(true);
        let data_y_cumulative = y_maker(false);

        // Position signals
        let positions_x = create_memo(move |_| {
            data_x.with(move |data_x| data_x.iter().map(|x| x.position()).collect::<Vec<_>>())
        });
        let positions_y = create_memo(move |_| {
            data_y_cumulative
                .get()
                .into_iter()
                .map(|ys| {
                    ys.into_iter()
                        .map(|(id, y)| (id, y.position()))
                        .collect::<HashMap<_, _>>()
                })
                .collect::<Vec<_>>()
        });

        // Range signals
        let range_x: Memo<Option<(X, X)>> = create_memo(move |_| {
            let range: Option<(X, X)> =
                with!(|positions_x, data_x| Self::data_range(positions_x, data_x));

            // Expand specified range to single Option
            let specified: Option<(X, X)> = match (min_x.get(), max_x.get()) {
                (Some(min_x), Some(max_x)) => Some((min_x.clone(), max_x.clone())),
                (Some(min_x), None) => Some((min_x.clone(), min_x.clone())),
                (None, Some(max_x)) => Some((max_x.clone(), max_x.clone())),
                (None, None) => None,
            };

            // Extend range by specified?
            match (range, specified) {
                (None, None) => None, // No data, no range

                // One of range or specified
                (Some(range), None) => Some(range),
                (None, Some(specified)) => Some(specified),

                // Calculate min / max of range and specified
                (Some((min_r, max_r)), Some((min_s, max_s))) => Some((
                    if min_r.position() < min_s.position() {
                        min_r
                    } else {
                        min_s
                    },
                    if max_r.position() > max_s.position() {
                        max_r
                    } else {
                        max_s
                    },
                )),
            }
        });

        // TODO: consider trying to minimise iterations over data
        let range_y = create_memo(move |_| {
            data_y_cumulative
                .get()
                .into_iter()
                .flat_map(|ys| ys.into_values())
                .chain(min_y.get())
                .chain(max_y.get())
                .map(|y| {
                    let pos = y.position();
                    (y, pos)
                })
                .fold(None, |acc, y @ (_, pos)| {
                    // Note this logic is duplicated in data_range
                    if pos.is_finite() {
                        acc.map(|(min, max): ((Y, f64), (Y, f64))| {
                            (
                                if pos < min.1 { y.clone() } else { min },
                                if pos > max.1 { y.clone() } else { max },
                            )
                        })
                        .or(Some((y.clone(), y)))
                    } else {
                        acc
                    }
                })
                .map(|((min, _), (max, _))| (min, max))
        });

        // Position range signal
        let position_range = create_memo(move |_| {
            let (min_x, max_x) = range_x
                .get()
                .map(|(min, max)| (min.position(), max.position()))
                .unwrap_or_default();
            let (min_y, max_y) = range_y
                .get()
                .map(|(min, max)| (min.position(), max.position()))
                .unwrap_or_default();
            Bounds::from_points(min_x, min_y, max_x, max_y)
        });

        UseData {
            series,
            data_x,
            data_y,
            range_x,
            range_y,
            positions_x,
            positions_y,
            position_range,
        }
    }

    /// Given a list of positions. Finds the min / max indexes using is_finite to skip infinite and NaNs. Returns the data values at those indexes. Returns `None` if no data.
    fn data_range<V: Clone + PartialOrd>(positions: &[f64], data: &[V]) -> Option<(V, V)> {
        // Find min / max indexes in positions
        let indexes = positions.iter().enumerate().fold(None, |acc, (i, &pos)| {
            if pos.is_finite() {
                acc.map(|(min, max)| {
                    (
                        if pos < positions[min] { i } else { min },
                        if pos > positions[max] { i } else { max },
                    )
                })
                .or(Some((i, i)))
            } else {
                acc
            }
        });
        // Return data values
        indexes.map(|(min, max)| (data[min].clone(), data[max].clone()))
    }
}

impl<X: 'static, Y: 'static> UseData<X, Y> {
    fn nearest_index(&self, pos_x: Signal<f64>) -> Signal<Option<usize>> {
        let positions_x = self.positions_x;
        Signal::derive(move || {
            positions_x.with(move |positions_x| {
                // No values
                if positions_x.is_empty() {
                    return None;
                }
                // Find index after pos
                let pos_x = pos_x.get();
                let index = positions_x.partition_point(|&v| v < pos_x);
                // No value before
                if index == 0 {
                    return Some(0);
                }
                // No value ahead
                if index == positions_x.len() {
                    return Some(index - 1);
                }
                // Find closest index
                let ahead = positions_x[index] - pos_x;
                let before = pos_x - positions_x[index - 1];
                if ahead < before {
                    Some(index)
                } else {
                    Some(index - 1)
                }
            })
        })
    }

    pub fn nearest_data_x(&self, pos_x: Signal<f64>) -> Memo<Option<X>>
    where
        X: Clone + PartialEq,
    {
        let data_x = self.data_x;
        let index = self.nearest_index(pos_x);
        create_memo(move |_| {
            index
                .get()
                .map(|index| with!(|data_x| data_x[index].clone()))
        })
    }

    /// Given an arbitrary (unaligned to data) X position, find the nearest X position aligned to data. Returns `f64::NAN` if no data.
    pub fn nearest_position_x(&self, pos_x: Signal<f64>) -> Memo<f64> {
        let positions_x = self.positions_x;
        let index = self.nearest_index(pos_x);
        create_memo(move |_| {
            index
                .get()
                .map(|index| with!(|positions_x| positions_x[index]))
                .unwrap_or(f64::NAN)
        })
    }

    pub fn nearest_data_y(&self, pos_x: Signal<f64>) -> Memo<Vec<(UseLine, Option<Y>)>>
    where
        Y: Clone + PartialEq,
    {
        let series = self.series;
        let data_y = self.data_y;
        let index_x = self.nearest_index(pos_x);
        create_memo(move |_| {
            let index_x = index_x.get();
            series
                .get()
                .into_iter()
                .map(|line| {
                    let y_value = index_x
                        .and_then(|index_x| with!(|data_y| data_y[index_x].get(&line.id).cloned()));
                    (line, y_value)
                })
                .collect::<Vec<_>>()
        })
    }
}

pub trait Position {
    fn position(&self) -> f64;
}

impl Position for f64 {
    fn position(&self) -> f64 {
        *self
    }
}

impl<Tz: TimeZone> Position for DateTime<Tz> {
    fn position(&self) -> f64 {
        self.timestamp() as f64 + (self.timestamp_subsec_nanos() as f64 / 1e9)
    }
}

#[component]
pub fn RenderData<X: Clone + 'static, Y: Clone + 'static>(state: State<X, Y>) -> impl IntoView {
    let data = state.pre.data;
    let pos_x = data.positions_x;
    let pos_y = data.positions_y;
    let proj = state.projection;
    let mk_svg_coords = move |id| {
        Signal::derive(move || {
            let proj = proj.get();
            pos_x
                .get()
                .into_iter()
                .enumerate()
                .map(|(i, x)| {
                    // TODO: our data model guarantees unwrap always succeeds but this doesn't hold true if we move to separated data iterators
                    let y = pos_y.with(|pos_y| *pos_y[i].get(&id).unwrap());
                    proj.position_to_svg(x, y)
                })
                .collect::<Vec<_>>()
        })
    };

    view! {
        <g class="_chartistry_series">
            <For
                each=move || data.series.get()
                key=|line| line.id
                children=move |line| line.render(mk_svg_coords(line.id))
            />
        </g>
    }
}
