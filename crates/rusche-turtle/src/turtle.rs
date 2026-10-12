//! Turtle model. Side effects go through [`Sink`].

use std::sync::mpsc;

use rusche::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 220, g: 50, b: 47 };
    pub const GREEN: Color = Color { r: 133, g: 153, b: 0 };
    pub const BLUE: Color = Color { r: 38, g: 139, b: 210 };
    pub const YELLOW: Color = Color { r: 181, g: 137, b: 0 };
    pub const ORANGE: Color = Color { r: 203, g: 75, b: 22 };
    pub const PURPLE: Color = Color { r: 108, g: 113, b: 196 };
    pub const CYAN: Color = Color { r: 42, g: 161, b: 152 };
    pub const MAGENTA: Color = Color { r: 211, g: 54, b: 130 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128 };
    pub const BROWN: Color = Color { r: 139, g: 90, b: 43 };

    pub fn from_name(name: &str) -> Option<Color> {
        Some(match name.to_ascii_lowercase().as_str() {
            "black" => Color::BLACK,
            "white" => Color::WHITE,
            "red" => Color::RED,
            "green" => Color::GREEN,
            "blue" => Color::BLUE,
            "yellow" => Color::YELLOW,
            "orange" => Color::ORANGE,
            "purple" | "violet" => Color::PURPLE,
            "cyan" | "turquoise" => Color::CYAN,
            "magenta" | "pink" => Color::MAGENTA,
            "gray" | "grey" => Color::GRAY,
            "brown" => Color::BROWN,
            _ => return None,
        })
    }

    pub fn clamp_channel(v: f64) -> u8 {
        v.round().clamp(0.0, 255.0) as u8
    }

    /// Canonical name for a color created with [`Self::from_name`].
    ///
    /// Aliases such as `violet` and `grey` share a color with `purple` and `gray`, so those
    /// report the canonical name. A color built from raw channels has no name.
    pub fn name(self) -> Option<&'static str> {
        match self {
            Self::BLACK => Some("black"),
            Self::WHITE => Some("white"),
            Self::RED => Some("red"),
            Self::GREEN => Some("green"),
            Self::BLUE => Some("blue"),
            Self::YELLOW => Some("yellow"),
            Self::ORANGE => Some("orange"),
            Self::PURPLE => Some("purple"),
            Self::CYAN => Some("cyan"),
            Self::MAGENTA => Some("magenta"),
            Self::GRAY => Some("gray"),
            Self::BROWN => Some("brown"),
            _ => None,
        }
    }
}

/// Stroke width of a freshly created or cleared turtle.
pub const DEFAULT_PEN_WIDTH: f32 = 1.5;

/// Line in turtle coords (origin center, y-up).
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub color: Color,
    pub width: f32,
}

/// Where an evaluation request came from. Controls where Print/Error/Result land in the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Script,
    Repl,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TurtleEvent {
    Line(Segment),
    Move {
        x: f64,
        y: f64,
        heading: f64,
        visible: bool,
    },
    /// Degrees; positive = counter-clockwise.
    Turn { delta: f64 },
    Clear,
    Print(String),
    Error(String),
    /// Pen state after `penup`/`pendown`, `pencolor`, `pensize`, or `clear`.
    Pen {
        down: bool,
        color: Color,
        width: f32,
    },
    /// Source span of the script form that produced the events that follow, until the next
    /// `Site`. `None` when the call did not come from the script source.
    Site(Option<Span>),
    /// A new evaluation request is starting. Routes subsequent Print/Error/Result.
    Started(Origin),
    /// Value of a successful REPL evaluation.
    Result(String),
    /// The current evaluation request finished (success or failure).
    Done,
}

pub trait Sink {
    fn emit(&mut self, event: TurtleEvent);
}

impl Sink for Vec<TurtleEvent> {
    fn emit(&mut self, event: TurtleEvent) {
        self.push(event);
    }
}

impl Sink for mpsc::Sender<TurtleEvent> {
    fn emit(&mut self, event: TurtleEvent) {
        let _ = self.send(event);
    }
}

pub struct Turtle<S: Sink> {
    pub x: f64,
    pub y: f64,
    /// Degrees; 0 = east, positive = counter-clockwise.
    pub heading: f64,
    pub pen_down: bool,
    pub color: Color,
    pub width: f32,
    pub visible: bool,
    sink: S,
    /// Last site emitted, so consecutive calls from the same form emit one `Site`.
    site: Option<Option<Span>>,
}

impl<S: Sink> Turtle<S> {
    pub fn new(sink: S) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            heading: 90.0,
            pen_down: true,
            color: Color::BLACK,
            width: DEFAULT_PEN_WIDTH,
            visible: true,
            sink,
            site: None,
        }
    }

    /// Records the script form responsible for the events emitted next. Emits
    /// [`TurtleEvent::Site`] only when the site changes.
    pub fn set_site(&mut self, span: Option<Span>) {
        if self.site != Some(span) {
            self.site = Some(span);
            self.sink.emit(TurtleEvent::Site(span));
        }
    }

    fn emit_move(&mut self) {
        self.sink.emit(TurtleEvent::Move {
            x: self.x,
            y: self.y,
            heading: self.heading,
            visible: self.visible,
        });
    }

    fn stroke_to(&mut self, x1: f64, y1: f64) {
        if self.pen_down {
            self.sink.emit(TurtleEvent::Line(Segment {
                x0: self.x,
                y0: self.y,
                x1,
                y1,
                color: self.color,
                width: self.width,
            }));
        }
        self.x = x1;
        self.y = y1;
        self.emit_move();
    }

    fn move_by(&mut self, distance: f64) {
        let rad = self.heading.to_radians();
        self.stroke_to(
            self.x + distance * rad.cos(),
            self.y + distance * rad.sin(),
        );
    }

    pub fn forward(&mut self, distance: f64) {
        self.move_by(distance);
    }

    pub fn back(&mut self, distance: f64) {
        self.move_by(-distance);
    }

    pub fn left(&mut self, degrees: f64) {
        self.heading = normalize_heading(self.heading + degrees);
        self.sink.emit(TurtleEvent::Turn { delta: degrees });
    }

    pub fn right(&mut self, degrees: f64) {
        self.heading = normalize_heading(self.heading - degrees);
        self.sink.emit(TurtleEvent::Turn { delta: -degrees });
    }

    pub fn goto(&mut self, x: f64, y: f64) {
        self.stroke_to(x, y);
    }

    pub fn set_heading(&mut self, degrees: f64) {
        let target = normalize_heading(degrees);
        let delta = shortest_turn(self.heading, target);
        self.heading = target;
        if delta.abs() > 1e-9 {
            self.sink.emit(TurtleEvent::Turn { delta });
        }
    }

    pub fn home(&mut self) {
        let was_down = self.pen_down;
        self.pen_down = false;
        self.goto(0.0, 0.0);
        self.pen_down = was_down;
        self.set_heading(90.0);
    }

    pub fn pen_up(&mut self) {
        self.pen_down = false;
        self.emit_pen();
    }

    pub fn pen_down(&mut self) {
        self.pen_down = true;
        self.emit_pen();
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
        self.emit_pen();
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width.max(0.1);
        self.emit_pen();
    }

    fn emit_pen(&mut self) {
        self.sink.emit(TurtleEvent::Pen {
            down: self.pen_down,
            color: self.color,
            width: self.width,
        });
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.emit_move();
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.emit_move();
    }

    pub fn clear(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.heading = 90.0;
        self.pen_down = true;
        self.color = Color::BLACK;
        self.width = DEFAULT_PEN_WIDTH;
        self.visible = true;
        self.sink.emit(TurtleEvent::Clear);
        self.emit_pen();
        self.emit_move();
    }

    pub fn print(&mut self, text: String) {
        self.sink.emit(TurtleEvent::Print(text));
    }
}

fn normalize_heading(degrees: f64) -> f64 {
    let h = degrees.rem_euclid(360.0);
    if h == 0.0 {
        0.0
    } else {
        h
    }
}

/// Signed shortest turn from `from` to `to`, in (-180, 180].
fn shortest_turn(from: f64, to: f64) -> f64 {
    let mut delta = normalize_heading(to) - normalize_heading(from);
    if delta > 180.0 {
        delta -= 360.0;
    } else if delta <= -180.0 {
        delta += 360.0;
    }
    delta
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(events: &[TurtleEvent]) -> Vec<&Segment> {
        events
            .iter()
            .filter_map(|e| match e {
                TurtleEvent::Line(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn square_returns_to_origin() {
        let mut t = Turtle::new(Vec::new());
        for _ in 0..4 {
            t.forward(10.0);
            t.right(90.0);
        }
        assert!(t.x.abs() < 1e-9 && t.y.abs() < 1e-9);
        assert!((t.heading - 90.0).abs() < 1e-9);
        assert_eq!(lines(&t.sink).len(), 4);
    }

    #[test]
    fn pen_up_moves_without_drawing() {
        let mut t = Turtle::new(Vec::new());
        t.pen_up();
        t.forward(50.0);
        assert!(lines(&t.sink).is_empty());
        assert!((t.y - 50.0).abs() < 1e-9);
    }

    #[test]
    fn left_and_right_adjust_heading() {
        let mut t = Turtle::new(Vec::new());
        t.left(45.0);
        assert!((t.heading - 135.0).abs() < 1e-9);
        assert!(matches!(
            t.sink.last(),
            Some(TurtleEvent::Turn { delta }) if (*delta - 45.0).abs() < 1e-9
        ));
        t.right(90.0);
        assert!((t.heading - 45.0).abs() < 1e-9);
        assert!(matches!(
            t.sink.last(),
            Some(TurtleEvent::Turn { delta }) if (*delta + 90.0).abs() < 1e-9
        ));
    }

    #[test]
    fn home_resets_without_drawing() {
        let mut t = Turtle::new(Vec::new());
        t.forward(30.0);
        t.right(45.0);
        let before = lines(&t.sink).len();
        t.home();
        assert_eq!(lines(&t.sink).len(), before);
        assert!(t.x.abs() < 1e-9 && t.y.abs() < 1e-9);
        assert!((t.heading - 90.0).abs() < 1e-9);
    }

    #[test]
    fn clear_emits_clear_and_resets() {
        let mut t = Turtle::new(Vec::new());
        t.set_color(Color::RED);
        t.forward(10.0);
        t.clear();
        assert!(t.sink.iter().any(|e| matches!(e, TurtleEvent::Clear)));
        assert_eq!(t.color, Color::BLACK);
        assert!(t.x.abs() < 1e-9);
    }

    #[test]
    fn pen_commands_emit_pen_state() {
        let mut t = Turtle::new(Vec::new());
        t.pen_up();
        t.set_color(Color::RED);
        t.set_width(4.0);
        t.set_width(0.0);
        t.pen_down();
        t.clear();

        let pens: Vec<_> = t
            .sink
            .iter()
            .filter_map(|e| match e {
                TurtleEvent::Pen { down, color, width } => Some((*down, *color, *width)),
                _ => None,
            })
            .collect();
        assert_eq!(
            pens,
            vec![
                (false, Color::BLACK, DEFAULT_PEN_WIDTH),
                (false, Color::RED, DEFAULT_PEN_WIDTH),
                (false, Color::RED, 4.0),
                (false, Color::RED, 0.1),
                (true, Color::RED, 0.1),
                (true, Color::BLACK, DEFAULT_PEN_WIDTH),
            ]
        );
    }

    #[test]
    fn color_from_name() {
        assert_eq!(Color::from_name("Red"), Some(Color::RED));
        assert_eq!(Color::from_name("violet"), Some(Color::PURPLE));
        assert!(Color::from_name("mauve").is_none());
        assert_eq!(Color::BLUE.name(), Some("blue"));
        assert_eq!(Color::from_name("violet").unwrap().name(), Some("purple"));
        assert_eq!(Color::from_name("grey").unwrap().name(), Some("gray"));
        assert_eq!(Color { r: 10, g: 20, b: 30 }.name(), None);
    }

    #[test]
    fn goto_with_pen_draws() {
        let mut t = Turtle::new(Vec::new());
        t.goto(10.0, 20.0);
        let segs = lines(&t.sink);
        assert_eq!(segs.len(), 1);
        assert!((segs[0].x1 - 10.0).abs() < 1e-9);
        assert!((segs[0].y1 - 20.0).abs() < 1e-9);
    }

    #[test]
    fn set_site_emits_only_on_change() {
        use rusche::Loc;

        let a = Some(Span::new(Loc::new(0, 0), Loc::new(0, 7)));
        let b = Some(Span::new(Loc::new(1, 0), Loc::new(1, 7)));

        let mut t = Turtle::new(Vec::new());
        t.set_site(a);
        t.set_site(a);
        t.forward(1.0);
        t.set_site(b);
        t.set_site(None);
        t.set_site(None);

        let sites: Vec<_> = t
            .sink
            .iter()
            .filter_map(|e| match e {
                TurtleEvent::Site(s) => Some(*s),
                _ => None,
            })
            .collect();
        assert_eq!(sites, vec![a, b, None]);
        assert!(matches!(t.sink[0], TurtleEvent::Site(_)));
        assert!(matches!(t.sink[1], TurtleEvent::Line(_)));
    }

    #[test]
    fn channel_sink_forwards() {
        let (tx, rx) = mpsc::channel();
        let mut t = Turtle::new(tx);
        t.print("hi".into());
        t.forward(1.0);
        drop(t);
        let events: Vec<_> = rx.try_iter().collect();
        assert!(matches!(&events[0], TurtleEvent::Print(s) if s == "hi"));
        assert!(matches!(&events[1], TurtleEvent::Line(_)));
    }
}
