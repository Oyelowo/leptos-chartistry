mod line;
mod stack;
mod use_data;

pub use line::{Line, Snippet, UseLine};
pub use stack::{Stack, STACK_COLOUR_SCHEME};
pub use use_data::{RenderData, UseData};

use crate::colours::{Colour, ColourScheme};
use leptos::signal_prelude::*;
use std::rc::Rc;

/// Arbitrary colours for a brighter palette than BATLOW
const SERIES_COLOUR_SCHEME: [Colour; 10] = [
    Colour::new(0x12, 0xA5, 0xED), // Blue
    Colour::new(0xF5, 0x32, 0x5B), // Red
    Colour::new(0x71, 0xc6, 0x14), // Green
    Colour::new(0xFF, 0x84, 0x00), // Orange
    Colour::new(0x7b, 0x4d, 0xff), // Purple
    Colour::new(0xdb, 0x4c, 0xb2), // Magenta
    Colour::new(0x92, 0xb4, 0x2c), // Darker green
    Colour::new(0xFF, 0xCA, 0x00), // Yellow
    Colour::new(0x22, 0xd2, 0xba), // Turquoise
    Colour::new(0xea, 0x60, 0xdf), // Pink
];

type GetX<T, X> = Rc<dyn Fn(&T) -> X>;
type GetY<T, Y> = Rc<dyn GetYValue<T, Y>>;

trait GetYValue<T, Y> {
    fn value(&self, t: &T) -> Y;
    fn cumulative_value(&self, t: &T) -> Y;
}

/// Describes how to render a series of data. A series is a collection of lines, bars, etc. that share the same X and Y axes.
///
/// ## Building a `Series`
///
/// You'll pass your chart a sequence of `T`. Each `T` should be a row from your results (e.g., from an API request) and should correspond to an `X` value (e.g., a timestamp) and one or more Y values (e.g., floats). For example `T`:
///
/// ```rust
/// // Defined somewhere in your code
/// pub struct Rate {
///     pub interval: Timestamp<Utc>,
///     pub in_octets: f64,
///     pub out_octets: f64,
/// }
/// ```
///
/// This `T` corresponds to `Rate`, `X` to `interval`, and `Y` to both `in_octets` and `out_octets`. The `Y` values can use `f64::NAN` to indicate missing data.
///
/// We then build up a `Series` to describe how to render this data. For example:
///
/// ```rust
/// let series = Series::new(|r: &Rate| r.interval)
///     .line(|r: &Rate| r.in_octets)
///     .line(|r: &Rate| r.out_octets);
/// ```
///
/// This is the simplest example and lacks details such as line names. Another more complete example is:
///
/// ```rust
/// let series = Series::new(|r: &Rate| r.interval)
///    .line(Line::new(|r: &Rate| r.in_octets).name("Rx"))
///    .line(Line::new(|r: &Rate| r.out_octets).name("Tx"));
/// ```
///
/// Another approach is to use a [Stack] to stack lines on top of each other.
///
/// ## Other options
///
/// The Series also allows you to control the range of the X and Y axes, and the colour scheme.
///
/// ## Full example
///
/// ```rust
/// TODO
/// ```
#[derive(Clone)]
pub struct Series<T: 'static, X: 'static, Y: 'static> {
    get_x: GetX<T, X>,
    lines: Vec<Rc<dyn ApplyUseSeries<T, Y>>>,
    /// Optional minimum X value. Extends the lower bound of the X axis if set.
    pub min_x: RwSignal<Option<X>>,
    /// Optional maximum X value. Extends the upper bound of the X axis if set.
    pub max_x: RwSignal<Option<X>>,
    /// Optional minimum Y value. Extends the lower bound of the Y axis if set.
    pub min_y: RwSignal<Option<Y>>,
    /// Optional maximum Y value. Extends the upper bound of the Y axis if set.
    pub max_y: RwSignal<Option<Y>>,
    /// Colour scheme for the series. If there are more lines than colours, the colours will repeat.
    pub colours: RwSignal<ColourScheme>,
}

trait ApplyUseSeries<T, Y> {
    fn apply_use_series(self: Rc<Self>, _: &mut SeriesAcc<T, Y>);
}

trait IntoUseLine<T, Y> {
    fn into_use_line(self, id: usize, colour: Memo<Colour>) -> (UseLine, GetY<T, Y>);
}

struct SeriesAcc<T, Y> {
    colour_id: usize,
    colours: RwSignal<ColourScheme>,
    lines: Vec<(UseLine, GetY<T, Y>)>,
}

impl<T, X, Y> Series<T, X, Y> {
    /// Create a new series. The `get_x` function is used to extract the X value from your struct.
    ///
    /// Intended to be a simple closure over your own data. For example `Series::new(|t: &MyType| t.x)`
    ///
    /// Next: add lines or stacks to the series with [Series::line] or [Series::stack].
    pub fn new(get_x: impl Fn(&T) -> X + 'static) -> Self {
        Self {
            get_x: Rc::new(get_x),
            min_x: RwSignal::default(),
            max_x: RwSignal::default(),
            min_y: RwSignal::default(),
            max_y: RwSignal::default(),
            colours: create_rw_signal(SERIES_COLOUR_SCHEME.into()),
            lines: Vec::new(),
        }
    }

    /// Set the colour scheme for the series. If there are more lines than colours, the colours will repeat.
    pub fn with_colours(self, colours: impl Into<ColourScheme>) -> Self {
        self.colours.set(colours.into());
        self
    }

    /// Set the minimum X value. Extends the lower bound of the X axis if set.
    pub fn with_min_x(self, max_x: impl Into<Option<X>>) -> Self {
        self.min_x.set(max_x.into());
        self
    }

    /// Set the maximum X value. Extends the upper bound of the X axis if set.
    pub fn with_max_x(self, max_x: impl Into<Option<X>>) -> Self {
        self.max_x.set(max_x.into());
        self
    }

    /// Set the minimum Y value. Extends the lower bound of the Y axis if set.
    pub fn with_min_y(self, min_y: impl Into<Option<Y>>) -> Self {
        self.min_y.set(min_y.into());
        self
    }

    /// Set the maximum Y value. Extends the upper bound of the Y axis if set.
    pub fn with_max_y(self, max_y: impl Into<Option<Y>>) -> Self {
        self.max_y.set(max_y.into());
        self
    }

    /// Set the X range. Extends the lower and upper bounds of the X axis if set.
    pub fn with_x_range(self, min_x: impl Into<Option<X>>, max_x: impl Into<Option<X>>) -> Self {
        self.with_min_x(min_x).with_max_x(max_x)
    }

    /// Set the Y range. Extends the lower and upper bounds of the Y axis if set.
    pub fn with_y_range(self, min_y: impl Into<Option<Y>>, max_y: impl Into<Option<Y>>) -> Self {
        self.with_min_y(min_y).with_max_y(max_y)
    }

    /// Adds a line to the series. See [Line] for more details.
    pub fn line(mut self, line: impl Into<Line<T, Y>>) -> Self {
        self.lines.push(Rc::new(line.into()));
        self
    }

    /// Adds multiple lines to the series at once. This is equivalent to calling [Series::line] multiple times.
    pub fn lines(mut self, lines: impl IntoIterator<Item = impl Into<Line<T, Y>>>) -> Self {
        for line in lines {
            self = self.line(line.into());
        }
        self
    }

    /// Gets the current size of the series (number of lines and stacks).
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns true if the series is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn to_use_lines(&self) -> Vec<(UseLine, GetY<T, Y>)> {
        let mut series = SeriesAcc::new(self.colours);
        for line in self.lines.clone() {
            line.apply_use_series(&mut series);
        }
        series.lines
    }
}

impl<T, X, Y: std::ops::Add<Output = Y>> Series<T, X, Y> {
    /// Adds a stack to the series. See [Stack] for more details.
    pub fn stack(mut self, stack: impl Into<Stack<T, Y>>) -> Self {
        self.lines.push(Rc::new(stack.into()));
        self
    }
}

impl<T, Y> SeriesAcc<T, Y> {
    fn new(colours: RwSignal<ColourScheme>) -> Self {
        Self {
            colour_id: 0,
            colours,
            lines: Vec::new(),
        }
    }

    fn next_colour(&mut self) -> Memo<Colour> {
        let id = self.colour_id;
        self.colour_id += 1;
        let colours = self.colours;
        create_memo(move |_| colours.get().by_index(id))
    }

    fn push(&mut self, colour: Memo<Colour>, line: impl IntoUseLine<T, Y>) -> GetY<T, Y> {
        // Create line
        let id = self.lines.len();
        let (line, get_y) = line.into_use_line(id, colour);
        // Insert line
        self.lines.push((line, get_y.clone()));
        get_y
    }
}
