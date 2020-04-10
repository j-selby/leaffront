//! An intermediate-mode UI layout and rendering API for Leaffront - designed to not be too complex
//! for designing layouts.

extern crate leaffront_core;

use leaffront_core::pos::{Position, Rect};
use leaffront_core::render::color::Color;
use leaffront_core::render::font::FontCache;
use leaffront_core::render::Drawer;
use std::fmt::Display;
use std::marker::{PhantomData, PhantomPinned};

/// Specifies a constraint on where an object can be drawn.
/// Values are from 0..1 inclusive, scaled to screen space.
#[derive(Copy, Clone)]
pub struct DrawConstraints {
    // Top-left
    origin: (f32, f32),
    dimensions: Option<(f32, f32)>,
}

/// Specifies how an object should be sized by layouts.
#[derive(Copy, Clone)]
pub enum SizingPolicy {
    /// If this object will always fit to a specific logical size
    Fixed(f32),
    /// If this object will always fit to a specific physical size
    FixedPhysical(usize),
    /// If this object should expand to fit an area
    Expanding
    // TODO: Other sizing mechanisms
}

/// A completed widget is something which has been discarded by the user and is ready to
/// be drawn to the screen.
pub trait CompletedWidget<DrawInstance: Drawer> {
    /// Draws this widget to the screen.
    /// TODO: Specify location/dimensions?
    fn draw(&self, drawer: &mut DrawContext<DrawInstance>, constraints: DrawConstraints);

    /// Returns the vertical sizing policy for this widget.
    fn get_vertical_sizing_policy(&self) -> SizingPolicy;

    /// Returns the horizontal sizing policy for this widget.
    fn get_horizontal_sizing_policy(&self) -> SizingPolicy;

    /// Gets this widgets desired offset from the origin.
    fn get_offset(&self) -> (f32, f32) {
        (0f32, 0f32)
    }
}

/// This allows for widgets to be added to a container.
pub trait WidgetContainer<'font_data_life, DrawInstance: 'static + Drawer>
where
    Self: Sized,
{
    /// Adds an arbitrary widget to the screen.
    /// Implementation note: these can either be cached or drawn immediately depending on
    ///                      what the needs of the implementation are.
    fn add_widget<T: 'static + CompletedWidget<DrawInstance>>(&mut self, widget: T);

    /// Returns the widgets current style information.
    fn get_style_info(&self) -> &Style;

    /// Returns all the available fonts for this instance.
    fn get_font_info(&self) -> &[&mut FontCache<'font_data_life, DrawInstance::NativeTexture>];

    /// Starts drawing a new window to the screen.
    fn begin_window<'a>(&'a mut self, options: WindowOptions) -> Option<Window<'a, 'font_data_life, DrawInstance, Self>> {
        let style_info = self.get_style_info().to_owned();
        Some(Window {
            parent: self,
            parent_font_life : PhantomData {},
            widgets: Vec::new(),
            style: style_info,
            options,
        })
    }

    fn begin_hbox<'a>(&'a mut self) -> Option<DividedBox<'a, 'font_data_life, DrawInstance, Self>> {
        let style_info = self.get_style_info().to_owned();
        Some(DividedBox {
            parent: self,
            parent_font_life : PhantomData {},
            widgets: Vec::new(),
            style: style_info,
            direction: DividedBoxDirection::Horizontal
        })
    }

    fn begin_vbox<'a>(&'a mut self) -> Option<DividedBox<'a, 'font_data_life, DrawInstance, Self>> {
        let style_info = self.get_style_info().to_owned();
        Some(DividedBox {
            parent: self,
            parent_font_life : PhantomData {},
            widgets: Vec::new(),
            style: style_info,
            direction: DividedBoxDirection::Vertical
        })
    }

    /// Draws some text to the screen. Customise using styling.
    fn text<T: Display>(&mut self, message: T) {
        self.add_widget(Text {
            style: self.get_style_info().to_owned(),
            contents: message.to_string(),
        })
    }
}

/// A draw context contains state needed to draw items to the screen.
pub struct DrawContext<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance: Drawer> {
    drawer: &'drawer_life mut DrawInstance,
    fonts: Vec<&'font_cache_life mut FontCache<'font_data_life, DrawInstance::NativeTexture>>,
    dimensions: (usize, usize),
}

/// Defines styling across the entire document. Can be modified on a per-element basis freely
/// (and should be public where possible).
/// The WidgetContainer will create fresh clones of this when creating sub-elements.
#[derive(Clone)]
pub struct Style {
    pub text: TextStyle,
    pub window: WindowStyle,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            text: TextStyle::default(),
            window: WindowStyle::default(),
        }
    }
}

/// Context for text styling.
#[derive(Clone)]
pub struct TextStyle {
    pub color: Color,
    pub size: i32,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            color: Color::new_3byte(255, 0, 0),
            size: 20,
        }
    }
}

/// Context for window styling.
#[derive(Clone)]
pub struct WindowStyle {
    pub background: Color,
    pub titlebar_background: Color,
    pub titlebar_height: usize,
    pub border_color: Color,
    pub border: usize,
}

impl Default for WindowStyle {
    fn default() -> Self {
        WindowStyle {
            background: Color::new_3byte(255, 255, 255),
            titlebar_background: Color::new_3byte(180, 180, 180),
            titlebar_height: 30,
            border_color: Color::new_3byte(100, 100, 100),
            border: 1,
        }
    }
}

/// The root is the beginning of the UI layout. This should be fed input data to be handled
/// by objects.
pub struct Root<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance: 'static + Drawer> {
    context: DrawContext<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance>,

    // TODO: Input data

    // TODO: Consider some kind of stack for this?
    pub style: Style,
}

impl<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance: 'static + Drawer>
    WidgetContainer<'font_data_life, DrawInstance>
    for Root<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance>
{
    fn add_widget<T: 'static + CompletedWidget<DrawInstance>>(&mut self, widget: T) {
        // As the root, we don't perform any layout calculations, so pass it through directly.
        let offset = widget.get_offset();

        let width = match widget.get_horizontal_sizing_policy() {
            SizingPolicy::Fixed(x) => x,
            SizingPolicy::FixedPhysical(x) => x as f32 / self.context.dimensions.0 as f32,
            SizingPolicy::Expanding => 1.0 - offset.0
        };
        let height = match widget.get_vertical_sizing_policy() {
            SizingPolicy::Fixed(y) => y,
            SizingPolicy::FixedPhysical(y) => y as f32 / self.context.dimensions.1 as f32,
            SizingPolicy::Expanding => 1.0 - offset.1
        };

        widget.draw(&mut self.context, DrawConstraints {
            origin: offset,
            dimensions: Some((width, height))
        });
    }

    fn get_style_info(&self) -> &Style {
        &self.style
    }
    fn get_font_info(&self) -> &[&mut FontCache<'font_data_life, DrawInstance::NativeTexture>] {
        self.context.fonts.as_ref()
    }
}

/// An in-progress window. A window, much like what an operating system would have, is comprised
/// of a toolbar and contents. This is typically of a fixed dimension.
/// Note that this window's options are fixed by this point - this is for adding containers
/// and so forth.
pub struct Window<'parent, 'font_data_life, DrawInstance: 'static + Drawer, T : WidgetContainer<'font_data_life, DrawInstance>>
{
    parent: &'parent mut T,
    parent_font_life : PhantomData<&'font_data_life ()>,

    widgets: Vec<Box<dyn CompletedWidget<DrawInstance>>>,
    options: WindowOptions,
    pub style: Style,
}

impl<'parent, 'font_data_life, DrawInstance: 'static + Drawer, T : WidgetContainer<'font_data_life, DrawInstance>> Drop for Window<'parent, 'font_data_life, DrawInstance, T>
{
    fn drop(&mut self) {
        self.parent.add_widget(CompletedWindow {
            widgets: self.widgets.split_off(0),
            style: self.style.clone(),
            options: self.options.clone(),
        })
    }
}

impl<'parent, 'font_data_life, DrawInstance: 'static + Drawer, T : WidgetContainer<'font_data_life, DrawInstance>> WidgetContainer<'font_data_life, DrawInstance>
    for Window<'parent, 'font_data_life, DrawInstance, T>
{
    fn add_widget<Widget: 'static + CompletedWidget<DrawInstance>>(&mut self, widget: Widget) {
        self.widgets.push(Box::new(widget));
    }

    fn get_style_info(&self) -> &Style {
        &self.style
    }

    fn get_font_info(&self) -> &[&mut FontCache<'font_data_life, DrawInstance::NativeTexture>] {
        self.parent.get_font_info()
    }
}

/// An instance of a completed window.
struct CompletedWindow<DrawInstance: 'static + Drawer> {
    widgets: Vec<Box<dyn CompletedWidget<DrawInstance>>>,
    pub style: Style,
    options: WindowOptions,
}

impl<DrawInstance: 'static + Drawer> CompletedWidget<DrawInstance>
    for CompletedWindow<DrawInstance>
{
    fn draw(&self, drawer: &mut DrawContext<DrawInstance>, constraints: DrawConstraints) {
        // We don't intrinsically do layouts, so just render all widgets at an origin of 0x0.
        // TODO: Decorations

        // Convert from our style options into logical space
        let title_scaled_height =
            (self.style.window.titlebar_height as f32) / (drawer.dimensions.1 as f32);
        let border_scaled_width = (self.style.window.border as f32) / (drawer.dimensions.0 as f32);
        let border_scaled_height = (self.style.window.border as f32) / (drawer.dimensions.1 as f32);

        // Calculate the space needed by the window
        let window_size = constraints.dimensions.unwrap_or_else(|| self.options.size);
        let window_dimensions = Rect::new_from_logical_space(
            constraints.origin.0,
            constraints.origin.1,
            window_size.0,
            window_size.1,
            &drawer.dimensions,
        );

        // Draw background
        drawer
            .drawer
            .draw_colored_rect(&window_dimensions, &self.style.window.background);

        // If we have decorations, our widgets are going to be padded by them
        let widget_constraints = if self.options.decorations {
            DrawConstraints {
                origin: (
                    constraints.origin.0 + border_scaled_width,
                    constraints.origin.1 + title_scaled_height,
                ),
                dimensions: Some((
                    window_size.0 - border_scaled_width * 2f32,
                    window_size.1 - border_scaled_height - title_scaled_height,
                )),
            }
        } else {
            constraints
        };

        for widget in &self.widgets {
            // Note we don't do layouts - that should be handled by some container
            widget.draw(drawer, widget_constraints);
        }

        // Draw titlebar + border (if needed)
        if self.options.decorations {
            let titlebar_dimensions = Rect::new_from_logical_space(
                constraints.origin.0,
                constraints.origin.1,
                window_size.0,
                title_scaled_height,
                &drawer.dimensions,
            );

            drawer
                .drawer
                .draw_colored_rect(&titlebar_dimensions, &self.style.window.titlebar_background);

            // And finally, the border
            // We have to do this on all 4 sides
            // Draw horizontal lines
            for y in &[0f32, 1f32] {
                let border_rect = Rect::new_from_logical_space(
                    constraints.origin.0,
                    constraints.origin.1 + y * window_size.1,
                    window_size.0,
                    border_scaled_height,
                    &drawer.dimensions,
                );

                drawer
                    .drawer
                    .draw_colored_rect(&border_rect, &self.style.window.border_color);
            }

            // And vertical lines
            for x in &[0f32, 1f32] {
                let border_rect = Rect::new_from_logical_space(
                    constraints.origin.0 + x * window_size.0,
                    constraints.origin.1,
                    border_scaled_width,
                    window_size.1,
                    &drawer.dimensions,
                );

                drawer
                    .drawer
                    .draw_colored_rect(&border_rect, &self.style.window.border_color);
            }
        }

    }

    fn get_horizontal_sizing_policy(&self) -> SizingPolicy {
        SizingPolicy::Fixed(self.options.size.0)
    }

    fn get_vertical_sizing_policy(&self) -> SizingPolicy {
        SizingPolicy::Fixed(self.options.size.1)
    }

    fn get_offset(&self) -> (f32, f32) {
        self.options.position
    }
}

/// Parameters for configuring a window.
#[derive(Clone)]
pub struct WindowOptions {
    pub title: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub decorations: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            title: "Window".to_string(),
            position: (0.1, 0.1),
            size: (0.4, 0.3),
            decorations: true,
        }
    }
}

/// A completed text string to be drawn onto the screen.
/// Note that this is a single line string.
pub struct Text {
    style: Style,
    contents: String,
}

impl<DrawInstance: 'static + Drawer> CompletedWidget<DrawInstance> for Text {
    fn draw(&self, drawer: &mut DrawContext<DrawInstance>, constraints: DrawConstraints) {
        // TODO: Constrain to width/height
        // TODO: Select font
        drawer.fonts[0].draw(
            &self.contents,
            &self.style.text.color,
            self.style.text.size,
            &Position::new(
                (constraints.origin.0 * (drawer.dimensions.0 as f32)) as _,
                (constraints.origin.1 * (drawer.dimensions.1 as f32)) as _,
            ),
            drawer.drawer,
        )
    }

    fn get_vertical_sizing_policy(&self) -> SizingPolicy {
        // TODO: Don't hardcode this - calculate from font before completing
        SizingPolicy::FixedPhysical((self.style.text.size as f32 * 1.25f32) as usize)
    }

    fn get_horizontal_sizing_policy(&self) -> SizingPolicy {
        // TODO: Don't hardcode this - calculate from font before completing
        SizingPolicy::Expanding
    }
}

/// The direction a divided box is operating in.
#[derive(Copy, Clone)]
enum DividedBoxDirection {
    Horizontal,
    Vertical
}

/// A DividedBox (implemented as HBox or VBox) divides some working area between elements.
/// This respects widgets sizing requirements where possible.
pub struct DividedBox<'parent, 'font_data_life, DrawInstance: 'static + Drawer, T : WidgetContainer<'font_data_life, DrawInstance>>
{
    parent: &'parent mut T,
    parent_font_life : PhantomData<&'font_data_life ()>,

    direction : DividedBoxDirection,
    widgets: Vec<Box<dyn CompletedWidget<DrawInstance>>>,
    pub style: Style,
}

impl<'parent, 'font_data_life, DrawInstance: 'static + Drawer, T : WidgetContainer<'font_data_life, DrawInstance>> Drop for DividedBox<'parent, 'font_data_life, DrawInstance, T>
{
    fn drop(&mut self) {
        self.parent.add_widget(CompletedDividedBox {
            widgets: self.widgets.split_off(0),
            style: self.style.clone(),
            direction: self.direction,
        })
    }
}

impl<'parent, 'font_data_life, DrawInstance: 'static + Drawer, T : WidgetContainer<'font_data_life, DrawInstance>> WidgetContainer<'font_data_life, DrawInstance>
for DividedBox<'parent, 'font_data_life, DrawInstance, T>
{
    fn add_widget<Widget: 'static + CompletedWidget<DrawInstance>>(&mut self, widget: Widget) {
        self.widgets.push(Box::new(widget));
    }

    fn get_style_info(&self) -> &Style {
        &self.style
    }

    fn get_font_info(&self) -> &[&mut FontCache<'font_data_life, DrawInstance::NativeTexture>] {
        self.parent.get_font_info()
    }
}

/// A completed divided box, ready to display.
struct CompletedDividedBox<DrawInstance : 'static + Drawer> {
    direction : DividedBoxDirection,
    widgets: Vec<Box<dyn CompletedWidget<DrawInstance>>>,
    pub style: Style,
}

impl<DrawInstance: 'static + Drawer> CompletedWidget<DrawInstance> for CompletedDividedBox<DrawInstance> {
    fn draw(&self, drawer: &mut DrawContext<DrawInstance>, constraints: DrawConstraints) {
        let mut x = constraints.origin.0;
        let mut y = constraints.origin.1;

        let mut remaining_dims = constraints.dimensions.unwrap_or_else(|| (1.0, 1.0));

        for widget in &self.widgets {
            let policy = match self.direction {
                DividedBoxDirection::Horizontal => widget.get_horizontal_sizing_policy(),
                DividedBoxDirection::Vertical => widget.get_vertical_sizing_policy(),
            };

            // Calculate how much space this widget needs
            let add_logical = match policy {
                SizingPolicy::Fixed(x) => x,
                SizingPolicy::FixedPhysical(y) => y as f32 / match self.direction {
                    DividedBoxDirection::Horizontal => drawer.dimensions.0 as f32,
                    DividedBoxDirection::Vertical => drawer.dimensions.1 as f32,
                },
                SizingPolicy::Expanding => match self.direction {
                    DividedBoxDirection::Horizontal => remaining_dims.0,
                    DividedBoxDirection::Vertical => remaining_dims.1,
                },
            };

            // Give the full width/height of the unimportant axis, and the required
            let allocated_size = match self.direction {
                DividedBoxDirection::Horizontal => (add_logical, remaining_dims.1),
                DividedBoxDirection::Vertical => (remaining_dims.0, add_logical),
            };;

            widget.draw(drawer, DrawConstraints {
                origin: (x, y),
                dimensions: Some(allocated_size)
            });

            // Update our internal state so we know where to allocate the next widget(s)
            match self.direction {
                DividedBoxDirection::Horizontal => {
                    x += add_logical;
                    remaining_dims.0 -= add_logical;
                },
                DividedBoxDirection::Vertical => {
                    y += add_logical;
                    remaining_dims.1 -= add_logical;
                }
            }
        }
    }

    fn get_vertical_sizing_policy(&self) -> SizingPolicy {
        SizingPolicy::Expanding
    }

    fn get_horizontal_sizing_policy(&self) -> SizingPolicy {
        SizingPolicy::Expanding
    }
}


/// Begins a new UI root. Note that this always returns true unless the UI is hidden -
/// this is to enable consistent syntax and proper drop mechanics.
pub fn begin_root<
    'drawer_life,
    'font_cache_life,
    'font_data_life,
    DrawInstance: 'static + Drawer,
>(
    drawer: &'drawer_life mut DrawInstance,
    fonts: Vec<&'font_cache_life mut FontCache<'font_data_life, DrawInstance::NativeTexture>>,
    dimensions: (usize, usize),
) -> Option<Root<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance>> {
    Some(Root {
        context: DrawContext {
            drawer,
            fonts,
            dimensions,
        },
        style: Style::default(),
    })
}
