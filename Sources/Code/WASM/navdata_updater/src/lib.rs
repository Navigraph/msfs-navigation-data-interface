use std::fs;
use std::io;
use msfs::{commbus::*, network::*, MSFSEvent};

fn parse_json_args(args: &[u8]) -> Result<serde_json::Value, serde_json::Error> {
    let json_string = String::from_utf8(args.to_vec()).unwrap_or_default();
    let trimmed_string = json_string.trim_end_matches(char::from(0));
    let json: serde_json::Value = serde_json::from_str(&trimmed_string)?;
    Ok(json)
}

fn unzip_files<R>(mut zip_archive: zip::ZipArchive<R>, path_buf: std::path::PathBuf) where R: io::Read + io::Seek {
    for i in 0..zip_archive.len() {
        // create file
        let mut file = zip_archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path_buf.join(path.to_owned()),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    
    }
}

fn request_finished_callback(request: NetworkRequest, status_code: i32) {
    println!("[WASM] request finished with status code: {}", status_code);

    if status_code != 200 {
        println!("[WASM] request failed");
        return;
    }

    let path: &str = "\\work/navdata";
    match fs::create_dir_all(path) {
        Ok(_) => println!("[WASM] created directory"),
        Err(e) => {
            println!("[WASM] directory error: {}", e);
            return;
        },
    }

    let data = request.data().unwrap();
    let path_buf = std::path::PathBuf::from(path);

    // unzip
    let cursor = std::io::Cursor::new(data);
    let zip = zip::ZipArchive::new(cursor).unwrap();

    unzip_files(zip, path_buf);

    println!("[WASM] finished unzip");
    
    CommBus::call("NavdataDownloaded", &[], CommBusBroadcastFlags::JS);

}

fn download_navdata(args: &[u8]) {
    println!("[WASM] call received");
    let json_result = parse_json_args(args);
    match json_result {
        Ok(_) => println!("[WASM] json parsed"),
        Err(e) => {
            println!("[WASM] json error: {}", e);
            return;
        },
    }
    let json = json_result.unwrap();

    let url = json["url"].as_str().unwrap_or_default();

    println!("[WASM] url: {}", url);

    let response = NetworkRequestBuilder::new(url)
        .unwrap()
        .with_callback(request_finished_callback)
        .get()
        .unwrap();
}

#[msfs::gauge(name=navdata_updater)]
async fn navdata_updater(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    // We need to to save the commbus so that the callback is not dropped
    let mut commbus: CommBus;
    while let Some(event) = gauge.next_event().await {
        match event {
            MSFSEvent::PostInitialize => {
                println!("[WASM] initialized");
                let result = CommBus::register("DownloadNavdata", download_navdata);
                // set the mut commbus variable
                if result.is_some() {
                    commbus = result.unwrap();
                }
            }
            _ => {}
        }
    }

    Ok(())
}