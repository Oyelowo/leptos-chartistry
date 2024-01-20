# Leptos Chartistry

An extensible charting library for Leptos.

## Review

- https://github.com/cxli233/FriendsDontLetFriends

## TODO

- Usecases that involve wrapping `<Chart>` in a Signal
- API: `Stack::push` has `mut Self -> Self` builder pattern.
- Panic if colour scheme with zero colours is used.
- Colours
    - Need a general write up on difficulties
    - Stacked colours iterate but should use sequential colours with "min step by" fn
    - Default stacked colours is not one colour

## Design experiments and notes:

- Series is not a `MaybeSignal` because it requires each Line to specify a key when it's a signal for use in <For>. To do this in a typesafe manner means bigger changes to the API however it doesn't make downstream code better. It still mostly ended up being wrapped in a signal -- just limited to the Series instead. It also complicates the internal logic that would probably make it harder to iterate on future designs. The library's API will probably change.

- Data is a `Vec<T>` to simplify building of line Series. To drop the <T> we could use an IntoIterator on the Series or each line. The has the drawback of complicating the chart internals as it still needs an efficient aggregate model. 
It's not clear that it could be efficient (avoiding extra iterations and copies of data) without impacting API ergonomics. For example, per line: `Iterator<Item = (X, Y)>`, per series: `Iterator<Item = (X, [Y])>` and `[Y] -> Y` per line which implies a generic Item = (X, T) and T -> Y. There are usecases for a data transform step but this looks better suited as a step before the data is passed to the library.
