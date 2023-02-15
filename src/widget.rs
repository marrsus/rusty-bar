use anyhow::Result;
use futures::stream::Stream;
use std::pin::Pin;

/// The stream of `Vec<Text>` returned by each widget.
///
/// This simple type alias makes referring to this stream a little easier. For
/// more information on the stream (and how widgets are structured), please
/// refer to the documentation on the [`Widget`] trait.
///
/// Any errors on the stream are logged but do not affect the runtime of the
/// main [`crate::Cnx`] instance.
///
pub type WidgetStream = Pin<Box<dyn Stream<Item = Result<Vec<Text>>>>>;

/// The main trait implemented by all widgets.
///
/// This simple trait defines a widget. A widget is essentially just a
/// [`futures::stream::Stream`] and this trait is the standard way of accessing
/// that stream.
///
/// See the [`WidgetStream`] type alias for the exact type of stream that
/// should be returned.
///
pub trait Widget {
    fn into_stream(self: Box<Self>) -> Result<WidgetStream>;
}



use tokio::runtime::Runtime;
use tokio::task;
use tokio_stream::{StreamExt, StreamMap};

use crate::bar::{Bar,Offset,Position};
use crate::xcb::XcbEventStream;
use crate::text::Text;


/// The main object, used to instantiate an instance of Cnx.
///
/// Widgets can be added using the [`add_widget()`] method. Once configured,
/// the [`run()`] method will take ownership of the instance and run it until
/// the process is killed or an error occurs.
///
/// [`add_widget()`]: #method.add_widget
/// [`run()`]: #method.run
pub struct Cnx {
    /// The position of the Cnx bar
    position: Position,
    /// The list of widgets attached to the Cnx bar
    widgets: Vec<Box<dyn Widget>>,
    /// The (x,y) offset of the bar
    /// It can be used in order to run multiple bars in a multi-monitor setup
    offset: Offset,
    /// The (optional) width of the bar
    /// It can be used in order to run multiple bars in a multi-monitor setup
    width: Option<u16>,
}

impl Cnx {
    /// Creates a new `Cnx` instance.
    ///
    /// This creates a new `Cnx` instance at either the top or bottom of the
    /// screen, depending on the value of the [`Position`] enum.
    ///
    /// [`Position`]: enum.Position.html
    pub fn new(position: Position) -> Self {
        let widgets = Vec::new();
        Self {
            position,
            widgets,
            offset: Offset::default(),
            width: None,
        }
    }

    /// Returns a new instance of `Cnx` with the specified width.
    ///
    /// This allows to specify the width of the `Cnx` bar,
    /// which can be used with the [`with_offset()`] method
    /// in order to have a multiple bar setup.
    ///
    /// [`with_offset()`]: #method.with_offset
    pub fn with_width(self, width: Option<u16>) -> Self {
        Self { width, ..self }
    }

    /// Returns a new instance of `Cnx` with the specified offset.
    ///
    /// This allows to specify the x and y offset of the `Cnx` bar,
    /// which can be used with the [`with_width()`] method
    /// in order to have a multiple bar setup.
    ///
    /// [`with_width()`]: #method.with_width
    pub fn with_offset(self, x: i16, y: i16) -> Self {
        Self {
            offset: Offset { x, y },
            ..self
        }
    }

    /// Adds a widget to the `Cnx` instance.
    ///
    /// Takes ownership of the [`Widget`] and adds it to the Cnx instance to
    /// the right of any existing widgets.
    ///
    /// [`Widget`]: widgets/trait.Widget.html
    pub fn add_widget<W>(&mut self, widget: W)
    where
        W: Widget + 'static,
    {
        self.widgets.push(Box::new(widget));
    }

    /// Runs the Cnx instance.
    ///
    /// This method takes ownership of the Cnx instance and runs it until either
    /// the process is terminated, or an internal error is returned.
    pub fn run(self) -> Result<()> {
        // Use a single-threaded event loop. We aren't interested in
        // performance too much, so don't mind if we block the loop
        // occasionally. We are using events to get woken up as
        // infrequently as possible (to save battery).
        let rt = Runtime::new()?;
        let local = task::LocalSet::new();
        local.block_on(&rt, self.run_inner())?;
        Ok(())
    }

    async fn run_inner(self) -> Result<()> {
        let mut bar = Bar::new(self.position, self.width, self.offset)?;

        let mut widgets = StreamMap::with_capacity(self.widgets.len());
        for widget in self.widgets {
            let idx = bar.add_content(Vec::new())?;
            widgets.insert(idx, widget.into_stream()?);
        }

        let mut event_stream = XcbEventStream::new(bar.connection().clone())?;
        task::spawn_local(async move {
            loop {
                tokio::select! {
                    // Pass each XCB event to the Bar.
                    Some(event) = event_stream.next() => {
                        if let Err(err) = bar.process_event(event) {
                            println!("Error processing XCB event: {err}");
                        }
                    },

                    // Each time a widget yields new values, pass to the bar.
                    // Ignore (but log) any errors from widgets.
                    Some((idx, result)) = widgets.next() => {
                        match result {
                            Err(err) => println!("Error from widget {idx}: {err}"),
                            Ok(texts) => {
                                if let Err(err) = bar.update_content(idx, texts) {
                                    println!("Error updating widget {idx}: {err}");
                                }
                            }
                        }
                    }
                }
            }
        })
        .await?;

        Ok(())
    }
}
