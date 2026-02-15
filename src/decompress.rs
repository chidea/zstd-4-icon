use super::_internals::*;

fn get_raw_bytes_by_path(decompressor: &mut DCtx, dict_id: u32, archive_path: impl AsRef<Path>, target_path: &str) -> Result<Vec<u8>> {
    let mut archive_file = OpenOptions::new().read(true).write(false).create(false).open(archive_path)?;
    let frame_pos = get_frame_pos(&mut archive_file, target_path)?;
    let mock = mock_read_frame(&mut archive_file, frame_pos, dict_id)?;
    // std::fs::write("mocked.zst", &mock)?;
    let mut buf = Vec::<u8>::with_capacity(4000);
    //  `du -b icons/*/*.svg | sort -n -r | head` => 3681    icons/material-symbols/transition_fade.svg
    // decompressor.decompress_using_dict(&mut buf, &mock , &dict)
    decompressor.decompress(&mut buf, &mock)
        .map_err(|s| anyhow!("could not decompress : {}", zstd_safe::get_error_name(s)))?;
    Ok(buf.to_vec())
}

fn mock_read_frame(archive_file: &mut (impl Read + Seek), frame_pos: u32, dict_id: u32) -> Result<Vec<u8>> {
    let mut v = Vec::with_capacity(5);
    v.extend(ZSTD_MAGIC_NUM);
    // let mut v = ZSTD_MAGIC_NUM.to_vec();
    
    archive_file.seek(SeekFrom::Start(frame_pos as u64))?;
    let header = {
        let mut buf = [0u8; 1];
        archive_file.read_exact(&mut buf)?;
        buf[0]
    };
    v.push(header);
    v.extend(&dict_id.to_le_bytes());

    let _out_size = if header == 35 { // fcs_field_size 1
        let mut buf = [0u8; 1];
        archive_file.read_exact(&mut buf)?;
        v.push(buf[0]);
        buf[0] as u32
    } else if header == 99 { // fcs_field_size 2
        let mut buf = [0u8; 2];
        archive_file.read_exact(&mut buf)?;
        v.extend(&buf);
        u16::from_le_bytes(buf) as u32 + 256
    } else { return Err(anyhow!("unexpected frame header : {}", header)) };
    let size = {
        let mut buf = [0u8; 3];
        archive_file.read_exact(&mut buf)?;
        v.extend(&buf);
        debug!("block header : {buf:?}");
        // let is_last_block = buf[0] & 1;
        let block_type = (buf[0] >> 1) & 0b11;
        assert_eq!(block_type, 2, "unexpected block type : {block_type}");
        ((buf[0] >> 3) as u32) | ((buf[1] as u32) << 5) | (buf[2] as u32) << 13
    };
    
    debug!("frame size @ {frame_pos} : {size} - extract -> {_out_size}");
    // v.reserve(size);
    v.append(&mut archive_file.bytes().take(size as usize).filter_map(Result::ok).collect::<Vec<_>>());
    // debug!("mocked zstd frame : {v:?}");
    // std::fs::write("mocked.zst", &v)?;
    Ok(v)
}

fn read_tree(index_file: &mut (impl Read + Seek)) -> Result<(u32, u32)> {
    let mut buf = [0u8; SIZE_TREE_SIZE];

    let mut pos = index_file.seek(SeekFrom::End(-(SIZE_TREE_SIZE as i64)))? as u32; //last 4byte 4byte = kv size, tree size
    debug!("reading tree size at : {pos}");

    index_file.read_exact(&mut buf)?;
    let tree_size = u32::from_le_bytes(buf);
    debug!("tree size : {tree_size}");

    index_file.seek_relative(-(SIZE_TREE_SIZE as i64) - (tree_size as i64))?;
    pos -= tree_size;
    debug!("tree position: {pos}");
    
    index_file.read_exact(&mut buf)?;
    let tree_item_len = u32::from_le_bytes(buf);
    debug!("total item length : {tree_item_len}");

    pos += SIZE_TREE_LEN as u32;
    debug!("tree body position: {pos}");
    Ok((pos, tree_item_len))
}



fn get_frame_pos(index_file: &mut (impl Read + Seek), key: &str) -> Result<u32> {
    let key_hash = (FxBuildHasher::default().hash_one(&key) % 0xffffffff) as u32;

    let (pos, tree_item_len) = read_tree(index_file)?;

    let mut buf = [0u8; SIZE_HASH_VAL];
    // start in-file bisection search
    let (mut low, mut high) = (0u32, tree_item_len-1);
    let mut i = low + (high - low)/2;
    for _step in 0..32 { // enough step to search 32bit unsigned hashes
        if high < low {
            break
        }
        let _pos = index_file.seek(SeekFrom::Start((pos + i*SIZE_HASH_VAL as u32)as u64))?;
        index_file.read_exact(&mut buf)?;
        let v = u32::from_le_bytes(buf);
        debug!("searching {_step}th for {key_hash} in {low}-{i}({v})-{high}...");
        if v == key_hash {
            debug!("found hash tree match at {}", _pos-SIZE_HASH_VAL as u64);
            index_file.seek_relative((tree_item_len as i64 - 1) * SIZE_ABS_POS as i64)?; // jump to archive position page
            index_file.read_exact(&mut buf)?;
            let archive_pos = u32::from_le_bytes(buf);
            debug!("found frame position : {archive_pos}");
            return Ok(archive_pos)
        } else if v < key_hash {
            low = i + 1;
            debug!("new low : {low}");
        } else {
            high = i - 1;
            debug!("new high : {high}");
        }
        i = low + (high - low)/2;
    }
    Err(anyhow!("key {key} has not found in tree"))
}

pub fn new_dict_decompressor<'a>(dict_path: impl AsRef<Path>) -> Result<(DCtx<'a>, zstd_safe::DDict<'static>, u32)> {
    debug!("opening dictionary :{}", dict_path.as_ref().display());
    let mut dict = [0u8; 2048];
    let mut dict_file = File::open(dict_path)?;
    let _dict_size = dict_file.read(&mut dict)?;
    let dict_id = u32::from_le_bytes(*dict[4..8].as_array().expect("dictionary id parse error"));
    debug!("parsed dict size:{_dict_size}, id : {dict_id}");
    let ddict = zstd_safe::DDict::create(&dict);

    let mut decompressor = DCtx::create();
    // decompressor.load_dictionary(&dict)
    decompressor.ref_ddict(&ddict)
        .map_err(|s| anyhow!("could not load dictionary : {}", zstd_safe::get_error_name(s)))?;
    Ok((decompressor, ddict, dict_id))
}
pub fn new_decompressor_wrapper<'a>(dict_path: impl AsRef<Path>) -> Result<(DecompressorWrapper<'a>, zstd_safe::DDict<'static>)> {
    let (decompressor, ddict, dict_id) = new_dict_decompressor(dict_path)?;
    Ok((DecompressorWrapper(decompressor, dict_id), ddict))
}
// use zstd_safe::DDict;
pub struct DecompressorWrapper<'a>(DCtx<'a>, u32);
pub fn decompress_with_decompressor_wrapper<'a>(wrapper: &mut DecompressorWrapper<'a>, writer: &mut impl std::io::Write, archive: impl AsRef<Path>, key: &str) -> Result<()> {
    decompress_with_dict_decompressor(&mut wrapper.0, wrapper.1, writer, archive, key)
}
/// reuse decompressor and digested dictionary to make decoding even faster
pub fn decompress_with_dict_decompressor<'a>(decompressor: &mut DCtx<'a>, dict_id: u32, writer: &mut impl std::io::Write, archive: impl AsRef<Path>, key: &str) -> Result<()> {
    let mut raw_data = get_raw_bytes_by_path(decompressor, dict_id, archive, key)?.into_iter(); 
    // let mut cur = Cursor::new(raw_data).bytes().filter_map(Result::ok);
    writer.write_all(b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 ")?;
    
    // writer.write_all(&cur.by_ref().take_while(|c| *c != b'<').collect::<Vec<_>>())?;
    writer.write_all(&raw_data.by_ref().take_while(|c| *c != b'<').collect::<Vec<_>>())?;

    // append attributes to svg tag
    // TODO : better way to support any icon packs
    if key.starts_with("lucide") {
        writer.write_all(b"\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><")?;
    } else if key.starts_with("heroicons/") {
        writer.write_all(b"\" fill=\"none\"><")?;
    } else {
        writer.write_all(b"\"><")?;
    }

    // writer.write_all(&cur.collect::<Vec<_>>())?;
    writer.write_all(&raw_data.collect::<Vec<_>>())?;
    writer.write_all(b"</svg>")?;
    Ok(())
}

pub fn decompress(writer: &mut impl std::io::Write, archive: impl AsRef<Path>, dict: impl AsRef<Path>, key: &str) -> Result<()> {
    let (mut decompressor, _ddict, dict_id) = new_dict_decompressor(dict)?;
    Ok(decompress_with_dict_decompressor(&mut decompressor, dict_id, writer, archive, key)?)
    // let mut wrapper = new_decompressor_wrapper(dict)?;
    // Ok(decompress_with_decompressor_wrapper(&mut wrapper, writer, archive, key)?)
}


// optional hashmap preloading

pub struct PreloadedHashMap(FxHashMap<u32, u32>);
impl PreloadedHashMap {
    pub fn hash(&self, key: &str) -> u32 {
        (self.0.hasher().hash_one(key) % 0xffffffff) as u32
    }
    pub fn search(&self, key: &str) -> Option<&u32> {
        self.0.get(&self.hash(key))
    }
}
/// read entire tree from file to memory. optional utility function for even faster searching.
pub fn get_tree(index_file: &mut (impl Read + Seek)) -> Result<FxHashMap<u32,u32>> {
    let (_pos, tree_item_len) = read_tree(index_file)?;
    let mut buf = [0u8; SIZE_HASH_VAL];
    let mut hashes = Vec::<u32>::with_capacity(tree_item_len as usize);
    for _i in 0..tree_item_len {
        index_file.read_exact(&mut buf)?;
        let hash = u32::from_le_bytes(buf);
        hashes.push(hash);
    }
    let mut map = FxHashMap::<u32,u32>::with_capacity_and_hasher(tree_item_len as usize, FxBuildHasher::default());
    for hash in hashes {
        index_file.read_exact(&mut buf)?;
        let archive_pos = u32::from_le_bytes(buf);
        map.insert(hash, archive_pos);
    }
    Ok(map)
}
