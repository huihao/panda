use std::io::Cursor;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Reader;
use quick_xml::writer::Writer;
use anyhow::Result;
use url::Url;
use std::sync::Arc;

use crate::models::feed::{Feed, FeedId};
use crate::models::category::Category;
use crate::services::rss::RssService;

#[derive(Debug)]
struct Outline {
    text: String,
    xml_url: Option<String>,
    html_url: Option<String>,
}

pub struct OpmlService {
    rss_service: Arc<RssService>,
}

impl OpmlService {
    pub fn new(rss_service: Arc<RssService>) -> Self {
        Self { rss_service }
    }

    pub async fn import(&self, content: &str) -> Result<(Vec<Feed>, Vec<Category>)> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);

        let mut feeds = Vec::new();
        let mut categories = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) if e.name().as_ref() == b"outline" => {
                    if let Some(outline) = self.parse_outline(e) {
                        if let Some(xml_url) = outline.xml_url {
                            let feed = Feed::new(
                                outline.text,
                                Url::parse(&xml_url)?
                            );
                            if let Some(html_url) = outline.html_url {
                                if let Ok(site_url) = Url::parse(&html_url) {
                                    feed.with_site_url(site_url);
                                }
                            }
                            feeds.push(feed);
                        } else {
                            let category = Category::new(outline.text, None);
                            categories.push(category);
                        }
                    }
                }
                Event::Eof => break,
                _ => (),
            }
        }

        Ok((feeds, categories))
    }

    pub fn export(&self, feeds: &[Feed], categories: &[Category]) -> Result<String> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        // Write XML declaration
        writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            None,
        )))?;

        // Write OPML root element
        let mut opml = BytesStart::new("opml");
        opml.push_attribute(("version", "2.0"));
        writer.write_event(Event::Start(opml))?;

        // Write head element
        writer.write_event(Event::Start(BytesStart::new("head")))?;
        writer.write_event(Event::End(BytesStart::new("head")))?;

        // Write body element
        writer.write_event(Event::Start(BytesStart::new("body")))?;

        // Write categories
        for category in categories {
            let mut cat_el = BytesStart::new("outline");
            cat_el.push_attribute(("text", category.name.as_str()));
            writer.write_event(Event::Start(cat_el.clone()))?;
            writer.write_event(Event::End(cat_el))?;
        }

        // Write feeds
        for feed in feeds {
            let mut feed_el = BytesStart::new("outline");
            feed_el.push_attribute(("text", feed.title.as_str()));
            feed_el.push_attribute(("xmlUrl", feed.url.as_str()));
            if let Some(site_url) = &feed.site_url {
                feed_el.push_attribute(("htmlUrl", site_url.as_str()));
            }
            writer.write_event(Event::Empty(feed_el))?;
        }

        // Close body and opml elements
        writer.write_event(Event::End(BytesStart::new("body")))?;
        writer.write_event(Event::End(BytesStart::new("opml")))?;

        let result = String::from_utf8(writer.into_inner().into_inner())?;
        Ok(result)
    }

    fn parse_outline(&self, element: &BytesStart) -> Option<Outline> {
        let mut text = None;
        let mut xml_url = None;
        let mut html_url = None;

        for attr in element.attributes() {
            if let Ok(attr) = attr {
                match attr.key.as_ref() {
                    b"text" => text = String::from_utf8(attr.value.to_vec()).ok(),
                    b"xmlUrl" => xml_url = String::from_utf8(attr.value.to_vec()).ok(),
                    b"htmlUrl" => html_url = String::from_utf8(attr.value.to_vec()).ok(),
                    _ => (),
                }
            }
        }

        text.map(|t| Outline {
            text: t,
            xml_url,
            html_url,
        })
    }
}