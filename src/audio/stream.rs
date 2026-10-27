//! HTTP audio streams (SHOUTcast/Icecast) with in-band ICY title metadata.

use std::io::{self, Read};
use std::sync::mpsc::Sender;

pub struct IcyReader<R: Read> {
    inner: R,
    metaint: Option<usize>,
    until_meta: usize,
    titles: Sender<String>,
}

impl<R: Read> IcyReader<R> {
    pub fn new(inner: R, metaint: Option<usize>, titles: Sender<String>) -> Self {
        IcyReader {
            inner,
            metaint,
            until_meta: metaint.unwrap_or(0),
            titles,
        }
    }

    fn read_metadata(&mut self) -> io::Result<()> {
        let mut len = [0u8; 1];
        self.inner.read_exact(&mut len)?;
        let n = len[0] as usize * 16;
        if n == 0 {
            return Ok(());
        }
        let mut buf = vec![0u8; n];
        self.inner.read_exact(&mut buf)?;
        let text = String::from_utf8_lossy(&buf);
        if let Some(t) = parse_stream_title(&text) {
            let _ = self.titles.send(t);
        }
        Ok(())
    }
}

pub fn parse_stream_title(meta: &str) -> Option<String> {
    let start = meta.find("StreamTitle='")? + "StreamTitle='".len();
    let rest = &meta[start..];
    let end = rest.find("';").or_else(|| rest.rfind('\''))?;
    let t = rest[..end].trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

impl<R: Read> Read for IcyReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Some(metaint) = self.metaint else {
            return self.inner.read(buf);
        };
        if self.until_meta == 0 {
            self.read_metadata()?;
            self.until_meta = metaint;
        }
        let want = buf.len().min(self.until_meta);
        let n = self.inner.read(&mut buf[..want])?;
        self.until_meta -= n;
        Ok(n)
    }
}

pub struct OpenedStream {
    pub reader: Box<dyn Read + Send + Sync>,
    pub name: Option<String>,
    pub content_type: Option<String>,
    pub bitrate: Option<u32>,
}

pub fn open(url: &str, titles: Sender<String>) -> Result<OpenedStream, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(10)))
        .build()
        .into();
    let resp = agent
        .get(url)
        .header("Icy-MetaData", "1")
        .header("User-Agent", concat!("Ferramp/", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let header = |name: &str| {
        resp.headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    };
    let metaint = header("icy-metaint").and_then(|v| v.trim().parse().ok());
    let name = header("icy-name");
    let content_type = header("content-type");
    let bitrate =
        header("icy-br").and_then(|v| v.split(',').next().and_then(|b| b.trim().parse().ok()));
    let body = resp.into_body().into_reader();
    Ok(OpenedStream {
        reader: Box::new(SyncReader(std::sync::Mutex::new(IcyReader::new(
            body, metaint, titles,
        )))),
        name,
        content_type,
        bitrate,
    })
}

/// Adapts a `Read + Send` into `Read + Send + Sync` for the decoder's source type.
struct SyncReader<R: Read + Send>(std::sync::Mutex<R>);

impl<R: Read + Send> Read for SyncReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0
            .get_mut()
            .map_err(|_| io::Error::other("poisoned"))?
            .read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_metadata_blocks() {
        let metaint = 4;
        let meta = b"StreamTitle='Artist - Song';";
        let blocks = meta.len().div_ceil(16);
        let mut data = b"abcd".to_vec();
        data.push(blocks as u8);
        let mut m = meta.to_vec();
        m.resize(blocks * 16, 0);
        data.extend_from_slice(&m);
        data.extend_from_slice(b"efgh");
        data.push(0);
        data.extend_from_slice(b"ij");
        let (tx, rx) = std::sync::mpsc::channel();
        let mut r = IcyReader::new(&data[..], Some(metaint), tx);
        let mut out = Vec::new();
        r.read_to_end(&mut out).unwrap();
        assert_eq!(out, b"abcdefghij");
        assert_eq!(rx.try_recv().unwrap(), "Artist - Song");
    }

    #[test]
    fn title_parsing() {
        assert_eq!(parse_stream_title("StreamTitle='';StreamUrl='';"), None);
        assert_eq!(
            parse_stream_title("StreamTitle='It's here';"),
            Some("It's here".into())
        );
    }
}
