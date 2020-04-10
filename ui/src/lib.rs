//! An intermediate-mode UI layout and rendering API for Leaffront - designed to not be too complex
//! for designing layouts.

extern crate leaffront_core;

use leaffront_core::pos::Position;
use leaffront_core::render::color::Color;
use leaffront_core::render::font::FontCache;
use leaffront_core::render::Drawer;
use std::fmt::Display;

/// A completed widget is something which has been discarded by the user and is ready to
/// be drawn to the screen.
pub trait CompletedWidget<DrawInstance: Drawer> {
    /// Draws this widget to the screen.
    /// TODO: Specify location/dimensions?
    fn draw(&self, drawer: &mut DrawContext<DrawInstance>);
}

/// This allows for widgets to be added to a container.
pub trait WidgetContainer<DrawInstance: 'static + Drawer>
where
    Self: Sized,
{
    /// Adds an arbitrary widget to the screen.
    /// Implementation note: these can either be cached or drawn immediately depending on
    ///                      what the needs of the implementation are.
    fn add_widget<T: 'static + CompletedWidget<DrawInstance>>(&mut self, widget: T);

    /// Returns the widgets current style information.
    fn get_style_info(&self) -> &Style;

    /// Starts drawing a new window to the screen.
    fn begin_window(&mut self, options: WindowOptions) -> Option<Window<Self, DrawInstance>> {
        let style_info = self.get_style_info().to_owned();
        Some(Window {
            parent: self,
            widgets: Vec::new(),
            style: style_info,
            options,
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
}

/// Defines styling across the entire document. Can be modified on a per-element basis freely
/// (and should be public where possible).
/// The WidgetContainer will create fresh clones of this when creating sub-elements.
#[derive(Clone)]
pub struct Style {
    text: TextStyle,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            text: TextStyle::default(),
        }
    }
}

/// Context for text styling.
#[derive(Clone)]
pub struct TextStyle {
    color: Color,
    size: i32,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            color: Color::new_3byte(255, 0, 0),
            size: 20,
        }
    }
}

/// The root is the beginning of the UI layout. This should be fed input data to be handled
/// by objects.
pub struct Root<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance: 'static + Drawer> {
    context: DrawContext<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance>,

    // TODO: Input data
    dimensions: (usize, usize),

    // TODO: Consider some kind of stack for this?
    style: Style,
}

impl<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance: 'static + Drawer>
    WidgetContainer<DrawInstance>
    for Root<'drawer_life, 'font_cache_life, 'font_data_life, DrawInstance>
{
    fn add_widget<T: 'static + CompletedWidget<DrawInstance>>(&mut self, widget: T) {
        // As the root, we don't perform any layout calculations, so pass it through directly.
        widget.draw(&mut self.context);
    }

    fn get_style_info(&self) -> &Style {
        &self.style
    }
}

/// An in-progress window. A window, much like what an operating system would have, is comprised
/// of a toolbar and contents. This is typically of a fixed dimension.
/// Note that this window's options are fixed by this point - this is for adding containers
/// and so forth.
pub struct Window<'parent, T, DrawInstance: 'static + Drawer>
where
    T: WidgetContainer<DrawInstance>,
{
    parent: &'parent mut T,
    widgets: Vec<Box<dyn CompletedWidget<DrawInstance>>>,
    options: WindowOptions,
    style: Style,
}

impl<'parent, T, DrawInstance: 'static + Drawer> Drop for Window<'parent, T, DrawInstance>
where
    T: WidgetContainer<DrawInstance>,
{
    fn drop(&mut self) {
        self.parent.add_widget(CompletedWindow {
            widgets: self.widgets.split_off(0),
            options: self.options.clone(),
        })
    }
}

impl<'parent, T, DrawInstance: 'static + Drawer> WidgetContainer<DrawInstance>
    for Window<'parent, T, DrawInstance>
where
    T: WidgetContainer<DrawInstance>,
{
    fn add_widget<Widget: 'static + CompletedWidget<DrawInstance>>(&mut self, widget: Widget) {
        self.widgets.push(Box::new(widget));
    }

    fn get_style_info(&self) -> &Style {
        &self.style
    }
}

/// An instance of a completed window.
struct CompletedWindow<DrawInstance: 'static + Drawer> {
    widgets: Vec<Box<dyn CompletedWidget<DrawInstance>>>,
    options: WindowOptions,
}

impl<DrawInstance: 'static + Drawer> CompletedWidget<DrawInstance>
    for CompletedWindow<DrawInstance>
{
    fn draw(&self, drawer: &mut DrawContext<DrawInstance>) {
        // We don't intrinsically do layouts, so just render all widgets at an origin of 0x0.
        // TODO: Decorations
        for widget in &self.widgets {
            widget.draw(drawer);
        }
    }
}

/// Parameters for configuring a window.
#[derive(Clone)]
pub struct WindowOptions {
    pub title: String,
    pub position: (usize, usize),
    pub size: (usize, usize),
    pub decorations: bool,
}

impl Default for WindowOptions {
    fn default() -> Self {
        WindowOptions {
            title: "Window".to_string(),
            position: (10, 10),
            size: (256, 100),
            decorations: true,
        }
    }
}

/// A completed text string to be drawn onto the screen.
pub struct Text {
    style: Style,
    contents: String,
}

impl<DrawInstance: 'static + Drawer> CompletedWidget<DrawInstance> for Text {
    fn draw(&self, drawer: &mut DrawContext<DrawInstance>) {
        drawer.fonts[0].draw(
            &self.contents,
            &self.style.text.color,
            self.style.text.size,
            // TODO: Relative to what the parent wants
            &Position::new(0, 0),
            drawer.drawer,
        )
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
        context: DrawContext { drawer, fonts },
        dimensions,
        style: Style::default(),
    })
}
