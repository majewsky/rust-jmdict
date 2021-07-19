/*******************************************************************************
* Copyright 2021 Stefan Majewsky <majewsky@gmx.net>
* SPDX-License-Identifier: Apache-2.0
* Refer to the file "LICENSE" for details.
*******************************************************************************/

use hex_literal::hex;
use std::path::PathBuf;

const ENTRYPACK_URL: &str = "https://dl.xyrillian.de/jmdict/entrypack-v1-2021-07-19.json.gz";
const ENTRYPACK_SHA256SUM: [u8; 32] =
    hex!("6d539f6b1841c213815ec9daa89bf9e5c1046e627f96db50ce800e995c1ca9ca");

pub struct EntryPack {
    pub path: PathBuf,
    pub sha256sum: Option<&'static [u8; 32]>,
}

impl EntryPack {
    pub fn locate_or_download() -> Self {
        match std::env::var_os("RUST_JMDICT_ENTRYPACK") {
            //download from hard-coded source if explicity requested
            Some(s) if s == "default" => Self {
                path: download_to_cache(ENTRYPACK_URL),
                sha256sum: Some(&ENTRYPACK_SHA256SUM),
            },
            //use override path if explicitly given
            Some(path_str) => Self {
                path: path_str.into(),
                sha256sum: None,
            },
            //default behavior: use file from repository for development builds, otherwise download
            //from hard-coded source
            None => {
                let local_path = std::path::Path::new("data/entrypack.json");
                if local_path.exists() {
                    Self {
                        path: local_path.into(),
                        sha256sum: None,
                    }
                } else {
                    Self {
                        path: download_to_cache(ENTRYPACK_URL),
                        sha256sum: Some(&ENTRYPACK_SHA256SUM),
                    }
                }
            }
        }
    }

    pub fn contents(&self) -> String {
        use libflate::gzip::Decoder;
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let data = std::fs::read(&self.path).unwrap();
        if let Some(expected_hash) = self.sha256sum {
            let hash = Sha256::digest(&data[..]);
            assert_eq!(&hash[..], expected_hash);
        }

        //check for GZip magic number
        if data[0] == 31 && data[1] == 139 {
            let mut decoder = Decoder::new(&data[..]).unwrap();
            let mut result = String::with_capacity(100 << 20);
            decoder.read_to_string(&mut result).unwrap();
            result
        } else {
            String::from_utf8(data).unwrap()
        }
    }
}

fn download_to_cache(url: &str) -> PathBuf {
    //construct path of the form "$HOME/.cache/rust-jmdict/entrypack-YYYY-MM-DD.json.gz"
    let base_dirs = directories::BaseDirs::new().unwrap();
    let mut path = PathBuf::new();
    path.push(base_dirs.cache_dir());
    path.push("rust-jmdict");
    std::fs::create_dir_all(&path).unwrap();
    let basename = url.rsplit('/').next().unwrap();
    path.push(&basename);

    //only need to download if not present yet
    if !path.exists() {
        //download with `curl`
        let status = std::process::Command::new("curl")
            .arg("--fail")
            .arg("--silent")
            .arg("--output")
            .arg(path.as_os_str())
            .arg(url)
            .status()
            .expect("failed to execute curl");
        assert!(status.success(), "{}", status);
    }

    path
}
