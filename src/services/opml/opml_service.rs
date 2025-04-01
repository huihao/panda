use anyhow::Result;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use url::Url;
use std::io::Cursor;
use crate::models::feed::{Feed, FeedId, FeedStatus};
use crate::services::rss::RssService;
use std::sync::Arc;

#[derive(Debug)]
struct OpmlOutline {
    text: String,
    html_url: Option<String>,
    xml_url: Option<String>,
}

pub struct OpmlService {
    rss_service: Arc<RssService>,
}

impl OpmlService {
    pub fn new(rss_service: Arc<RssService>) -> Self {
        Self { rss_service }
    }

    pub fn import_opml(&self, content: &str) -> Result<Vec<Feed>> {
        let mut reader = Reader::from_str(content);
        reader.trim_text(true);
        
        let mut feeds = Vec::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                    if e.name().as_ref() == b"outline" {
                        if let Some(outline) = self.parse_outline(&e) {
                            if let Some(xml_url) = outline.xml_url {
                                let feed = Feed::new(
                                    outline.text,
                                    Url::parse(&xml_url)?
                                );
                                feeds.push(feed);
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("Error at position {}: {:?}", reader.buffer_position(), e)),
                _ => (),
            }
            buf.clear();
        }
        
        Ok(feeds)
    }

    fn parse_outline(&self, element: &BytesStart) -> Option<OpmlOutline> {
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
        
        text.map(|t| OpmlOutline {
            text: t,
            xml_url,
            html_url,
        })
    }
}