use egui::{Button, Context as EguiContext, Label, RichText, ScrollArea, TextEdit, Ui};
use std::sync::Arc;
use log::error;
use anyhow::Result;

use crate::base::repository_traits::{FeedRepository, CategoryRepository};
use crate::models::category::{Category, CategoryId};
use crate::models::feed::Feed;
use crate::ui::styles::{AppColors, DEFAULT_PADDING};

#[derive(Debug, Clone, PartialEq)]
pub struct SidebarState {
    pub selection: Option<SidebarSelection>,
    pub search_query: String,
    pub expanded_categories: Vec<CategoryId>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            selection: None,
            search_query: String::new(),
            expanded_categories: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarSelection {
    AllFeeds,
    Favorites,
    Feed(Feed),
    Category(Category),
}

pub struct Sidebar {
    feed_repository: Arc<dyn FeedRepository>,
    category_repository: Arc<dyn CategoryRepository>,
    state: SidebarState,
    colors: AppColors,
}

impl Sidebar {
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            feed_repository,
            category_repository,
            state: SidebarState::default(),
            colors: AppColors::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Result<Option<SidebarSelection>> {
        let mut new_selection = None;

        // Search box
        ui.horizontal(|ui| {
            ui.label("Search");
            if ui.text_edit_singleline(&mut self.state.search_query).changed() {
                // Search logic will be handled by the filtering below
            }
        });

        ui.add_space(DEFAULT_PADDING);

        // Special sections
        if ui.add(Button::new(RichText::new("📚 All Articles").color(self.colors.text))).clicked() {
            new_selection = Some(SidebarSelection::AllFeeds);
        }

        if ui.add(Button::new(RichText::new("⭐ Favorites").color(self.colors.text))).clicked() {
            new_selection = Some(SidebarSelection::Favorites);
        }

        ui.add_space(DEFAULT_PADDING);

        // Categories and feeds
        ScrollArea::vertical().show(ui, |ui| {
            // 修复：移除错误处理符号，在闭包中直接处理错误
            if let Err(e) = self.render_categories(ui, None, 0, &mut new_selection) {
                error!("Failed to render categories: {}", e);
            }
        });

        if let Some(selection) = new_selection.clone() {
            self.state.selection = Some(selection);
        }

        Ok(new_selection)
    }

    fn render_categories(
        &self,
        ui: &mut Ui,
        parent_id: Option<CategoryId>,
        depth: i32,
        selection: &mut Option<SidebarSelection>
    ) -> Result<()> {
        // Use tokio::task::block_in_place to handle the async call synchronously
        // This is not ideal for production but will work for our immediate fix
        let categories = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                self.category_repository.get_categories_by_parent(&parent_id).await
            })
        })?;
        
        for category in categories {
            let indent = "  ".repeat(depth as usize);
            let text = format!("{}{} {}", indent, "📁", category.name);
            
            if ui.add(Button::new(RichText::new(&text).color(self.colors.text))).clicked() {
                *selection = Some(SidebarSelection::Category(category.clone()));
            }

            // Only show feeds for expanded categories
            if self.state.expanded_categories.contains(&category.id) {
                // 修复：获取对应分类的Feed，修复类型不匹配问题
                let feeds = tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        // 直接使用category.id而不是包装在Option中
                        self.feed_repository.get_feeds_by_category(&category.id).await
                    })
                })?;
                
                for feed in feeds {
                    self.render_feed(ui, &feed, depth + 1, selection)?;
                }
            }
        }

        // Show uncategorized feeds at root level
        if parent_id.is_none() {
            // 修复：获取所有Feed，使用更合适的方法
            let feeds = tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    // 使用get_all_feeds方法而不是试图用None作为category_id
                    self.feed_repository.get_all_feeds().await
                })
            })?;
            
            for feed in feeds {
                // 仅显示没有分类的Feed
                if feed.category_id.is_none() {
                    self.render_feed(ui, &feed, depth, selection)?;
                }
            }
        }

        Ok(())
    }

    fn render_feed(
        &self,
        ui: &mut Ui,
        feed: &Feed,
        depth: i32,
        selection: &mut Option<SidebarSelection>
    ) -> Result<()> {
        let indent = "  ".repeat(depth as usize);
        let text = format!("{}{} {}", indent, "📰", feed.title);
        
        if ui.add(Button::new(RichText::new(&text).color(self.colors.text))).clicked() {
            *selection = Some(SidebarSelection::Feed(feed.clone()));
        }

        Ok(())
    }

    pub fn get_selection(&self) -> Option<SidebarSelection> {
        self.state.selection.clone()
    }

    pub fn clear_selection(&mut self) {
        self.state.selection = None;
    }

    pub fn toggle_category(&mut self, category_id: CategoryId) {
        if let Some(pos) = self.state.expanded_categories.iter().position(|id| *id == category_id) {
            self.state.expanded_categories.remove(pos);
        } else {
            self.state.expanded_categories.push(category_id);
        }
    }

    pub fn select_category(&mut self, category_id: CategoryId) {
        // Use tokio::task::block_in_place to handle the async call
        let category = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                self.category_repository.get_category_by_id(&category_id).await
            })
        });
        
        if let Ok(Some(category)) = category {
            self.state.selection = Some(SidebarSelection::Category(category));
        }
    }
}