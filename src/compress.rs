use super::_internals::*;

fn create_archive_with_index(
    src_dir: impl AsRef<Path>,
    output_zst: impl AsRef<Path>,
    dict: impl AsRef<Path>,
) -> Result<()> {
    let mut out = File::create(output_zst)?;
    let mut compressor = CCtx::create();
    // compressor.init(19).map_err(|s| anyhow!("could not init compressor : {s}"))?;
    let dict = File::open(dict)?
        .bytes()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    let dict = zstd_safe::CDict::create(&dict, 19);
    let dict_id = dict.get_dict_id().map(|v| v.get()).unwrap_or_default();
    debug!("dictionary id : {dict_id} = {dict_id:x}");
    let mut buf = Vec::with_capacity(2048);

    let mut paths = vec![];
    for entry in WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if !entry
                .file_name()
                .to_str()
                .map(|s| s.ends_with(".svg"))
                .unwrap_or_default()
            {
                continue;
            }
            let path = entry.path();
            let relative_path = path.strip_prefix(&src_dir)?.to_string_lossy().into_owned();
            paths.push(relative_path);
        }
    }
    // paths.sort();
    // debug!("sorted {} file paths : {}\n...", paths.len(), paths[..10].join("\n"));
    let mut index_map = FxHashMap::with_capacity_and_hasher(paths.len(), FxBuildHasher::default());
    let mut pos = 0; //out.stream_position()?;
    for path in paths.iter() {
        // debug!("compressing : {relative_path} to frame {frame_count}");
        let mut f = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&src_dir.as_ref().join(path))?;
        f.read_to_end(&mut buf)?;
        let uncompressed_size = buf.len();
        let mut obuf = Vec::with_capacity(2048);
        compressor
            .compress_using_cdict(&mut obuf, &buf, &dict)
            .map_err(|s| anyhow!("could not compress : {}", zstd_safe::get_error_name(s)))?;
        let frame_header = obuf[4]; // 1byte frame header descriptor
        assert!(
            frame_header == 99 || frame_header == 35,
            "unexpected zstd frame header:{} {}",
            frame_header,
            format!("{:?}", obuf)
        );
        // 99 = 0b01100011 = fcs_field_size=2, single segment=true, no checksum, dictionary id field size=4
        // 35 = 0b00100011 = fcs_field_size=0(if single segment then 1), single segment=true, no checksum, dictionary id field size=4
        // assert_eq!(frame_header, 99, "unexpected zstd frame header:{} {}", frame_header,  format!("{:?}", ocur.get_ref()));
        let fcs_field_size = frame_header >> 6; //
        // 0 => 1, // single segment makes this 1
        // 1 => 2,
        // 2 => 4,
        // 3 => 8,
        // _ => 0,
        assert!(
            fcs_field_size == 0 || fcs_field_size == 1,
            "too large icon :{path} ( size > 65791 )"
        );
        assert!(
            frame_header >> 5 & 1 == 1,
            "not single segment ( probably too large file )"
        );

        // window_descriptor (1 byte) is skipped by single segment mode, and window size = frame content size
        // let dict_id_written = u32::from_le_bytes(*obuf[5..9].as_array().unwrap()); // 4bytes dictionary id
        let dict_id_written = u32::from_le_bytes(obuf[5..9].try_into().unwrap()); // 4bytes dictionary id
        assert_eq!(
            dict_id, dict_id_written,
            "dict id mismatch : {dict_id_written}"
        );
        // fcs_field_size = 2 -> variable range 256 - 65791 ( offset 256 + 2bytes )
        let content_size = match fcs_field_size {
            0 => obuf[9] as u32,
            // 1 => u16::from_le_bytes(*obuf[9..11].as_array().unwrap()) as u32 + 256,
            1 => u16::from_le_bytes(obuf[9..11].try_into().unwrap()) as u32 + 256,
            // 2 => { u32::from_le_bytes(*obuf[9..13].as_array().unwrap())},
            _ => {
                panic!("icon {path} is too big")
            } // should not happen for small icons
        };
        assert_eq!(uncompressed_size as u32, content_size);
        out.write_all(&[frame_header])?; //ocur.get_ref()[4..5])?;
        out.write_all(&obuf[9..])?;
        let written_len = obuf.len() - 8; // magic_num: 4, dict_id: 4
        debug!("compressing {uncompressed_size} bytes from {path} to {written_len} bytes at {pos}");

        index_map.insert(&path.as_str()[..(path.len() - 4)], pos as u32);
        pos += written_len;
        buf.clear();
    }

    write_index_tree(out, &index_map)?;
    Ok(())
}

fn rebuild_dict(src_dir: impl AsRef<Path>, dict: impl AsRef<Path>) -> Result<()> {
    std::process::Command::new("bash")
        .args(&[
            "-c",
            &format!(
                "zstd --train --maxdict=2048 -o {} {}/*/*.svg",
                dict.as_ref().display(),
                src_dir.as_ref().display()
            ),
        ])
        .spawn()?
        .wait()?;
    Ok(())
}

fn write_index_tree(mut index_file: impl Write + Seek, map: &Map) -> Result<usize> {
    let start_pos = index_file.stream_position()?; // to calculate total tree size later

    let mut keys = map
        .keys()
        .map(|s| (*s, (map.hasher().hash_one(s) % 0xffffffff) as u32))
        .collect::<Vec<_>>();
    keys.sort_by_key(|(_key, hash)| *hash);
    debug!("{}", keys[0].0);

    let len = keys.len();
    let buf = (len as u32).to_le_bytes();
    index_file.write_all(&buf)?;
    debug!("key length {len} written to {start_pos}");

    let (keys, hashes): (Vec<_>, Vec<_>) = keys.into_iter().unzip();
    for hash in hashes {
        debug!("writing hash : {hash} @ {}", index_file.stream_position()?);
        index_file.write_all(&hash.to_le_bytes())?;
    }
    for key in keys {
        let v = map.get(key).unwrap();
        debug!(
            "writing value of {key} : {v} @ {}",
            index_file.stream_position()?
        );
        index_file.write_all(&v.to_le_bytes())?; // absolute position of archive bodies
    }
    let pos = index_file.stream_position()?;
    let tree_size = pos as u32 - start_pos as u32;
    debug!("writing tree size {tree_size} to {pos}");
    index_file.write_all(&tree_size.to_le_bytes())?; // write tree size
    Ok(tree_size as usize)
}

fn trim_svg_tags(src_dir: &str) -> Result<()> {
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file()
            || !entry
                .file_name()
                .to_str()
                .map(|s| s.ends_with(".svg"))
                .unwrap_or_default()
        {
            continue;
        }
        let path = entry.path();
        // let relative_path = path.strip_prefix(src_dir)?.to_string_lossy().into_owned();
        debug!("opening {}", path.display());
        let mut f = File::open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        drop(f);
        if s.starts_with("<svg") {
            if let Some(i) = s.find("viewBox=\"") {
                let _ = s.drain(..(i + 9)).collect::<String>();
            }
            assert_eq!('0', s.chars().nth(0).expect(""));
            assert_eq!(' ', s.chars().nth(1).expect(""));
            let _ = s.drain(..2).collect::<String>();
            let x = s.find('"').unwrap_or_default();
            if let Some(i) = s.find('>') {
                let _ = s.drain(x..=i).collect::<String>();
            }
            if let Some(i) = s.rfind("</svg>") {
                let _ = s.drain(i..).collect::<String>();
            }
            let s = s.lines().map(|s| s.trim()).collect::<Vec<_>>().join("");
            // debug!("{s}");
            std::fs::write(path, &s)?;
        }
    }
    Ok(())
}

pub fn compress(archive: impl AsRef<Path>, dict: impl AsRef<Path>) -> Result<()> {
    trim_svg_tags("./icons")?;
    rebuild_dict("./icons", &dict)?;
    create_archive_with_index("./icons", &archive, &dict)?;
    Ok(())
}
