use iter_identify_first_last::IteratorIdentifyFirstLastExt;
use crate::base::{Vector, Rect, Point, Range1d, Screen, Fg, Bg, text_width};

pub struct RenderPort<'a> {
    pub(crate) screen: &'a mut dyn Screen,
    pub(crate) bounds: Rect,
    pub(crate) offset: Vector,
    pub(crate) invalidated_rect: Rect,
    pub(crate) cursor: Option<Point>,
}

impl<'a> RenderPort<'a> {
    pub fn text(&mut self, p: Point, color: (Fg, Bg), text: &str) {
        let screen_size = self.screen.size();
        let p = p.offset(self.offset);
        if !self.bounds.v_range().contains(p.y) { return; }
        if !self.invalidated_rect.v_range().contains(p.y) { return; }
        if p.y < 0 || p.y >= screen_size.y { return; }
        if p.x >= self.bounds.r() || p.x >= self.invalidated_rect.r() { return; } // don't screen do same check?
        let rendered = self.screen.out(
            p, color.0, color.1, text, self.bounds.h_range(), self.invalidated_rect.h_range()
        );
        self.invalidated_rect = self.invalidated_rect.union_intersect(
            Rect::from_h_v_ranges(rendered, Range1d { start: p.y, end: p.y.wrapping_add(1) }),
            Rect { tl: Point { x: 0, y: 0 }, size: screen_size }
        );
        if let Some(cursor) = self.cursor {
            if p.y == cursor.y && rendered.contains(cursor.x) {
                self.cursor = None;
            }
        }
    }

    pub fn cursor(&mut self, p: Point) {
        let screen_size = self.screen.size();
        let p = p.offset(self.offset);
        if p.y < 0 || p.y >= self.screen.size().y { return; }
        if !self.bounds.contains(p) || !self.invalidated_rect.contains(p) { return; }
        self.cursor = Some(p);
        self.invalidated_rect = self.invalidated_rect.union_intersect(
            Rect { tl: p, size: Vector { x: 1, y: 1 } },
            Rect { tl: Point { x: 0, y: 0 }, size: screen_size }
        );
    }

    pub fn fill(&mut self, mut f: impl FnMut(&mut Self, Point)) {
        for p in self.bounds.intersect(self.invalidated_rect).points() {
            f(self, p.offset(-self.offset));
        }
    }

    pub fn label(&mut self, mut p: Point, color: (Fg, Bg), color_hotkey: (Fg, Bg), text: &str) {
        let mut hotkey = false;
        for (first, last, text) in text.split('~').identify_first_last() {
            if !first && !text.is_empty() {
                hotkey = !hotkey;
            }
            let actual_text = if !first && !last && text.is_empty() { "~" } else { text };
            self.text(p, if hotkey { color_hotkey } else { color }, actual_text);
            p = p.offset(Vector { x: text_width(actual_text), y: 0 });
            if !first && text.is_empty() {
                hotkey = !hotkey;
            }
        }
    }

    pub fn fill_bg(&mut self, color: (Fg, Bg)) {
        self.fill(|rp, p| rp.text(p, color, " "));
    }

    pub fn h_line(&mut self, start: Point, len: i16, double: bool, color: (Fg, Bg)) {
        let s = if double { "═" } else { "─" };
        for x in Range1d::new(start.x, start.x.wrapping_add(len)) {
            self.text(Point { x, y: start.y }, color, s);
        }
    }

    pub fn v_line(&mut self, start: Point, len: i16, double: bool, color: (Fg, Bg)) {
        let s = if double { "║" } else { "│" };
        for y in Range1d::new(start.y, start.y.wrapping_add(len)) {
            self.text(Point { x: start.x, y }, color, s);
        }
    }

    pub fn tl_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╔" } else { "┌" });
    }

    pub fn tr_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╗" } else { "┐" });
    }

    pub fn bl_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╚" } else { "└" });
    }

    pub fn br_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╝" } else { "┘" });
    }
}
