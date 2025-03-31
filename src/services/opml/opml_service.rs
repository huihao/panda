use anyhow::Result;
use quick_xml::events::{BytesStart, Event};
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
                                )
                                .with_description(None)
                                .with_language(None);
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

    fn parse_outline(&self, e: &BytesStart) -> Option<OpmlOutline> {
        let mut text = None;
        let mut html_url = None;
        let mut xml_url = None;

        for attr in e.attributes().flatten() {
            match attr.key.as_ref() {
                b"text" => text = String::from_utf8(attr.value.into_owned()).ok(),
                b"htmlUrl" => html_url = String::from_utf8(attr.value.into_owned()).ok(),
                b"xmlUrl" => xml_url = String::from_utf8(attr.value.into_owned()).ok(),
                _ => (),
            }
        }

        text.map(|text| OpmlOutline {
            text,
            html_url,
            xml_url,
        })
    }

    pub fn export_opml(&self, feeds: &[Feed]) -> Result<String> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        
        // Write XML declaration and root element
        writer.write_event(Event::Start(BytesStart::new("opml")))?;
        
        // Write head element
        writer.write_event(Event::Start(BytesStart::new("head")))?;
        writer.write_event(Event::End(BytesStart::new("head")))?;
        
        // Write body element
        writer.write_event(Event::Start(BytesStart::new("body")))?;
        
        // Write feeds
        for feed in feeds {
            let mut outline = BytesStart::new("outline");
            outline.extend_attributes(vec![
                ("text", feed.title.as_str()),
                ("xmlUrl", feed.url.as_str()),
            ].into_iter());
            
            if let Some(site_url) = &feed.site_url {
                outline.extend_attributes(vec![("htmlUrl", site_url.as_str())]);
            }
            
            writer.write_event(Event::Empty(outline))?;
        }
        
        // Close body and opml elements
        writer.write_event(Event::End(BytesStart::new("body")))?;
        writer.write_event(Event::End(BytesStart::new("opml")))?;
        
        let result = writer.into_inner().into_inner();
        Ok(String::from_utf8(result)?)
    }
}