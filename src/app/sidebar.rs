//! Shared phone sidebar navigation.
use super::{App, chrome};
use crate::state::{PanelPage, SidebarTab};

impl<P: super::SendEvent> App<P> {
	pub(super) fn phone_folder_open(&self) -> bool {
		self.tab_strip.phone
			&& self.interaction.panel == PanelPage::Tabs
			&& self.interaction.sidebar_tab == SidebarTab::Folder
	}

	pub(super) fn switch_sidebar_tab(&mut self, tab: SidebarTab) {
		if !self.tab_strip.phone {
			return;
		}
		self.interaction.sidebar_tab = tab;
		self.folder.open = false;
		self.interaction.folder_animation = None;
		self.interaction.show_panel(PanelPage::Tabs);
		self.interaction.widgets.focus =
			Some(crate::state::Command::SidebarTab(tab).widget_id());
		self.refresh_hover();
		self.redraw();
	}

	pub(super) fn toggle_phone_sidebar(&mut self) {
		if !self.tab_strip.phone {
			return;
		}
		self.readers.session.cancel_scroll_animation();
		self.tab_strip.cancel_drag();
		if self.interaction.panel == PanelPage::Tabs {
			self.interaction.show_panel(PanelPage::Closed);
		} else {
			self.switch_sidebar_tab(self.interaction.sidebar_tab);
			if self.interaction.sidebar_tab == SidebarTab::Tabs
				&& self.readers.session.path.is_some()
			{
				let (width, height, _) = self.dimensions();
				let list = chrome::tab_drawer::list(
					width,
					height,
					self.readers.entries().len(),
					self.tab_strip.drawer_scroll,
					320_f32,
				);
				let row = list.row_rect(self.readers.active());
				self.tab_strip.drawer_scroll = (list.scroll
					+ (row.y - list.viewport.y).min(0.)
					+ (row.y + row.h - list.viewport.y - list.viewport.h)
						.max(0.))
				.clamp(0., list.max_scroll());
			}
		}
		self.refresh_hover();
		self.redraw();
	}
}
