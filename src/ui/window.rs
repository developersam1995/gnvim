use gtk::prelude::*;

use neovim_lib::neovim_api::Window as NvimWindow;

use crate::ui::grid::Grid;

pub struct MsgWindow {
    fixed: gtk::Fixed,
    frame: gtk::Frame,
}

impl MsgWindow {
    pub fn new(fixed: gtk::Fixed, css_provider: gtk::CssProvider) -> Self {
        let frame = gtk::Frame::new(None);

        fixed.put(&frame, 0, 0);

        add_css_provider!(&css_provider, frame);

        Self { fixed, frame }
    }

    pub fn set_pos(&self, grid: &Grid, row: u64) {
        let w = grid.widget();

        // Only add/change the child widget if its different
        // from the previous one.
        if let Some(child) = self.frame.get_child() {
            if w != child {
                self.frame.remove(&child);
                w.unparent(); // Unparent the grid.
                self.frame.add(&w);
            }
        } else {
            self.frame.add(&w);
        }

        let metrics = grid.get_grid_metrics();
        let w = metrics.cols * metrics.cell_width;
        let h = metrics.rows * metrics.cell_height;
        self.frame.set_size_request(w as i32, h as i32);

        self.fixed
            .move_(&self.frame, 0, (metrics.cell_height * row) as i32);
        self.fixed.show_all();
    }
}

pub struct Window {
    fixed: gtk::Fixed,
    frame: gtk::Frame,

    external_win: Option<gtk::Window>,

    pub x: u64,
    pub y: u64,

    /// Currently shown grid's id.
    pub grid_id: i64,
    pub nvim_win: NvimWindow,
}

impl Window {
    pub fn new(
        win: NvimWindow,
        fixed: gtk::Fixed,
        grid: &Grid,
        css_provider: Option<gtk::CssProvider>,
    ) -> Self {
        let frame = gtk::Frame::new(None);
        fixed.put(&frame, 0, 0);

        let widget = grid.widget();
        frame.add(&widget);

        if let Some(css_provider) = css_provider {
            add_css_provider!(&css_provider, frame);
        }

        Self {
            fixed,
            frame,
            external_win: None,
            grid_id: grid.id,
            nvim_win: win,
            x: 0,
            y: 0,
        }
    }

    pub fn set_external(&mut self, parent: &gtk::Window, size: (i32, i32)) {
        if self.external_win.is_some() {
            return;
        }

        let win = gtk::Window::new(gtk::WindowType::Toplevel);
        self.fixed.remove(&self.frame);
        win.add(&self.frame);

        win.set_default_size(size.0, size.1);
        win.set_accept_focus(false);
        win.set_deletable(false);
        win.set_resizable(false);

        win.set_transient_for(Some(parent));
        win.set_attached_to(Some(parent));

        win.show_all();

        self.external_win = Some(win);
    }

    pub fn set_position(&mut self, x: u64, y: u64, w: u64, h: u64) {
        if let Some(win) = self.external_win.take() {
            win.remove(&self.frame);
            self.fixed.add(&self.frame);
            win.close();
        }

        self.x = x;
        self.y = y;
        self.fixed.move_(&self.frame, x as i32, y as i32);

        self.frame.set_size_request(w as i32, h as i32);
    }

    pub fn show(&self) {
        self.frame.show_all();
    }

    pub fn hide(&self) {
        self.frame.hide();
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // TODO(ville): Test that we release all resources.
        if let Some(child) = self.frame.get_child() {
            // We don't want to destroy the child widget, so just remove the child from our
            // container.
            self.frame.remove(&child);
        }
        self.frame.destroy();

        if let Some(ref win) = self.external_win {
            win.destroy();
        }
    }
}
