//! Pure message scheduling and widget interaction for native, self-drawn UI.
//!
//! Components return effect descriptions. The host owns I/O, clocks, workers,
//! fonts and rendering, and sends their completions back as ordinary messages.
mod component;
mod widget;

pub use component::{Component, Invalidation, Runtime, Update};
pub use widget::{Input, Interaction, Layout, Rect, Widget, WidgetId};
