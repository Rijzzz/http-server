use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::{
    Request, Response, StatusCode, detect_mime, gzip_compress, is_compressible, sanitize_path,
};

pub struct StaticFileServer {
    root_dir: PathBuf,
}

impl StaticFileServer {
    #[must_use]
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root_dir: root.as_ref().to_path_buf(),
        }
    }

    #[must_use]
    pub fn serve(&self, req: &Request) -> Response {
        let safe_rel = sanitize_path(&req.path);
        let file_path = self.root_dir.join(safe_rel);

        if !file_path.is_file() {
            return Response::not_found();
        }

        let Ok(mut file) = File::open(&file_path) else {
            return Response::not_found();
        };

        let Ok(metadata) = file.metadata() else {
            return Response::new(StatusCode::InternalServerError);
        };

        let Ok(file_len) = usize::try_from(metadata.len()) else {
            return Response::new(StatusCode::PayloadTooLarge);
        };

        let mod_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.elapsed().ok())
            .map_or(0, |d| d.as_secs());
        let etag = format!("\"{file_len:x}-{mod_time:x}\"");

        if let Some(if_none_match) = req.headers.get("if-none-match")
            && if_none_match.trim() == etag
        {
            let mut res = Response::new(StatusCode::NotModified);
            res.header("etag", &etag);
            return res;
        }

        let mut data = Vec::with_capacity(file_len);
        if file.read_to_end(&mut data).is_err() {
            return Response::new(StatusCode::InternalServerError);
        }

        let mime = detect_mime(&file_path);

        if let Some(range_header) = req.headers.get("range")
            && let Some((start, end)) = parse_range(range_header, file_len)
        {
            let slice = &data[start..=end];
            let mut res = Response::bytes(StatusCode::PartialContent, mime, slice.to_vec());
            res.header("content-range", &format!("bytes {start}-{end}/{file_len}"));
            res.header("etag", &etag);
            res.header("accept-ranges", "bytes");
            return res;
        }

        let accepts_gzip = req
            .headers
            .get("accept-encoding")
            .is_some_and(|enc| enc.contains("gzip"));

        if accepts_gzip && is_compressible(mime) && data.len() > 128 {
            let compressed = gzip_compress(&data);
            let mut res = Response::bytes(StatusCode::Ok, mime, compressed);
            res.header("content-encoding", "gzip");
            res.header("etag", &etag);
            res.header("accept-ranges", "bytes");
            res
        } else {
            let mut res = Response::bytes(StatusCode::Ok, mime, data);
            res.header("etag", &etag);
            res.header("accept-ranges", "bytes");
            res
        }
    }
}

fn parse_range(header: &str, file_len: usize) -> Option<(usize, usize)> {
    let range_str = header.strip_prefix("bytes=")?;
    let (start_str, end_str) = range_str.split_once('-')?;

    let start = start_str.parse::<usize>().ok()?;
    let end = if end_str.is_empty() {
        file_len.saturating_sub(1)
    } else {
        end_str
            .parse::<usize>()
            .ok()?
            .min(file_len.saturating_sub(1))
    };

    if start <= end && start < file_len {
        Some((start, end))
    } else {
        None
    }
}
