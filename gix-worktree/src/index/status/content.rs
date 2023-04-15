use gix_features::hash;
use gix_hash::ObjectId;
use gix_index as index;
use gix_object::encode::loose_header;
use index::Entry;

use crate::index::status::ContentComparison;

/// Lazy borrwoed access to blob data from disk or ODB
pub trait LazyBlob<'a, E> {
    /// Returns the contents of this blob, potentially performs IO
    /// and other expensive operations and should only be called
    /// when necessary.
    fn read(self) -> Result<&'a [u8], E>;
}

/// Compares to blobs by comparing their size and oid, only looks at the file if
/// the size matches, therefore very fast
pub struct FastEq;

impl ContentComparison for FastEq {
    type Output = ();

    fn compare_blob<'a, E>(
        &self,
        entry: &'a Entry,
        blob_size: usize,
        blob: impl LazyBlob<'a, E>,
        _resolve_oid: impl FnMut(gix_hash::ObjectId) -> Result<&'a [u8], E>,
    ) -> Result<Option<Self::Output>, E> {
        // make sure to account for racily smudged entries here
        // so that they don't always keep showing up as modified even
        // after their contents have changed again (to a potentially unmodified state)
        // that means that we want to ignore stat.size == 0 for non_empty_blobs
        if entry.stat.size as usize != blob_size && (entry.id.is_empty_blob() || entry.stat.size != 0) {
            return Ok(Some(()));
        }
        let blob = blob.read()?;
        let header = loose_header(gix_object::Kind::Blob, blob.len());
        let mut hasher = hash::hasher(entry.id.kind());
        hasher.update(&header);
        hasher.update(blob);
        let file_hash: ObjectId = hasher.digest().into();
        Ok((entry.id != file_hash).then_some(()))
    }
}

/// Compares files to blobs by comparing their oids. Same as [`Fast`] but does
/// not contain a fast path for files with mismatched files and therefore always
/// returns an OID that can be reused later
pub struct HashEq;

impl ContentComparison for HashEq {
    type Output = ObjectId;

    fn compare_blob<'a, E>(
        &self,
        entry: &'a Entry,
        _blob_size: usize,
        blob: impl LazyBlob<'a, E>,
        _resolve_oid: impl FnMut(gix_hash::ObjectId) -> Result<&'a [u8], E>,
    ) -> Result<Option<Self::Output>, E> {
        let blob = blob.read()?;
        let header = loose_header(gix_object::Kind::Blob, blob.len());
        let mut hasher = hash::hasher(entry.id.kind());
        hasher.update(&header);
        hasher.update(blob);
        let file_hash: ObjectId = hasher.digest().into();
        Ok((entry.id != file_hash).then_some(file_hash))
    }
}