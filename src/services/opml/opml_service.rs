use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Result, Context};
use chrono::Utc;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Reader;
use quick_xml::Writer;
use log::{info, error};
use std::sync::Arc;

use crate::core::{FeedRepository, CategoryRepository};
use crate::models::{Feed, Category, CategoryId};

/// Service for importing and exporting OPML files
pub struct OpmlService {
    feed_repository: Arc<dyn FeedRepository>,
    category_repository: Arc<dyn CategoryRepository>,
}

impl OpmlService {
    /// Creates a new OPML service
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        category_repository: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            feed_repository,
            category_repository,
        }
    }
    
    /// Imports feeds from an OPML file
    pub fn import<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Feed>> {
        info!("Importing OPML from {:?}", path.as_ref());
        
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut feeds = Vec::new();
        let mut current_category: Option<String> = None;
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"outline" => {
                    let mut title = None;
                    let mut xml_url = None;
                    let mut html_url = None;
                    
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"text" | b"title" => title = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                            b"xmlUrl" => xml_url = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                            b"htmlUrl" => html_url = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                            _ => {}
                        }
                    }
                    
                    if let (Some(title), Some(xml_url)) = (title, xml_url) {
                        let mut feed = Feed::new(title, xml_url.parse()?);
                        if let Some(url) = html_url {
                            feed.website_url = Some(url.parse()?);
                        }
                        
                        // Handle category
                        if let Some(category_name) = &current_category {
                            // Try to find existing category
                            let categories = self.category_repository.search_categories(category_name)?;
                            if let Some(category) = categories.first() {
                                feed.category_id = Some(category.id.clone());
                            } else {
                                // Create new category if it doesn't exist
                                let new_category = Category::new(category_name.clone());
                                self.category_repository.save_category(&new_category)?;
                                feed.category_id = Some(new_category.id.clone());
                            }
                        }
                        
                        feeds.push(feed);
                    }
                }
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"outline" => {
                    for attr in e.attributes() {
                        let attr = attr?;
                        if attr.key.as_ref() == b"text" || attr.key.as_ref() == b"title" {
                            current_category = Some(String::from_utf8_lossy(&attr.value).into_owned());
                            break;
                        }
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"outline" => {
                    current_category = None;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }
        
        // Save imported feeds
        for feed in &feeds {
            if let Err(e) = self.feed_repository.save_feed(feed) {
                error!("Failed to save feed {}: {}", feed.title, e);
            }
        }
        
        info!("Imported {} feeds from OPML", feeds.len());
        Ok(feeds)
    }
    
    /// Exports feeds to an OPML file
    pub fn export<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        info!("Exporting OPML to {:?}", path.as_ref());
        
        let mut writer = Writer::new(File::create(path)?);
        
        // Write OPML header
        writer.write_event(Event::Start(BytesStart::new("opml").with_attributes([("version", "1.0")])))?;
        writer.write_event(Event::Start(BytesStart::new("head")))?;
        
        // Write title
        writer.write_event(Event::Start(BytesStart::new("title")))?;
        writer.write_event(Event::Text("Panda RSS Feeds".into()))?;
        writer.write_event(Event::End(BytesStart::new("title")))?;
        
        // Write date created
        writer.write_event(Event::Start(BytesStart::new("dateCreated")))?;
        writer.write_event(Event::Text(Utc::now().to_rfc3339().into()))?;
        writer.write_event(Event::End(BytesStart::new("dateCreated")))?;
        
        writer.write_event(Event::End(BytesStart::new("head")))?;
        
        // Write body
        writer.write_event(Event::Start(BytesStart::new("body")))?;
        
        // Get all feeds and group by category
        let feeds = self.feed_repository.get_all_feeds()?;
        let mut feeds_by_category: std::collections::HashMap<Option<String>, Vec<&Feed>> = std::collections::HashMap::new();
        
        for feed in &feeds {
            feeds_by_category
                .entry(feed.category_name.clone())
                .or_default()
                .push(feed);
        }
        
        // Write uncategorized feeds first
        if let Some(feeds) = feeds_by_category.get(&None) {
            for feed in feeds {
                self.write_feed(&mut writer, feed)?;
            }
        }
        
        // Write categorized feeds
        for (category, feeds) in feeds_by_category.iter() {
            if let Some(category_name) = category {
                // Write category outline
                let mut start = BytesStart::new("outline");
                start.push_attribute(("text", category_name.as_str()));
                writer.write_event(Event::Start(start))?;
                
                // Write feeds in this category
                for feed in feeds {
                    self.write_feed(&mut writer, feed)?;
                }
                
                writer.write_event(Event::End(BytesStart::new("outline")))?;
            }
        }
        
        writer.write_event(Event::End(BytesStart::new("body")))?;
        writer.write_event(Event::End(BytesStart::new("opml")))?;
        
        info!("Exported {} feeds to OPML", feeds.len());
        Ok(())
    }
    
    /// Helper function to write a feed as an OPML outline
    fn write_feed<W: Write>(&self, writer: &mut Writer<W>, feed: &Feed) -> Result<()> {
        let mut start = BytesStart::new("outline");
        start.push_attribute(("text", feed.title.as_str()));
        start.push_attribute(("type", "rss"));
        start.push_attribute(("xmlUrl", feed.url.as_str()));
        
        if let Some(website_url) = &feed.website_url {
            start.push_attribute(("htmlUrl", website_url.as_str()));
        }
        
        writer.write_event(Event::Empty(start))?;
        
        Ok(())
    }
}