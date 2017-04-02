use super::block_split::BlockSplit;
use super::command::{Command, NewCommand};
use super::constants::{kInsBase, kInsExtra, kCopyBase, kCopyExtra};
use super::static_dict::{BROTLI_UNALIGNED_LOAD32, BROTLI_UNALIGNED_LOAD64, FindMatchLengthWithLimit};
use super::static_dict::BrotliDictionary;
use super::super::alloc;
use super::super::alloc::{SliceWrapper, SliceWrapperMut};
use super::util::Log2FloorNonZero;
use core;
static kBrotliMinWindowBits: i32 = 10i32;

static kBrotliMaxWindowBits: i32 = 24i32;

static kInvalidMatch: u32 = 0xfffffffu32;

static kCutoffTransformsCount: u32 = 10u32;

static kCutoffTransforms: u64 = 0x71b520au64 << 32i32 | 0xda2d3200u32 as (u64);

static kHashMul32: u32 = 0x1e35a7bdu32;

static kHashMul64: u64 = 0x1e35a7bdu64 << 32i32 | 0x1e35a7bdu64;

static kHashMul64Long: u64 = 0x1fe35a7bu32 as (u64) << 32i32 | 0xd3579bd3u32 as (u64);



pub enum BrotliEncoderMode {
  BROTLI_MODE_GENERIC = 0,
  BROTLI_MODE_TEXT = 1,
  BROTLI_MODE_FONT = 2,
}



pub struct BrotliHasherParams {
  pub type_: i32,
  pub bucket_bits: i32,
  pub block_bits: i32,
  pub hash_len: i32,
  pub num_last_distances_to_check: i32,
}



pub struct BrotliEncoderParams {
  pub mode: BrotliEncoderMode,
  pub quality: i32,
  pub lgwin: i32,
  pub lgblock: i32,
  pub size_hint: usize,
  pub disable_literal_context_modeling: i32,
  pub hasher: BrotliHasherParams,
}



fn StoreLookaheadH2() -> usize {
  8usize
}

fn LiteralSpreeLengthForSparseSearch(mut params: &BrotliEncoderParams) -> usize {
  (if (*params).quality < 9i32 {
     64i32
   } else {
     512i32
   }) as (usize)
}

fn PrepareDistanceCacheH2(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  handle;
  distance_cache;
}

fn HashTypeLengthH2() -> usize {
  8usize
}

fn brotli_min_size_t(mut a: usize, mut b: usize) -> usize {
  if a < b { a } else { b }
}



pub struct HasherSearchResult {
  pub len: usize,
  pub len_x_code: usize,
  pub distance: usize,
  pub score: usize,
}

pub struct Struct1 {
  pub params: BrotliHasherParams,
  pub is_prepared_: i32,
  pub dict_num_lookups: usize,
  pub dict_num_matches: usize,
}

trait AnyHasher {
  fn GetHasherCommon(&mut self) -> &mut Struct1;
  fn HashBytes(&self, data: &[u8]) -> usize;
  fn FindLongestMatch(&mut self,
                      mut dictionary: &BrotliDictionary,
                      mut dictionary_hash: &[u16],
                      mut data: &[u8],
                      ring_buffer_mask: usize,
                      mut distance_cache: &[i32],
                      cur_ix: usize,
                      max_length: usize,
                      max_backward: usize,
                      mut out: &mut HasherSearchResult)
                      -> bool;
}
pub trait BasicHashComputer {
  fn HashBytes(&self, data: &[u8]) -> u32;
  fn BUCKET_BITS(&self) -> i32;
  fn USE_DICTIONARY(&self) -> i32;
  fn BUCKET_SWEEP(&self) -> i32;
}
pub struct BasicHasher<Buckets: SliceWrapperMut<u32> + SliceWrapper<u32> + BasicHashComputer> {
  pub GetHasherCommon: Struct1,
  pub buckets_: Buckets,
}
pub struct H2Sub {
  pub buckets_: [u32; 65537],
}
impl<T: SliceWrapperMut<u32> + SliceWrapper<u32> + BasicHashComputer> AnyHasher for BasicHasher<T> {
  fn GetHasherCommon(&mut self) -> &mut Struct1 {
    return &mut self.GetHasherCommon;
  }
  fn HashBytes(&self, data: &[u8]) -> usize {
    self.buckets_.HashBytes(data) as usize
  }
  fn FindLongestMatch(&mut self,
                      mut dictionary: &BrotliDictionary,
                      mut dictionary_hash: &[u16],
                      mut data: &[u8],
                      ring_buffer_mask: usize,
                      mut distance_cache: &[i32],
                      cur_ix: usize,
                      max_length: usize,
                      max_backward: usize,
                      mut out: &mut HasherSearchResult)
                      -> bool {
    let best_len_in: usize = (*out).len;
    let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
    let key: u32 = self.HashBytes(&data[(cur_ix_masked as (usize))..]) as u32;
    let mut compare_char: i32 = data[(cur_ix_masked.wrapping_add(best_len_in) as (usize))] as (i32);
    let mut best_score: usize = (*out).score;
    let mut best_len: usize = best_len_in;
    let mut cached_backward: usize = distance_cache[(0usize)] as (usize);
    let mut prev_ix: usize = cur_ix.wrapping_sub(cached_backward);
    let mut is_match_found: i32 = 0i32;
    (*out).len_x_code = 0usize;
    if prev_ix < cur_ix {
      prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
      if compare_char == data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
        let mut len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))..],
                                                      &data[(cur_ix_masked as (usize))..],
                                                      max_length);
        if len >= 4usize {
          best_score = BackwardReferenceScoreUsingLastDistance(len);
          best_len = len;
          (*out).len = len;
          (*out).distance = cached_backward;
          (*out).score = best_score;
          compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
          if self.buckets_.BUCKET_SWEEP() == 1i32 {
            (*self).buckets_.slice_mut()[key as (usize)] = cur_ix as (u32);
            return true;
          } else {
            is_match_found = 1i32;
          }
        }
      }
    }
    let BUCKET_SWEEP = self.buckets_.BUCKET_SWEEP();
    if BUCKET_SWEEP == 1i32 {
      let mut backward: usize;
      let mut len: usize;
      prev_ix = (*self).buckets_.slice()[key as (usize)] as (usize);
      (*self).buckets_.slice_mut()[key as (usize)] = cur_ix as (u32);
      backward = cur_ix.wrapping_sub(prev_ix);
      prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
      if compare_char != data[(prev_ix.wrapping_add(best_len_in) as (usize))] as (i32) {
        return false;
      }
      if backward == 0usize || backward > max_backward {
        return false;
      }
      len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))..],
                                     &data[(cur_ix_masked as (usize))..],
                                     max_length);
      if len >= 4usize {
        (*out).len = len;
        (*out).distance = backward;
        (*out).score = BackwardReferenceScore(len, backward);
        return true;
      }
    } else {
      let (old_, mut bucket) = (*self).buckets_.slice_mut()[key as usize..].split_at_mut(1);
      let mut i: i32;
      prev_ix = old_[0] as (usize);
      i = 0i32;

      while i < BUCKET_SWEEP {
        'continue3: loop {
          {
            let backward: usize = cur_ix.wrapping_sub(prev_ix);
            let mut len: usize;
            prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
            if compare_char != data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
              {
                break 'continue3;
              }
            }
            if backward == 0usize || backward > max_backward {
              {
                break 'continue3;
              }
            }
            len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))..],
                                           &data[(cur_ix_masked as (usize))..],
                                           max_length);
            if len >= 4usize {
              let score: usize = BackwardReferenceScore(len, backward);
              if best_score < score {
                best_score = score;
                best_len = len;
                (*out).len = best_len;
                (*out).distance = backward;
                (*out).score = score;
                compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
                is_match_found = 1i32;
              }
            }
          }
          break;
        }
        i = i + 1;
        {
          let (_old, new_bucket) = core::mem::replace(&mut bucket, &mut []).split_at_mut(1);
          prev_ix = _old[0] as usize;
          bucket = new_bucket;
        }
      }
    }
    if self.buckets_.USE_DICTIONARY() != 0 && (is_match_found == 0) {
      is_match_found = SearchInStaticDictionary(dictionary,
                                                dictionary_hash,
                                                self,
                                                &data[(cur_ix_masked as (usize))..],
                                                max_length,
                                                max_backward,
                                                out,
                                                1i32);
    }
    (*self).buckets_.slice_mut()[(key as (usize)).wrapping_add((cur_ix >> 3).wrapping_rem(self.buckets_.BUCKET_SWEEP() as
                                                             usize))] = cur_ix as (u32);
    is_match_found != 0

  }
}
impl BasicHashComputer for H2Sub {
  fn HashBytes(&self, data: &[u8]) -> u32 {
    let h: u64 = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 5i32).wrapping_mul(kHashMul64);
    (h >> 64i32 - 16i32) as (u32)
  }
  fn BUCKET_BITS(&self) -> i32 {
    16
  }
  fn BUCKET_SWEEP(&self) -> i32 {
    1
  }
  fn USE_DICTIONARY(&self) -> i32 {
    1
  }
}
impl SliceWrapperMut<u32> for H2Sub {
  fn slice_mut(&mut self) -> &mut [u32] {
    return &mut self.buckets_[..];
  }
}
impl SliceWrapper<u32> for H2Sub {
  fn slice(&self) -> &[u32] {
    return &self.buckets_[..];
  }
}
pub struct H3Sub {
  pub buckets_: [u32; 65538],
}
impl SliceWrapperMut<u32> for H3Sub {
  fn slice_mut(&mut self) -> &mut [u32] {
    return &mut self.buckets_[..];
  }
}
impl SliceWrapper<u32> for H3Sub {
  fn slice(&self) -> &[u32] {
    return &self.buckets_[..];
  }
}
impl BasicHashComputer for H3Sub {
  fn BUCKET_BITS(&self) -> i32 {
    16
  }
  fn BUCKET_SWEEP(&self) -> i32 {
    2
  }
  fn USE_DICTIONARY(&self) -> i32 {
    0
  }
  fn HashBytes(&self, data: &[u8]) -> u32 {
    let h: u64 = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 5i32).wrapping_mul(kHashMul64);
    (h >> 64i32 - 16i32) as (u32)
  }
}
pub struct H4Sub {
  pub buckets_: [u32; 131076],
}
impl BasicHashComputer for H4Sub {
  fn BUCKET_BITS(&self) -> i32 {
    17
  }
  fn BUCKET_SWEEP(&self) -> i32 {
    4
  }
  fn USE_DICTIONARY(&self) -> i32 {
    1
  }
  fn HashBytes(&self, data: &[u8]) -> u32 {
    let h: u64 = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 5i32).wrapping_mul(kHashMul64);
    (h >> 64i32 - 17i32) as (u32)
  }
}
impl SliceWrapperMut<u32> for H4Sub {
  fn slice_mut(&mut self) -> &mut [u32] {
    return &mut self.buckets_[..];
  }
}
impl SliceWrapper<u32> for H4Sub {
  fn slice(&self) -> &[u32] {
    return &self.buckets_[..];
  }
}
pub struct H54Sub {
  pub buckets_: [u32; 1048580],
}
impl BasicHashComputer for H54Sub {
  fn BUCKET_BITS(&self) -> i32 {
    20
  }
  fn BUCKET_SWEEP(&self) -> i32 {
    4
  }
  fn USE_DICTIONARY(&self) -> i32 {
    0
  }
  fn HashBytes(&self, data: &[u8]) -> u32 {
    let h: u64 = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 7i32).wrapping_mul(kHashMul64);
    (h >> 64i32 - 20i32) as (u32)
  }
}

impl SliceWrapperMut<u32> for H54Sub {
  fn slice_mut(&mut self) -> &mut [u32] {
    return &mut self.buckets_[..];
  }
}
impl SliceWrapper<u32> for H54Sub {
  fn slice(&self) -> &[u32] {
    return &self.buckets_[..];
  }
}
pub trait AdvHashSpecialization {
  fn get_hash_mask(&self) -> u64;
  fn set_hash_mask(&mut self, params_hash_len: i32);
  fn HashTypeLength(&self) -> usize;
  fn StoreLookahead(&self) -> usize;
}

pub struct AdvHasher<Specialization: AdvHashSpecialization + Sized,
                     AllocU16: alloc::Allocator<u16>,
                     AllocU32: alloc::Allocator<u32>>
{
  pub GetHasherCommon: Struct1,
  pub bucket_size_: u64,
  pub block_size_: u64,
  pub specialization: Specialization, // contains hash_mask_
  pub hash_shift_: i32,
  pub block_mask_: u32,
  pub num: AllocU16::AllocatedMemory,
  pub buckets: AllocU32::AllocatedMemory,
}
pub struct H5Sub {
}
impl AdvHashSpecialization for H5Sub {
  fn get_hash_mask(&self) -> u64 {
     return 0xffffffffffffffffu64;
  }
  fn set_hash_mask(&mut self, params_hash_len: i32){}
  fn HashTypeLength(&self) -> usize {4}
  fn StoreLookahead(&self) -> usize {4}
}

pub struct H6Sub {
  hash_mask :u64,
}

impl AdvHashSpecialization for H6Sub {
  fn get_hash_mask(&self) -> u64 {
     self.hash_mask
  }
  fn set_hash_mask(&mut self, params_hash_len: i32){
     self.hash_mask = !(0u32 as (u64)) >> 64i32 - 8i32 * params_hash_len;
  }
  fn HashTypeLength(&self) -> usize {8}
  fn StoreLookahead(&self) -> usize {8}
}

fn BackwardReferencePenaltyUsingLastDistance(mut distance_short_code: usize) -> usize {
  (39usize).wrapping_add((0x1ca10i32 >> (distance_short_code & 0xeusize) & 0xei32) as (usize))
}


impl<Specialization: AdvHashSpecialization, AllocU16: alloc::Allocator<u16>, AllocU32: alloc::Allocator<u32>> AnyHasher
  for AdvHasher<Specialization, AllocU16, AllocU32> {
  fn GetHasherCommon(&mut self) -> &mut Struct1 {
    return &mut self.GetHasherCommon;
  }
  fn HashBytes(&self, mut data: &[u8]) -> usize {
    let mask = self.specialization.get_hash_mask();
    let shift = self.hash_shift_;
    let h: u64 = (BROTLI_UNALIGNED_LOAD64(data) & mask).wrapping_mul(kHashMul64Long);
    (h >> shift) as (u32) as usize
  }
  fn FindLongestMatch(&mut self,
                      mut dictionary: &BrotliDictionary,
                      mut dictionary_hash: &[u16],
                      mut data: &[u8],
                      ring_buffer_mask: usize,
                      mut distance_cache: &[i32],
                      cur_ix: usize,
                      max_length: usize,
                      max_backward: usize,
                      mut out: &mut HasherSearchResult)
                      -> bool {
    let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
    let mut is_match_found: i32 = 0i32;
    let mut best_score: usize = (*out).score;
    let mut best_len: usize = (*out).len;
    let mut i: usize;
    (*out).len = 0usize;
    (*out).len_x_code = 0usize;
    i = 0usize;
    while i < self.GetHasherCommon.params.num_last_distances_to_check as (usize) {
      'continue45: loop {
        {
          let backward: usize = distance_cache[(i as (usize))] as (usize);
          let mut prev_ix: usize = cur_ix.wrapping_sub(backward);
          if prev_ix >= cur_ix {
            {
              break 'continue45;
            }
          }
          if backward > max_backward {
            {
              break 'continue45;
            }
          }
          prev_ix = prev_ix & ring_buffer_mask;
          if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask || prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
             data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
             data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
            {
              break 'continue45;
            }
          }
          {
            let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))..],
                                                      &data[(cur_ix_masked as (usize))..],
                                                      max_length);
            if len >= 3usize || len == 2usize && (i < 2usize) {
              let mut score: usize = BackwardReferenceScoreUsingLastDistance(len);
              if best_score < score {
                if i != 0usize {
                  score = score.wrapping_sub(BackwardReferencePenaltyUsingLastDistance(i));
                }
                if best_score < score {
                  best_score = score;
                  best_len = len;
                  (*out).len = best_len;
                  (*out).distance = backward;
                  (*out).score = best_score;
                  is_match_found = 1i32;
                }
              }
            }
          }
        }
        break;
      }
      i = i.wrapping_add(1 as (usize));
    }
    {
      let key: u32 = self.HashBytes(&data[(cur_ix_masked as (usize))..]) as u32;
      let common_block_bits = self.GetHasherCommon.params.block_bits;
      let mut bucket: &mut [u32] = &mut self.buckets.slice_mut()[((key << common_block_bits) as (usize))..];
      let down: usize = if self.num.slice()[(key as (usize))] as (u64) > (*self).block_size_ {
        (self.num.slice()[(key as (usize))] as (u64)).wrapping_sub((*self).block_size_) as usize
      } else {
        0u32 as (usize)
      };
      i = self.num.slice()[(key as (usize))] as (usize);
      while i > down {
        let mut prev_ix: usize = bucket[(({
            i = i.wrapping_sub(1 as (usize));
            i
          } & (*self).block_mask_ as (usize)) as (usize))] as (usize);
        let backward: usize = cur_ix.wrapping_sub(prev_ix);
        if backward > max_backward {
          {
            break;
          }
        }
        prev_ix = prev_ix & ring_buffer_mask;
        if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask || prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
           data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
           data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
          {
            continue;
          }
        }
        {
          let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))..],
                                                    &data[(cur_ix_masked as (usize))..],
                                                    max_length);
          if len >= 4usize {
            let mut score: usize = BackwardReferenceScore(len, backward);
            if best_score < score {
              best_score = score;
              best_len = len;
              (*out).len = best_len;
              (*out).distance = backward;
              (*out).score = best_score;
              is_match_found = 1i32;
            }
          }
        }
      }
      bucket[((self.num.slice()[(key as (usize))] as (u32) & (self).block_mask_) as (usize))] = cur_ix as (u32);
      {
        let _rhs = 1;
        let _lhs = &mut self.num.slice_mut()[(key as (usize))];
        *_lhs = (*_lhs as (i32) + _rhs) as (u16);
      }
    }
    if is_match_found == 0 {
      is_match_found = SearchInStaticDictionary(dictionary,
                                                dictionary_hash,
                                                self,
                                                &data[(cur_ix_masked as (usize))..],
                                                max_length,
                                                max_backward,
                                                out,
                                                0i32);
    }
    is_match_found != 0

  }
}


pub struct BankH40 {
  pub slots: [SlotH40; 65536],
}

pub struct BankH41 {
  pub slots: [SlotH41; 65536],
}

pub struct BankH42 {
  pub slots: [SlotH42; 512],
}


pub struct SlotH40 {
  pub delta: u16,
  pub next: u16,
}
pub struct SlotH41 {
  pub delta: u16,
  pub next: u16,
}

pub struct SlotH42 {
  pub delta: u16,
  pub next: u16,
}

// UNSUPPORTED, for now.
pub struct H40 {
  pub common: Struct1,
  pub addr: [u32; 32768],
  pub head: [u16; 32768],
  pub tiny_hash: [u8; 65536],
  pub banks: [BankH40; 1],
  pub free_slot_idx: [u16; 1],
  pub max_hops: usize,
}


pub struct H41 {
  pub common: Struct1,
  pub addr: [u32; 32768],
  pub head: [u16; 32768],
  pub tiny_hash: [u8; 65536],
  pub banks: [BankH41; 1],
  pub free_slot_idx: [u16; 1],
  pub max_hops: usize,
}

pub struct H42 {
  pub common: Struct1,
  pub addr: [u32; 32768],
  pub head: [u16; 32768],
  pub tiny_hash: [u8; 65536],
  pub banks: [BankH42; 512],
  free_slot_idx: [u16; 512],
  pub max_hops: usize,
}




fn HashBytesH2(mut data: &[u8]) -> u32 {
  let h: u64 = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 5i32).wrapping_mul(kHashMul64);
  (h >> 64i32 - 16i32) as (u32)
}

fn unopt_ctzll(mut val: usize) -> u8 {
  let mut cnt: u8 = 0i32 as (u8);
  while val & 1usize == 0usize {
    val = val >> 1i32;
    cnt = (cnt as (i32) + 1) as (u8);
  }
  cnt
}


fn BackwardReferenceScoreUsingLastDistance(mut copy_length: usize) -> usize {
  (135usize)
    .wrapping_mul(copy_length)
    .wrapping_add(((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()))
    .wrapping_add(15usize)
}


fn BackwardReferenceScore(mut copy_length: usize, mut backward_reference_offset: usize) -> usize {
  ((30i32 * 8i32) as (usize))
    .wrapping_mul(::core::mem::size_of::<usize>())
    .wrapping_add((135usize).wrapping_mul(copy_length))
    .wrapping_sub((30u32).wrapping_mul(Log2FloorNonZero(backward_reference_offset)) as (usize))
}

fn Hash14(mut data: &[u8]) -> u32 {
  let mut h: u32 = BROTLI_UNALIGNED_LOAD32(data).wrapping_mul(kHashMul32);
  h >> 32i32 - 14i32
}

fn TestStaticDictionaryItem(mut dictionary: &BrotliDictionary,
                            mut item: usize,
                            mut data: &[u8],
                            mut max_length: usize,
                            mut max_backward: usize,
                            mut out: &mut HasherSearchResult)
                            -> i32 {
  let mut len: usize;
  let mut dist: usize;
  let mut offset: usize;
  let mut matchlen: usize;
  let mut backward: usize;
  let mut score: usize;
  len = item & 0x1fusize;
  dist = item >> 5i32;
  offset = ((*dictionary).offsets_by_length[len] as (usize)).wrapping_add(len.wrapping_mul(dist));
  if len > max_length {
    return 0i32;
  }
  matchlen = FindMatchLengthWithLimit(data, &(*dictionary).data[offset..], len);
  if matchlen.wrapping_add(kCutoffTransformsCount as usize) <= len || matchlen == 0usize {
    return 0i32;
  }
  {
    let mut cut: usize = len.wrapping_sub(matchlen);
    let mut transform_id: usize =
      (cut << 2i32).wrapping_add(kCutoffTransforms as usize >> cut.wrapping_mul(6) & 0x3f);
    backward = max_backward.wrapping_add(dist)
      .wrapping_add(1usize)
      .wrapping_add(transform_id << (*dictionary).size_bits_by_length[len] as (i32));
  }
  score = BackwardReferenceScore(matchlen, backward);
  if score < (*out).score {
    return 0i32;
  }
  (*out).len = matchlen;
  (*out).len_x_code = len ^ matchlen;
  (*out).distance = backward;
  (*out).score = score;
  1i32
}

fn SearchInStaticDictionary<HasherType: AnyHasher>(mut dictionary: &BrotliDictionary,
                                                   mut dictionary_hash: &[u16],
                                                   mut handle: &mut HasherType,
                                                   mut data: &[u8],
                                                   mut max_length: usize,
                                                   mut max_backward: usize,
                                                   mut out: &mut HasherSearchResult,
                                                   mut shallow: i32)
                                                   -> i32 {
  let mut key: usize;
  let mut i: usize;
  let mut is_match_found: i32 = 0i32;
  let mut xself: &mut Struct1 = handle.GetHasherCommon();
  if (*xself).dict_num_matches < (*xself).dict_num_lookups >> 7i32 {
    return 0i32;
  }
  key = (Hash14(data) << 1i32) as (usize);
  i = 0usize;
  while i < if shallow != 0 { 1u32 } else { 2u32 } as (usize) {
    {
      let mut item: usize = dictionary_hash[(key as (usize))] as (usize);
      (*xself).dict_num_lookups = (*xself).dict_num_lookups.wrapping_add(1 as (usize));
      if item != 0usize {
        let mut item_matches: i32 =
          TestStaticDictionaryItem(dictionary, item, data, max_length, max_backward, out);
        if item_matches != 0 {
          (*xself).dict_num_matches = (*xself).dict_num_matches.wrapping_add(1 as (usize));
          is_match_found = 1i32;
        }
      }
    }
    i = i.wrapping_add(1 as (usize));
    key = key.wrapping_add(1 as (usize));
  }
  is_match_found
}


/*

fn StoreH2(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let key: u32 = HashBytesH2(&data[((ix & mask) as (usize))]);
  let off: u32 = (ix >> 3i32).wrapping_rem(xself.BUCKET_SWEEP()) as (u32);
  (*SelfH2(handle)).buckets_[key.wrapping_add(off) as (usize)] = ix as (u32);
}

fn StoreRangeH2(mut handle: &mut [u8],
                mut data: &[u8],
                mask: usize,
                ix_start: usize,
                ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH2(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn brotli_max_size_t(mut a: usize, mut b: usize) -> usize {
  if a > b { a } else { b }
}

fn CreateBackwardReferencesH2(mut dictionary: &[BrotliDictionary],
                              mut dictionary_hash: &[u16],
                              mut num_bytes: usize,
                              mut position: usize,
                              mut ringbuffer: &[u8],
                              mut ringbuffer_mask: usize,
                              mut params: &[BrotliEncoderParams],
                              mut hasher: &mut [u8],
                              mut dist_cache: &mut [i32],
                              mut last_insert_len: &mut [usize],
                              mut commands: &mut [Command],
                              mut num_commands: &mut [usize],
                              mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH2() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH2()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH2(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH2()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH2(hasher,
                          dictionary,
                          dictionary_hash,
                          ringbuffer,
                          ringbuffer_mask,
                          dist_cache,
                          position,
                          max_length,
                          max_distance,
                          &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break6: loop {
        'continue7: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH2(hasher,
                                                dictionary,
                                                dictionary_hash,
                                                ringbuffer,
                                                ringbuffer_mask,
                                                dist_cache,
                                                position.wrapping_add(1usize),
                                                max_length,
                                                max_distance,
                                                &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH2()) < pos_end) {
                {
                  break 'continue7;
                }
              }
            }
            {
              {
                break 'break6;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH2(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH2(hasher,
                   ringbuffer,
                   ringbuffer_mask,
                   position.wrapping_add(2usize),
                   brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH2().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH2(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH2().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH2(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH3() -> usize {
  8usize
}

fn PrepareDistanceCacheH3(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  handle;
  distance_cache;
}

fn HashTypeLengthH3() -> usize {
  8usize
}



pub struct H3 {
  pub buckets_: [u32; 65538],
}

fn SelfH3(mut handle: &mut [u8]) -> *mut H3 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn HashBytesH3(mut data: &[u8]) -> u32 {
  let h: usize = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 5i32).wrapping_mul(kHashMul64);
  (h >> 64i32 - 16i32) as (u32)
}

fn FindLongestMatchH3(mut handle: &mut [u8],
                      mut dictionary: &[BrotliDictionary],
                      mut dictionary_hash: &[u16],
                      mut data: &[u8],
                      ring_buffer_mask: usize,
                      mut distance_cache: &[i32],
                      cur_ix: usize,
                      max_length: usize,
                      max_backward: usize,
                      mut out: &mut [HasherSearchResult])
                      -> i32 {
  let mut xself: *mut H3 = SelfH3(handle);
  let best_len_in: usize = (*out).len;
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let key: u32 = HashBytesH3(&data[(cur_ix_masked as (usize))]);
  let mut compare_char: i32 = data[(cur_ix_masked.wrapping_add(best_len_in) as (usize))] as (i32);
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = best_len_in;
  let mut cached_backward: usize = distance_cache[(0usize)] as (usize);
  let mut prev_ix: usize = cur_ix.wrapping_sub(cached_backward);
  let mut is_match_found: i32 = 0i32;
  (*out).len_x_code = 0usize;
  if prev_ix < cur_ix {
    prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
    if compare_char == data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
      let mut len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
      if len >= 4usize {
        best_score = BackwardReferenceScoreUsingLastDistance(len);
        best_len = len;
        (*out).len = len;
        (*out).distance = cached_backward;
        (*out).score = best_score;
        compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
        if 2i32 == 1i32 {
          (*xself).buckets_[key as (usize)] = cur_ix as (u32);
          return 1i32;
        } else {
          is_match_found = 1i32;
        }
      }
    }
  }
  if 2i32 == 1i32 {
    let mut backward: usize;
    let mut len: usize;
    prev_ix = (*xself).buckets_[key as (usize)] as (usize);
    (*xself).buckets_[key as (usize)] = cur_ix as (u32);
    backward = cur_ix.wrapping_sub(prev_ix);
    prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
    if compare_char != data[(prev_ix.wrapping_add(best_len_in) as (usize))] as (i32) {
      return 0i32;
    }
    if backward == 0usize || backward > max_backward {
      return 0i32;
    }
    len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                   &data[(cur_ix_masked as (usize))],
                                   max_length);
    if len >= 4usize {
      (*out).len = len;
      (*out).distance = backward;
      (*out).score = BackwardReferenceScore(len, backward);
      return 1i32;
    }
  } else {
    let mut bucket: *mut u32 = (*xself).buckets_.as_mut_ptr().offset(key as (isize));
    let mut i: i32;
    prev_ix = *{
                 let _old = bucket;
                 bucket = bucket[(1 as (usize))..];
                 _old
               } as (usize);
    i = 0i32;
    while i < 2i32 {
      'continue15: loop {
        {
          let backward: usize = cur_ix.wrapping_sub(prev_ix);
          let mut len: usize;
          prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
          if compare_char != data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
            {
              break 'continue15;
            }
          }
          if backward == 0usize || backward > max_backward {
            {
              break 'continue15;
            }
          }
          len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                         &data[(cur_ix_masked as (usize))],
                                         max_length);
          if len >= 4usize {
            let score: usize = BackwardReferenceScore(len, backward);
            if best_score < score {
              best_score = score;
              best_len = len;
              (*out).len = best_len;
              (*out).distance = backward;
              (*out).score = score;
              compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
              is_match_found = 1i32;
            }
          }
        }
        break;
      }
      i = i + 1;
      prev_ix = *{
                   let _old = bucket;
                   bucket = bucket[(1 as (usize))..];
                   _old
                 } as (usize);
    }
  }
  if 0i32 != 0 && (is_match_found == 0) {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              1i32);
  }
  (*xself).buckets_[(key as (usize)).wrapping_add((cur_ix >> 3i32).wrapping_rem(2usize))] =
    cur_ix as (u32);
  is_match_found
}

fn StoreH3(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let key: u32 = HashBytesH3(&data[((ix & mask) as (usize))]);
  let off: u32 = (ix >> 3i32).wrapping_rem(2usize) as (u32);
  (*SelfH3(handle)).buckets_[key.wrapping_add(off) as (usize)] = ix as (u32);
}

fn StoreRangeH3(mut handle: &mut [u8],
                mut data: &[u8],
                mask: usize,
                ix_start: usize,
                ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH3(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH3(mut dictionary: &[BrotliDictionary],
                              mut dictionary_hash: &[u16],
                              mut num_bytes: usize,
                              mut position: usize,
                              mut ringbuffer: &[u8],
                              mut ringbuffer_mask: usize,
                              mut params: &[BrotliEncoderParams],
                              mut hasher: &mut [u8],
                              mut dist_cache: &mut [i32],
                              mut last_insert_len: &mut [usize],
                              mut commands: &mut [Command],
                              mut num_commands: &mut [usize],
                              mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH3() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH3()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH3(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH3()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH3(hasher,
                          dictionary,
                          dictionary_hash,
                          ringbuffer,
                          ringbuffer_mask,
                          dist_cache,
                          position,
                          max_length,
                          max_distance,
                          &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break16: loop {
        'continue17: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH3(hasher,
                                                dictionary,
                                                dictionary_hash,
                                                ringbuffer,
                                                ringbuffer_mask,
                                                dist_cache,
                                                position.wrapping_add(1usize),
                                                max_length,
                                                max_distance,
                                                &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH3()) < pos_end) {
                {
                  break 'continue17;
                }
              }
            }
            {
              {
                break 'break16;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH3(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH3(hasher,
                   ringbuffer,
                   ringbuffer_mask,
                   position.wrapping_add(2usize),
                   brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH3().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH3(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH3().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH3(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH4() -> usize {
  8usize
}

fn PrepareDistanceCacheH4(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  handle;
  distance_cache;
}

fn HashTypeLengthH4() -> usize {
  8usize
}



pub struct H4 {
  pub buckets_: [u32; 131076],
}

fn SelfH4(mut handle: &mut [u8]) -> *mut H4 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn HashBytesH4(mut data: &[u8]) -> u32 {
  let h: usize = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 5i32).wrapping_mul(kHashMul64);
  (h >> 64i32 - 17i32) as (u32)
}

fn FindLongestMatchH4(mut handle: &mut [u8],
                      mut dictionary: &[BrotliDictionary],
                      mut dictionary_hash: &[u16],
                      mut data: &[u8],
                      ring_buffer_mask: usize,
                      mut distance_cache: &[i32],
                      cur_ix: usize,
                      max_length: usize,
                      max_backward: usize,
                      mut out: &mut [HasherSearchResult])
                      -> i32 {
  let mut xself: *mut H4 = SelfH4(handle);
  let best_len_in: usize = (*out).len;
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let key: u32 = HashBytesH4(&data[(cur_ix_masked as (usize))]);
  let mut compare_char: i32 = data[(cur_ix_masked.wrapping_add(best_len_in) as (usize))] as (i32);
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = best_len_in;
  let mut cached_backward: usize = distance_cache[(0usize)] as (usize);
  let mut prev_ix: usize = cur_ix.wrapping_sub(cached_backward);
  let mut is_match_found: i32 = 0i32;
  (*out).len_x_code = 0usize;
  if prev_ix < cur_ix {
    prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
    if compare_char == data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
      let mut len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
      if len >= 4usize {
        best_score = BackwardReferenceScoreUsingLastDistance(len);
        best_len = len;
        (*out).len = len;
        (*out).distance = cached_backward;
        (*out).score = best_score;
        compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
        if 4i32 == 1i32 {
          (*xself).buckets_[key as (usize)] = cur_ix as (u32);
          return 1i32;
        } else {
          is_match_found = 1i32;
        }
      }
    }
  }
  if 4i32 == 1i32 {
    let mut backward: usize;
    let mut len: usize;
    prev_ix = (*xself).buckets_[key as (usize)] as (usize);
    (*xself).buckets_[key as (usize)] = cur_ix as (u32);
    backward = cur_ix.wrapping_sub(prev_ix);
    prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
    if compare_char != data[(prev_ix.wrapping_add(best_len_in) as (usize))] as (i32) {
      return 0i32;
    }
    if backward == 0usize || backward > max_backward {
      return 0i32;
    }
    len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                   &data[(cur_ix_masked as (usize))],
                                   max_length);
    if len >= 4usize {
      (*out).len = len;
      (*out).distance = backward;
      (*out).score = BackwardReferenceScore(len, backward);
      return 1i32;
    }
  } else {
    let mut bucket: *mut u32 = (*xself).buckets_.as_mut_ptr().offset(key as (isize));
    let mut i: i32;
    prev_ix = *{
                 let _old = bucket;
                 bucket = bucket[(1 as (usize))..];
                 _old
               } as (usize);
    i = 0i32;
    while i < 4i32 {
      'continue25: loop {
        {
          let backward: usize = cur_ix.wrapping_sub(prev_ix);
          let mut len: usize;
          prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
          if compare_char != data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
            {
              break 'continue25;
            }
          }
          if backward == 0usize || backward > max_backward {
            {
              break 'continue25;
            }
          }
          len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                         &data[(cur_ix_masked as (usize))],
                                         max_length);
          if len >= 4usize {
            let score: usize = BackwardReferenceScore(len, backward);
            if best_score < score {
              best_score = score;
              best_len = len;
              (*out).len = best_len;
              (*out).distance = backward;
              (*out).score = score;
              compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
              is_match_found = 1i32;
            }
          }
        }
        break;
      }
      i = i + 1;
      prev_ix = *{
                   let _old = bucket;
                   bucket = bucket[(1 as (usize))..];
                   _old
                 } as (usize);
    }
  }
  if 1i32 != 0 && (is_match_found == 0) {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              1i32);
  }
  (*xself).buckets_[(key as (usize)).wrapping_add((cur_ix >> 3i32).wrapping_rem(4usize))] =
    cur_ix as (u32);
  is_match_found
}

fn StoreH4(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let key: u32 = HashBytesH4(&data[((ix & mask) as (usize))]);
  let off: u32 = (ix >> 3i32).wrapping_rem(4usize) as (u32);
  (*SelfH4(handle)).buckets_[key.wrapping_add(off) as (usize)] = ix as (u32);
}

fn StoreRangeH4(mut handle: &mut [u8],
                mut data: &[u8],
                mask: usize,
                ix_start: usize,
                ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH4(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH4(mut dictionary: &[BrotliDictionary],
                              mut dictionary_hash: &[u16],
                              mut num_bytes: usize,
                              mut position: usize,
                              mut ringbuffer: &[u8],
                              mut ringbuffer_mask: usize,
                              mut params: &[BrotliEncoderParams],
                              mut hasher: &mut [u8],
                              mut dist_cache: &mut [i32],
                              mut last_insert_len: &mut [usize],
                              mut commands: &mut [Command],
                              mut num_commands: &mut [usize],
                              mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH4() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH4()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH4(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH4()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH4(hasher,
                          dictionary,
                          dictionary_hash,
                          ringbuffer,
                          ringbuffer_mask,
                          dist_cache,
                          position,
                          max_length,
                          max_distance,
                          &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break26: loop {
        'continue27: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH4(hasher,
                                                dictionary,
                                                dictionary_hash,
                                                ringbuffer,
                                                ringbuffer_mask,
                                                dist_cache,
                                                position.wrapping_add(1usize),
                                                max_length,
                                                max_distance,
                                                &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH4()) < pos_end) {
                {
                  break 'continue27;
                }
              }
            }
            {
              {
                break 'break26;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH4(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH4(hasher,
                   ringbuffer,
                   ringbuffer_mask,
                   position.wrapping_add(2usize),
                   brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH4().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH4(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH4().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH4(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH5() -> usize {
  4usize
}

fn PrepareDistanceCache(mut distance_cache: &mut [i32], num_distances: i32) {
  if num_distances > 4i32 {
    let mut last_distance: i32 = distance_cache[(0usize)];
    distance_cache[(4usize)] = last_distance - 1i32;
    distance_cache[(5usize)] = last_distance + 1i32;
    distance_cache[(6usize)] = last_distance - 2i32;
    distance_cache[(7usize)] = last_distance + 2i32;
    distance_cache[(8usize)] = last_distance - 3i32;
    distance_cache[(9usize)] = last_distance + 3i32;
    if num_distances > 10i32 {
      let mut next_last_distance: i32 = distance_cache[(1usize)];
      distance_cache[(10usize)] = next_last_distance - 1i32;
      distance_cache[(11usize)] = next_last_distance + 1i32;
      distance_cache[(12usize)] = next_last_distance - 2i32;
      distance_cache[(13usize)] = next_last_distance + 2i32;
      distance_cache[(14usize)] = next_last_distance - 3i32;
      distance_cache[(15usize)] = next_last_distance + 3i32;
    }
  }
}

fn PrepareDistanceCacheH5(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  PrepareDistanceCache(distance_cache,
                       (*GetHasherCommon(handle)).params.num_last_distances_to_check);
}

fn HashTypeLengthH5() -> usize {
  4usize
}



pub struct H5 {
  pub bucket_size_: usize,
  pub block_size_: usize,
  pub hash_shift_: i32,
  pub block_mask_: u32,
}

fn SelfH5(mut handle: &mut [u8]) -> *mut H5 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn NumH5(mut xself: &mut H5) -> *mut u16 {
  &mut xself[(1usize)]
}

fn BucketsH5(mut xself: &mut H5) -> *mut u32 {
  &mut *NumH5(xself).offset((*xself).bucket_size_ as (isize))
}

fn BackwardReferencePenaltyUsingLastDistance(mut distance_short_code: usize) -> usize {
  (39usize).wrapping_add((0x1ca10i32 >> (distance_short_code & 0xeusize) & 0xei32) as (usize))
}

fn HashBytesH5(mut data: &[u8], shift: i32) -> u32 {
  let mut h: u32 = BROTLI_UNALIGNED_LOAD32(data).wrapping_mul(kHashMul32);
  h >> shift
}

fn FindLongestMatchH5(mut handle: &mut [u8],
                      mut dictionary: &[BrotliDictionary],
                      mut dictionary_hash: &[u16],
                      mut data: &[u8],
                      ring_buffer_mask: usize,
                      mut distance_cache: &[i32],
                      cur_ix: usize,
                      max_length: usize,
                      max_backward: usize,
                      mut out: &mut [HasherSearchResult])
                      -> i32 {
  let mut common: *mut Struct1 = GetHasherCommon(handle);
  let mut xself: *mut H5 = SelfH5(handle);
  let mut num: *mut u16 = NumH5(xself);
  let mut buckets: *mut u32 = BucketsH5(xself);
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let mut is_match_found: i32 = 0i32;
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = (*out).len;
  let mut i: usize;
  (*out).len = 0usize;
  (*out).len_x_code = 0usize;
  i = 0usize;
  while i < (*common).params.num_last_distances_to_check as (usize) {
    'continue35: loop {
      {
        let backward: usize = distance_cache[(i as (usize))] as (usize);
        let mut prev_ix: usize = cur_ix.wrapping_sub(backward);
        if prev_ix >= cur_ix {
          {
            break 'continue35;
          }
        }
        if backward > max_backward {
          {
            break 'continue35;
          }
        }
        prev_ix = prev_ix & ring_buffer_mask;
        if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask ||
           prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
           data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
           data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
          {
            break 'continue35;
          }
        }
        {
          let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
          if len >= 3usize || len == 2usize && (i < 2usize) {
            let mut score: usize = BackwardReferenceScoreUsingLastDistance(len);
            if best_score < score {
              if i != 0usize {
                score = score.wrapping_sub(BackwardReferencePenaltyUsingLastDistance(i));
              }
              if best_score < score {
                best_score = score;
                best_len = len;
                (*out).len = best_len;
                (*out).distance = backward;
                (*out).score = best_score;
                is_match_found = 1i32;
              }
            }
          }
        }
      }
      break;
    }
    i = i.wrapping_add(1 as (usize));
  }
  {
    let key: u32 = HashBytesH5(&data[(cur_ix_masked as (usize))], (*xself).hash_shift_);
    let mut bucket: *mut u32 = &mut buckets[((key << (*common).params.block_bits) as (usize))];
    let down: usize = if num[(key as (usize))] as (usize) > (*xself).block_size_ {
      (num[(key as (usize))] as (usize)).wrapping_sub((*xself).block_size_)
    } else {
      0u32 as (usize)
    };
    i = num[(key as (usize))] as (usize);
    while i > down {
      let mut prev_ix: usize = bucket[(({
          i = i.wrapping_sub(1 as (usize));
          i
        } & (*xself).block_mask_ as (usize)) as
       (usize))] as (usize);
      let backward: usize = cur_ix.wrapping_sub(prev_ix);
      if backward > max_backward {
        {
          break;
        }
      }
      prev_ix = prev_ix & ring_buffer_mask;
      if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask ||
         prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
         data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
         data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
        {
          continue;
        }
      }
      {
        let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                  &data[(cur_ix_masked as (usize))],
                                                  max_length);
        if len >= 4usize {
          let mut score: usize = BackwardReferenceScore(len, backward);
          if best_score < score {
            best_score = score;
            best_len = len;
            (*out).len = best_len;
            (*out).distance = backward;
            (*out).score = best_score;
            is_match_found = 1i32;
          }
        }
      }
    }
    bucket[((num[(key as (usize))] as (u32) & (*xself).block_mask_) as (usize))] = cur_ix as (u32);
    {
      let _rhs = 1;
      let _lhs = &mut num[(key as (usize))];
      *_lhs = (*_lhs as (i32) + _rhs) as (u16);
    }
  }
  if is_match_found == 0 {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              0i32);
  }
  is_match_found
}

fn StoreH5(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let mut xself: *mut H5 = SelfH5(handle);
  let mut num: *mut u16 = NumH5(xself);
  let key: u32 = HashBytesH5(&data[((ix & mask) as (usize))], (*xself).hash_shift_);
  let minor_ix: usize = (num[(key as (usize))] as (u32) & (*xself).block_mask_) as (usize);
  let offset: usize = minor_ix.wrapping_add((key << (*GetHasherCommon(handle)).params.block_bits) as
                                            (usize));
  *BucketsH5(xself).offset(offset as (isize)) = ix as (u32);
  {
    let _rhs = 1;
    let _lhs = &mut num[(key as (usize))];
    *_lhs = (*_lhs as (i32) + _rhs) as (u16);
  }
}

fn StoreRangeH5(mut handle: &mut [u8],
                mut data: &[u8],
                mask: usize,
                ix_start: usize,
                ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH5(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH5(mut dictionary: &[BrotliDictionary],
                              mut dictionary_hash: &[u16],
                              mut num_bytes: usize,
                              mut position: usize,
                              mut ringbuffer: &[u8],
                              mut ringbuffer_mask: usize,
                              mut params: &[BrotliEncoderParams],
                              mut hasher: &mut [u8],
                              mut dist_cache: &mut [i32],
                              mut last_insert_len: &mut [usize],
                              mut commands: &mut [Command],
                              mut num_commands: &mut [usize],
                              mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH5() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH5()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH5(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH5()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH5(hasher,
                          dictionary,
                          dictionary_hash,
                          ringbuffer,
                          ringbuffer_mask,
                          dist_cache,
                          position,
                          max_length,
                          max_distance,
                          &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break36: loop {
        'continue37: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH5(hasher,
                                                dictionary,
                                                dictionary_hash,
                                                ringbuffer,
                                                ringbuffer_mask,
                                                dist_cache,
                                                position.wrapping_add(1usize),
                                                max_length,
                                                max_distance,
                                                &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH5()) < pos_end) {
                {
                  break 'continue37;
                }
              }
            }
            {
              {
                break 'break36;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH5(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH5(hasher,
                   ringbuffer,
                   ringbuffer_mask,
                   position.wrapping_add(2usize),
                   brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH5().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH5(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH5().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH5(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH6() -> usize {
  8usize
}

fn PrepareDistanceCacheH6(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  PrepareDistanceCache(distance_cache,
                       (*GetHasherCommon(handle)).params.num_last_distances_to_check);
}

fn HashTypeLengthH6() -> usize {
  8usize
}



pub struct H6 {
  pub bucket_size_: usize,
  pub block_size_: usize,
  pub hash_shift_: i32,
  pub hash_mask_: usize,
  pub block_mask_: u32,
}

fn SelfH6(mut handle: &mut [u8]) -> *mut H6 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn NumH6(mut xself: &mut H6) -> *mut u16 {
  &mut xself[(1usize)]
}

fn BucketsH6(mut xself: &mut H6) -> *mut u32 {
  &mut *NumH6(xself).offset((*xself).bucket_size_ as (isize))
}

fn HashBytesH6(mut data: &[u8], mask: usize, shift: i32) -> u32 {
  let h: usize = (BROTLI_UNALIGNED_LOAD64(data) & mask).wrapping_mul(kHashMul64Long);
  (h >> shift) as (u32)
}

fn FindLongestMatchH6(mut handle: &mut [u8],
                      mut dictionary: &[BrotliDictionary],
                      mut dictionary_hash: &[u16],
                      mut data: &[u8],
                      ring_buffer_mask: usize,
                      mut distance_cache: &[i32],
                      cur_ix: usize,
                      max_length: usize,
                      max_backward: usize,
                      mut out: &mut [HasherSearchResult])
                      -> i32 {
  let mut common: *mut Struct1 = GetHasherCommon(handle);
  let mut xself: *mut H6 = SelfH6(handle);
  let mut num: *mut u16 = NumH6(xself);
  let mut buckets: *mut u32 = BucketsH6(xself);
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let mut is_match_found: i32 = 0i32;
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = (*out).len;
  let mut i: usize;
  (*out).len = 0usize;
  (*out).len_x_code = 0usize;
  i = 0usize;
  while i < (*common).params.num_last_distances_to_check as (usize) {
    'continue45: loop {
      {
        let backward: usize = distance_cache[(i as (usize))] as (usize);
        let mut prev_ix: usize = cur_ix.wrapping_sub(backward);
        if prev_ix >= cur_ix {
          {
            break 'continue45;
          }
        }
        if backward > max_backward {
          {
            break 'continue45;
          }
        }
        prev_ix = prev_ix & ring_buffer_mask;
        if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask ||
           prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
           data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
           data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
          {
            break 'continue45;
          }
        }
        {
          let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
          if len >= 3usize || len == 2usize && (i < 2usize) {
            let mut score: usize = BackwardReferenceScoreUsingLastDistance(len);
            if best_score < score {
              if i != 0usize {
                score = score.wrapping_sub(BackwardReferencePenaltyUsingLastDistance(i));
              }
              if best_score < score {
                best_score = score;
                best_len = len;
                (*out).len = best_len;
                (*out).distance = backward;
                (*out).score = best_score;
                is_match_found = 1i32;
              }
            }
          }
        }
      }
      break;
    }
    i = i.wrapping_add(1 as (usize));
  }
  {
    let key: u32 = HashBytesH6(&data[(cur_ix_masked as (usize))],
                               (*xself).hash_mask_,
                               (*xself).hash_shift_);
    let mut bucket: *mut u32 = &mut buckets[((key << (*common).params.block_bits) as (usize))];
    let down: usize = if num[(key as (usize))] as (usize) > (*xself).block_size_ {
      (num[(key as (usize))] as (usize)).wrapping_sub((*xself).block_size_)
    } else {
      0u32 as (usize)
    };
    i = num[(key as (usize))] as (usize);
    while i > down {
      let mut prev_ix: usize = bucket[(({
          i = i.wrapping_sub(1 as (usize));
          i
        } & (*xself).block_mask_ as (usize)) as
       (usize))] as (usize);
      let backward: usize = cur_ix.wrapping_sub(prev_ix);
      if backward > max_backward {
        {
          break;
        }
      }
      prev_ix = prev_ix & ring_buffer_mask;
      if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask ||
         prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
         data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
         data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
        {
          continue;
        }
      }
      {
        let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                  &data[(cur_ix_masked as (usize))],
                                                  max_length);
        if len >= 4usize {
          let mut score: usize = BackwardReferenceScore(len, backward);
          if best_score < score {
            best_score = score;
            best_len = len;
            (*out).len = best_len;
            (*out).distance = backward;
            (*out).score = best_score;
            is_match_found = 1i32;
          }
        }
      }
    }
    bucket[((num[(key as (usize))] as (u32) & (*xself).block_mask_) as (usize))] = cur_ix as (u32);
    {
      let _rhs = 1;
      let _lhs = &mut num[(key as (usize))];
      *_lhs = (*_lhs as (i32) + _rhs) as (u16);
    }
  }
  if is_match_found == 0 {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              0i32);
  }
  is_match_found
}

fn StoreH6(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let mut xself: *mut H6 = SelfH6(handle);
  let mut num: *mut u16 = NumH6(xself);
  let key: u32 = HashBytesH6(&data[((ix & mask) as (usize))],
                             (*xself).hash_mask_,
                             (*xself).hash_shift_);
  let minor_ix: usize = (num[(key as (usize))] as (u32) & (*xself).block_mask_) as (usize);
  let offset: usize = minor_ix.wrapping_add((key << (*GetHasherCommon(handle)).params.block_bits) as
                                            (usize));
  *BucketsH6(xself).offset(offset as (isize)) = ix as (u32);
  {
    let _rhs = 1;
    let _lhs = &mut num[(key as (usize))];
    *_lhs = (*_lhs as (i32) + _rhs) as (u16);
  }
}

fn StoreRangeH6(mut handle: &mut [u8],
                mut data: &[u8],
                mask: usize,
                ix_start: usize,
                ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH6(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH6(mut dictionary: &[BrotliDictionary],
                              mut dictionary_hash: &[u16],
                              mut num_bytes: usize,
                              mut position: usize,
                              mut ringbuffer: &[u8],
                              mut ringbuffer_mask: usize,
                              mut params: &[BrotliEncoderParams],
                              mut hasher: &mut [u8],
                              mut dist_cache: &mut [i32],
                              mut last_insert_len: &mut [usize],
                              mut commands: &mut [Command],
                              mut num_commands: &mut [usize],
                              mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH6() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH6()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH6(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH6()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH6(hasher,
                          dictionary,
                          dictionary_hash,
                          ringbuffer,
                          ringbuffer_mask,
                          dist_cache,
                          position,
                          max_length,
                          max_distance,
                          &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break46: loop {
        'continue47: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH6(hasher,
                                                dictionary,
                                                dictionary_hash,
                                                ringbuffer,
                                                ringbuffer_mask,
                                                dist_cache,
                                                position.wrapping_add(1usize),
                                                max_length,
                                                max_distance,
                                                &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH6()) < pos_end) {
                {
                  break 'continue47;
                }
              }
            }
            {
              {
                break 'break46;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH6(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH6(hasher,
                   ringbuffer,
                   ringbuffer_mask,
                   position.wrapping_add(2usize),
                   brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH6().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH6(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH6().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH6(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH40() -> usize {
  4usize
}

fn PrepareDistanceCacheH40(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  handle;
  PrepareDistanceCache(distance_cache, 4i32);
}

fn HashTypeLengthH40() -> usize {
  4usize
}






pub struct BankH40 {
  pub slots: [SlotH40; 65536],
}



pub struct H40 {
  pub addr: [u32; 32768],
  pub head: [u16; 32768],
  pub tiny_hash: [u8; 65536],
  pub banks: [BankH40; 1],
  pub free_slot_idx: [u16; 1],
  pub max_hops: usize,
}

fn SelfH40(mut handle: &mut [u8]) -> *mut H40 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn HashBytesH40(mut data: &[u8]) -> usize {
  let h: u32 = BROTLI_UNALIGNED_LOAD32(data).wrapping_mul(kHashMul32);
  (h >> 32i32 - 15i32) as (usize)
}

fn StoreH40(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let mut xself: *mut H40 = SelfH40(handle);
  let key: usize = HashBytesH40(&data[((ix & mask) as (usize))]);
  let bank: usize = key & (1i32 - 1i32) as (usize);
  let idx: usize = (({
                       let _rhs = 1;
                       let _lhs = &mut (*xself).free_slot_idx[bank];
                       let _old = *_lhs;
                       *_lhs = (*_lhs as (i32) + _rhs) as (u16);
                       _old
                     }) as (i32) & 65536i32 - 1i32) as (usize);
  let mut delta: usize = ix.wrapping_sub((*xself).addr[key] as (usize));
  (*xself).tiny_hash[ix as (u16) as (usize)] = key as (u8);
  if delta > 0xffffusize {
    delta = if 0i32 != 0 { 0i32 } else { 0xffffi32 } as (usize);
  }
  (*xself).banks[bank].slots[idx].delta = delta as (u16);
  (*xself).banks[bank].slots[idx].next = (*xself).head[key];
  (*xself).addr[key] = ix as (u32);
  (*xself).head[key] = idx as (u16);
}

fn FindLongestMatchH40(mut handle: &mut [u8],
                       mut dictionary: &[BrotliDictionary],
                       mut dictionary_hash: &[u16],
                       mut data: &[u8],
                       ring_buffer_mask: usize,
                       mut distance_cache: &[i32],
                       cur_ix: usize,
                       max_length: usize,
                       max_backward: usize,
                       mut out: &mut [HasherSearchResult])
                       -> i32 {
  let mut xself: *mut H40 = SelfH40(handle);
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let mut is_match_found: i32 = 0i32;
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = (*out).len;
  let mut i: usize;
  let key: usize = HashBytesH40(&data[(cur_ix_masked as (usize))]);
  let tiny_hash: u8 = key as (u8);
  (*out).len = 0usize;
  (*out).len_x_code = 0usize;
  i = 0usize;
  while i < 4usize {
    'continue55: loop {
      {
        let backward: usize = distance_cache[(i as (usize))] as (usize);
        let mut prev_ix: usize = cur_ix.wrapping_sub(backward);
        if i > 0usize &&
           ((*xself).tiny_hash[prev_ix as (u16) as (usize)] as (i32) != tiny_hash as (i32)) {
          {
            break 'continue55;
          }
        }
        if prev_ix >= cur_ix || backward > max_backward {
          {
            break 'continue55;
          }
        }
        prev_ix = prev_ix & ring_buffer_mask;
        {
          let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
          if len >= 2usize {
            let mut score: usize = BackwardReferenceScoreUsingLastDistance(len);
            if best_score < score {
              if i != 0usize {
                score = score.wrapping_sub(BackwardReferencePenaltyUsingLastDistance(i));
              }
              if best_score < score {
                best_score = score;
                best_len = len;
                (*out).len = best_len;
                (*out).distance = backward;
                (*out).score = best_score;
                is_match_found = 1i32;
              }
            }
          }
        }
      }
      break;
    }
    i = i.wrapping_add(1 as (usize));
  }
  {
    let bank: usize = key & (1i32 - 1i32) as (usize);
    let mut backward: usize = 0usize;
    let mut hops: usize = (*xself).max_hops;
    let mut delta: usize = cur_ix.wrapping_sub((*xself).addr[key] as (usize));
    let mut slot: usize = (*xself).head[key] as (usize);
    while {
            let _old = hops;
            hops = hops.wrapping_sub(1 as (usize));
            _old
          } != 0 {
      let mut prev_ix: usize;
      let mut last: usize = slot;
      backward = backward.wrapping_add(delta);
      if backward > max_backward || 0i32 != 0 && (delta == 0) {
        {
          break;
        }
      }
      prev_ix = cur_ix.wrapping_sub(backward) & ring_buffer_mask;
      slot = (*xself).banks[bank].slots[last].next as (usize);
      delta = (*xself).banks[bank].slots[last].delta as (usize);
      if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask ||
         prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
         data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
         data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
        {
          continue;
        }
      }
      {
        let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                  &data[(cur_ix_masked as (usize))],
                                                  max_length);
        if len >= 4usize {
          let mut score: usize = BackwardReferenceScore(len, backward);
          if best_score < score {
            best_score = score;
            best_len = len;
            (*out).len = best_len;
            (*out).distance = backward;
            (*out).score = best_score;
            is_match_found = 1i32;
          }
        }
      }
    }
    StoreH40(handle, data, ring_buffer_mask, cur_ix);
  }
  if is_match_found == 0 {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              0i32);
  }
  is_match_found
}

fn StoreRangeH40(mut handle: &mut [u8],
                 mut data: &[u8],
                 mask: usize,
                 ix_start: usize,
                 ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH40(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH40(mut dictionary: &[BrotliDictionary],
                               mut dictionary_hash: &[u16],
                               mut num_bytes: usize,
                               mut position: usize,
                               mut ringbuffer: &[u8],
                               mut ringbuffer_mask: usize,
                               mut params: &[BrotliEncoderParams],
                               mut hasher: &mut [u8],
                               mut dist_cache: &mut [i32],
                               mut last_insert_len: &mut [usize],
                               mut commands: &mut [Command],
                               mut num_commands: &mut [usize],
                               mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH40() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH40()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH40(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH40()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH40(hasher,
                           dictionary,
                           dictionary_hash,
                           ringbuffer,
                           ringbuffer_mask,
                           dist_cache,
                           position,
                           max_length,
                           max_distance,
                           &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break56: loop {
        'continue57: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH40(hasher,
                                                 dictionary,
                                                 dictionary_hash,
                                                 ringbuffer,
                                                 ringbuffer_mask,
                                                 dist_cache,
                                                 position.wrapping_add(1usize),
                                                 max_length,
                                                 max_distance,
                                                 &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH40()) < pos_end) {
                {
                  break 'continue57;
                }
              }
            }
            {
              {
                break 'break56;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH40(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH40(hasher,
                    ringbuffer,
                    ringbuffer_mask,
                    position.wrapping_add(2usize),
                    brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH40().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH40(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH40().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH40(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH41() -> usize {
  4usize
}

fn PrepareDistanceCacheH41(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  handle;
  PrepareDistanceCache(distance_cache, 10i32);
}

fn HashTypeLengthH41() -> usize {
  4usize
}



pub struct SlotH41 {
  pub delta: u16,
  pub next: u16,
}



pub struct BankH41 {
  pub slots: [SlotH41; 65536],
}



pub struct H41 {
  pub addr: [u32; 32768],
  pub head: [u16; 32768],
  pub tiny_hash: [u8; 65536],
  pub banks: [BankH41; 1],
  pub free_slot_idx: [u16; 1],
  pub max_hops: usize,
}

fn SelfH41(mut handle: &mut [u8]) -> *mut H41 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn HashBytesH41(mut data: &[u8]) -> usize {
  let h: u32 = BROTLI_UNALIGNED_LOAD32(data).wrapping_mul(kHashMul32);
  (h >> 32i32 - 15i32) as (usize)
}

fn StoreH41(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let mut xself: *mut H41 = SelfH41(handle);
  let key: usize = HashBytesH41(&data[((ix & mask) as (usize))]);
  let bank: usize = key & (1i32 - 1i32) as (usize);
  let idx: usize = (({
                       let _rhs = 1;
                       let _lhs = &mut (*xself).free_slot_idx[bank];
                       let _old = *_lhs;
                       *_lhs = (*_lhs as (i32) + _rhs) as (u16);
                       _old
                     }) as (i32) & 65536i32 - 1i32) as (usize);
  let mut delta: usize = ix.wrapping_sub((*xself).addr[key] as (usize));
  (*xself).tiny_hash[ix as (u16) as (usize)] = key as (u8);
  if delta > 0xffffusize {
    delta = if 0i32 != 0 { 0i32 } else { 0xffffi32 } as (usize);
  }
  (*xself).banks[bank].slots[idx].delta = delta as (u16);
  (*xself).banks[bank].slots[idx].next = (*xself).head[key];
  (*xself).addr[key] = ix as (u32);
  (*xself).head[key] = idx as (u16);
}

fn FindLongestMatchH41(mut handle: &mut [u8],
                       mut dictionary: &[BrotliDictionary],
                       mut dictionary_hash: &[u16],
                       mut data: &[u8],
                       ring_buffer_mask: usize,
                       mut distance_cache: &[i32],
                       cur_ix: usize,
                       max_length: usize,
                       max_backward: usize,
                       mut out: &mut [HasherSearchResult])
                       -> i32 {
  let mut xself: *mut H41 = SelfH41(handle);
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let mut is_match_found: i32 = 0i32;
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = (*out).len;
  let mut i: usize;
  let key: usize = HashBytesH41(&data[(cur_ix_masked as (usize))]);
  let tiny_hash: u8 = key as (u8);
  (*out).len = 0usize;
  (*out).len_x_code = 0usize;
  i = 0usize;
  while i < 10usize {
    'continue65: loop {
      {
        let backward: usize = distance_cache[(i as (usize))] as (usize);
        let mut prev_ix: usize = cur_ix.wrapping_sub(backward);
        if i > 0usize &&
           ((*xself).tiny_hash[prev_ix as (u16) as (usize)] as (i32) != tiny_hash as (i32)) {
          {
            break 'continue65;
          }
        }
        if prev_ix >= cur_ix || backward > max_backward {
          {
            break 'continue65;
          }
        }
        prev_ix = prev_ix & ring_buffer_mask;
        {
          let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
          if len >= 2usize {
            let mut score: usize = BackwardReferenceScoreUsingLastDistance(len);
            if best_score < score {
              if i != 0usize {
                score = score.wrapping_sub(BackwardReferencePenaltyUsingLastDistance(i));
              }
              if best_score < score {
                best_score = score;
                best_len = len;
                (*out).len = best_len;
                (*out).distance = backward;
                (*out).score = best_score;
                is_match_found = 1i32;
              }
            }
          }
        }
      }
      break;
    }
    i = i.wrapping_add(1 as (usize));
  }
  {
    let bank: usize = key & (1i32 - 1i32) as (usize);
    let mut backward: usize = 0usize;
    let mut hops: usize = (*xself).max_hops;
    let mut delta: usize = cur_ix.wrapping_sub((*xself).addr[key] as (usize));
    let mut slot: usize = (*xself).head[key] as (usize);
    while {
            let _old = hops;
            hops = hops.wrapping_sub(1 as (usize));
            _old
          } != 0 {
      let mut prev_ix: usize;
      let mut last: usize = slot;
      backward = backward.wrapping_add(delta);
      if backward > max_backward || 0i32 != 0 && (delta == 0) {
        {
          break;
        }
      }
      prev_ix = cur_ix.wrapping_sub(backward) & ring_buffer_mask;
      slot = (*xself).banks[bank].slots[last].next as (usize);
      delta = (*xself).banks[bank].slots[last].delta as (usize);
      if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask ||
         prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
         data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
         data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
        {
          continue;
        }
      }
      {
        let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                  &data[(cur_ix_masked as (usize))],
                                                  max_length);
        if len >= 4usize {
          let mut score: usize = BackwardReferenceScore(len, backward);
          if best_score < score {
            best_score = score;
            best_len = len;
            (*out).len = best_len;
            (*out).distance = backward;
            (*out).score = best_score;
            is_match_found = 1i32;
          }
        }
      }
    }
    StoreH41(handle, data, ring_buffer_mask, cur_ix);
  }
  if is_match_found == 0 {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              0i32);
  }
  is_match_found
}

fn StoreRangeH41(mut handle: &mut [u8],
                 mut data: &[u8],
                 mask: usize,
                 ix_start: usize,
                 ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH41(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH41(mut dictionary: &[BrotliDictionary],
                               mut dictionary_hash: &[u16],
                               mut num_bytes: usize,
                               mut position: usize,
                               mut ringbuffer: &[u8],
                               mut ringbuffer_mask: usize,
                               mut params: &[BrotliEncoderParams],
                               mut hasher: &mut [u8],
                               mut dist_cache: &mut [i32],
                               mut last_insert_len: &mut [usize],
                               mut commands: &mut [Command],
                               mut num_commands: &mut [usize],
                               mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH41() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH41()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH41(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH41()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH41(hasher,
                           dictionary,
                           dictionary_hash,
                           ringbuffer,
                           ringbuffer_mask,
                           dist_cache,
                           position,
                           max_length,
                           max_distance,
                           &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break66: loop {
        'continue67: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH41(hasher,
                                                 dictionary,
                                                 dictionary_hash,
                                                 ringbuffer,
                                                 ringbuffer_mask,
                                                 dist_cache,
                                                 position.wrapping_add(1usize),
                                                 max_length,
                                                 max_distance,
                                                 &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH41()) < pos_end) {
                {
                  break 'continue67;
                }
              }
            }
            {
              {
                break 'break66;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH41(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH41(hasher,
                    ringbuffer,
                    ringbuffer_mask,
                    position.wrapping_add(2usize),
                    brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH41().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH41(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH41().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH41(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH42() -> usize {
  4usize
}

fn PrepareDistanceCacheH42(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  handle;
  PrepareDistanceCache(distance_cache, 16i32);
}

fn HashTypeLengthH42() -> usize {
  4usize
}



pub struct SlotH42 {
  pub delta: u16,
  pub next: u16,
}



pub struct BankH42 {
  pub slots: [SlotH42; 512],
}



pub struct H42 {
  pub addr: [u32; 32768],
  pub head: [u16; 32768],
  pub tiny_hash: [u8; 65536],
  pub banks: [BankH42; 512],
  pub free_slot_idx: [u16; 512],
  pub max_hops: usize,
}

fn SelfH42(mut handle: &mut [u8]) -> *mut H42 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn HashBytesH42(mut data: &[u8]) -> usize {
  let h: u32 = BROTLI_UNALIGNED_LOAD32(data).wrapping_mul(kHashMul32);
  (h >> 32i32 - 15i32) as (usize)
}

fn StoreH42(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let mut xself: *mut H42 = SelfH42(handle);
  let key: usize = HashBytesH42(&data[((ix & mask) as (usize))]);
  let bank: usize = key & (512i32 - 1i32) as (usize);
  let idx: usize = (({
                       let _rhs = 1;
                       let _lhs = &mut (*xself).free_slot_idx[bank];
                       let _old = *_lhs;
                       *_lhs = (*_lhs as (i32) + _rhs) as (u16);
                       _old
                     }) as (i32) & 512i32 - 1i32) as (usize);
  let mut delta: usize = ix.wrapping_sub((*xself).addr[key] as (usize));
  (*xself).tiny_hash[ix as (u16) as (usize)] = key as (u8);
  if delta > 0xffffusize {
    delta = if 0i32 != 0 { 0i32 } else { 0xffffi32 } as (usize);
  }
  (*xself).banks[bank].slots[idx].delta = delta as (u16);
  (*xself).banks[bank].slots[idx].next = (*xself).head[key];
  (*xself).addr[key] = ix as (u32);
  (*xself).head[key] = idx as (u16);
}

fn FindLongestMatchH42(mut handle: &mut [u8],
                       mut dictionary: &[BrotliDictionary],
                       mut dictionary_hash: &[u16],
                       mut data: &[u8],
                       ring_buffer_mask: usize,
                       mut distance_cache: &[i32],
                       cur_ix: usize,
                       max_length: usize,
                       max_backward: usize,
                       mut out: &mut [HasherSearchResult])
                       -> i32 {
  let mut xself: *mut H42 = SelfH42(handle);
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let mut is_match_found: i32 = 0i32;
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = (*out).len;
  let mut i: usize;
  let key: usize = HashBytesH42(&data[(cur_ix_masked as (usize))]);
  let tiny_hash: u8 = key as (u8);
  (*out).len = 0usize;
  (*out).len_x_code = 0usize;
  i = 0usize;
  while i < 16usize {
    'continue75: loop {
      {
        let backward: usize = distance_cache[(i as (usize))] as (usize);
        let mut prev_ix: usize = cur_ix.wrapping_sub(backward);
        if i > 0usize &&
           ((*xself).tiny_hash[prev_ix as (u16) as (usize)] as (i32) != tiny_hash as (i32)) {
          {
            break 'continue75;
          }
        }
        if prev_ix >= cur_ix || backward > max_backward {
          {
            break 'continue75;
          }
        }
        prev_ix = prev_ix & ring_buffer_mask;
        {
          let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
          if len >= 2usize {
            let mut score: usize = BackwardReferenceScoreUsingLastDistance(len);
            if best_score < score {
              if i != 0usize {
                score = score.wrapping_sub(BackwardReferencePenaltyUsingLastDistance(i));
              }
              if best_score < score {
                best_score = score;
                best_len = len;
                (*out).len = best_len;
                (*out).distance = backward;
                (*out).score = best_score;
                is_match_found = 1i32;
              }
            }
          }
        }
      }
      break;
    }
    i = i.wrapping_add(1 as (usize));
  }
  {
    let bank: usize = key & (512i32 - 1i32) as (usize);
    let mut backward: usize = 0usize;
    let mut hops: usize = (*xself).max_hops;
    let mut delta: usize = cur_ix.wrapping_sub((*xself).addr[key] as (usize));
    let mut slot: usize = (*xself).head[key] as (usize);
    while {
            let _old = hops;
            hops = hops.wrapping_sub(1 as (usize));
            _old
          } != 0 {
      let mut prev_ix: usize;
      let mut last: usize = slot;
      backward = backward.wrapping_add(delta);
      if backward > max_backward || 0i32 != 0 && (delta == 0) {
        {
          break;
        }
      }
      prev_ix = cur_ix.wrapping_sub(backward) & ring_buffer_mask;
      slot = (*xself).banks[bank].slots[last].next as (usize);
      delta = (*xself).banks[bank].slots[last].delta as (usize);
      if cur_ix_masked.wrapping_add(best_len) > ring_buffer_mask ||
         prev_ix.wrapping_add(best_len) > ring_buffer_mask ||
         data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32) !=
         data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
        {
          continue;
        }
      }
      {
        let len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                  &data[(cur_ix_masked as (usize))],
                                                  max_length);
        if len >= 4usize {
          let mut score: usize = BackwardReferenceScore(len, backward);
          if best_score < score {
            best_score = score;
            best_len = len;
            (*out).len = best_len;
            (*out).distance = backward;
            (*out).score = best_score;
            is_match_found = 1i32;
          }
        }
      }
    }
    StoreH42(handle, data, ring_buffer_mask, cur_ix);
  }
  if is_match_found == 0 {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              0i32);
  }
  is_match_found
}

fn StoreRangeH42(mut handle: &mut [u8],
                 mut data: &[u8],
                 mask: usize,
                 ix_start: usize,
                 ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH42(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH42(mut dictionary: &[BrotliDictionary],
                               mut dictionary_hash: &[u16],
                               mut num_bytes: usize,
                               mut position: usize,
                               mut ringbuffer: &[u8],
                               mut ringbuffer_mask: usize,
                               mut params: &[BrotliEncoderParams],
                               mut hasher: &mut [u8],
                               mut dist_cache: &mut [i32],
                               mut last_insert_len: &mut [usize],
                               mut commands: &mut [Command],
                               mut num_commands: &mut [usize],
                               mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH42() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH42()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH42(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH42()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH42(hasher,
                           dictionary,
                           dictionary_hash,
                           ringbuffer,
                           ringbuffer_mask,
                           dist_cache,
                           position,
                           max_length,
                           max_distance,
                           &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break76: loop {
        'continue77: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH42(hasher,
                                                 dictionary,
                                                 dictionary_hash,
                                                 ringbuffer,
                                                 ringbuffer_mask,
                                                 dist_cache,
                                                 position.wrapping_add(1usize),
                                                 max_length,
                                                 max_distance,
                                                 &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH42()) < pos_end) {
                {
                  break 'continue77;
                }
              }
            }
            {
              {
                break 'break76;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH42(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH42(hasher,
                    ringbuffer,
                    ringbuffer_mask,
                    position.wrapping_add(2usize),
                    brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH42().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH42(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH42().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH42(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}

fn StoreLookaheadH54() -> usize {
  8usize
}

fn PrepareDistanceCacheH54(mut handle: &mut [u8], mut distance_cache: &mut [i32]) {
  handle;
  distance_cache;
}

fn HashTypeLengthH54() -> usize {
  8usize
}



pub struct H54 {
  pub buckets_: [u32; 1048580],
}

fn SelfH54(mut handle: &mut [u8]) -> *mut H54 {
  &mut *GetHasherCommon(handle).offset(1i32 as (isize))
}

fn HashBytesH54(mut data: &[u8]) -> u32 {
  let h: usize = (BROTLI_UNALIGNED_LOAD64(data) << 64i32 - 8i32 * 7i32).wrapping_mul(kHashMul64);
  (h >> 64i32 - 20i32) as (u32)
}

fn FindLongestMatchH54(mut handle: &mut [u8],
                       mut dictionary: &[BrotliDictionary],
                       mut dictionary_hash: &[u16],
                       mut data: &[u8],
                       ring_buffer_mask: usize,
                       mut distance_cache: &[i32],
                       cur_ix: usize,
                       max_length: usize,
                       max_backward: usize,
                       mut out: &mut [HasherSearchResult])
                       -> i32 {
  let mut xself: *mut H54 = SelfH54(handle);
  let best_len_in: usize = (*out).len;
  let cur_ix_masked: usize = cur_ix & ring_buffer_mask;
  let key: u32 = HashBytesH54(&data[(cur_ix_masked as (usize))]);
  let mut compare_char: i32 = data[(cur_ix_masked.wrapping_add(best_len_in) as (usize))] as (i32);
  let mut best_score: usize = (*out).score;
  let mut best_len: usize = best_len_in;
  let mut cached_backward: usize = distance_cache[(0usize)] as (usize);
  let mut prev_ix: usize = cur_ix.wrapping_sub(cached_backward);
  let mut is_match_found: i32 = 0i32;
  (*out).len_x_code = 0usize;
  if prev_ix < cur_ix {
    prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
    if compare_char == data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
      let mut len: usize = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                                    &data[(cur_ix_masked as (usize))],
                                                    max_length);
      if len >= 4usize {
        best_score = BackwardReferenceScoreUsingLastDistance(len);
        best_len = len;
        (*out).len = len;
        (*out).distance = cached_backward;
        (*out).score = best_score;
        compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
        if 4i32 == 1i32 {
          (*xself).buckets_[key as (usize)] = cur_ix as (u32);
          return 1i32;
        } else {
          is_match_found = 1i32;
        }
      }
    }
  }
  if 4i32 == 1i32 {
    let mut backward: usize;
    let mut len: usize;
    prev_ix = (*xself).buckets_[key as (usize)] as (usize);
    (*xself).buckets_[key as (usize)] = cur_ix as (u32);
    backward = cur_ix.wrapping_sub(prev_ix);
    prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
    if compare_char != data[(prev_ix.wrapping_add(best_len_in) as (usize))] as (i32) {
      return 0i32;
    }
    if backward == 0usize || backward > max_backward {
      return 0i32;
    }
    len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                   &data[(cur_ix_masked as (usize))],
                                   max_length);
    if len >= 4usize {
      (*out).len = len;
      (*out).distance = backward;
      (*out).score = BackwardReferenceScore(len, backward);
      return 1i32;
    }
  } else {
    let mut bucket: *mut u32 = (*xself).buckets_.as_mut_ptr().offset(key as (isize));
    let mut i: i32;
    prev_ix = *{
                 let _old = bucket;
                 bucket = bucket[(1 as (usize))..];
                 _old
               } as (usize);
    i = 0i32;
    while i < 4i32 {
      'continue85: loop {
        {
          let backward: usize = cur_ix.wrapping_sub(prev_ix);
          let mut len: usize;
          prev_ix = prev_ix & ring_buffer_mask as (u32) as (usize);
          if compare_char != data[(prev_ix.wrapping_add(best_len) as (usize))] as (i32) {
            {
              break 'continue85;
            }
          }
          if backward == 0usize || backward > max_backward {
            {
              break 'continue85;
            }
          }
          len = FindMatchLengthWithLimit(&data[(prev_ix as (usize))],
                                         &data[(cur_ix_masked as (usize))],
                                         max_length);
          if len >= 4usize {
            let score: usize = BackwardReferenceScore(len, backward);
            if best_score < score {
              best_score = score;
              best_len = len;
              (*out).len = best_len;
              (*out).distance = backward;
              (*out).score = score;
              compare_char = data[(cur_ix_masked.wrapping_add(best_len) as (usize))] as (i32);
              is_match_found = 1i32;
            }
          }
        }
        break;
      }
      i = i + 1;
      prev_ix = *{
                   let _old = bucket;
                   bucket = bucket[(1 as (usize))..];
                   _old
                 } as (usize);
    }
  }
  if 0i32 != 0 && (is_match_found == 0) {
    is_match_found = SearchInStaticDictionary(dictionary,
                                              dictionary_hash,
                                              handle,
                                              &data[(cur_ix_masked as (usize))],
                                              max_length,
                                              max_backward,
                                              out,
                                              1i32);
  }
  (*xself).buckets_[(key as (usize)).wrapping_add((cur_ix >> 3i32).wrapping_rem(4usize))] =
    cur_ix as (u32);
  is_match_found
}

fn StoreH54(mut handle: &mut [u8], mut data: &[u8], mask: usize, ix: usize) {
  let key: u32 = HashBytesH54(&data[((ix & mask) as (usize))]);
  let off: u32 = (ix >> 3i32).wrapping_rem(4usize) as (u32);
  (*SelfH54(handle)).buckets_[key.wrapping_add(off) as (usize)] = ix as (u32);
}

fn StoreRangeH54(mut handle: &mut [u8],
                 mut data: &[u8],
                 mask: usize,
                 ix_start: usize,
                 ix_end: usize) {
  let mut i: usize;
  i = ix_start;
  while i < ix_end {
    {
      StoreH54(handle, data, mask, i);
    }
    i = i.wrapping_add(1 as (usize));
  }
}

fn CreateBackwardReferencesH54(mut dictionary: &[BrotliDictionary],
                               mut dictionary_hash: &[u16],
                               mut num_bytes: usize,
                               mut position: usize,
                               mut ringbuffer: &[u8],
                               mut ringbuffer_mask: usize,
                               mut params: &[BrotliEncoderParams],
                               mut hasher: &mut [u8],
                               mut dist_cache: &mut [i32],
                               mut last_insert_len: &mut [usize],
                               mut commands: &mut [Command],
                               mut num_commands: &mut [usize],
                               mut num_literals: &mut [usize]) {
  let max_backward_limit: usize = (1usize << (*params).lgwin).wrapping_sub(16usize);
  let orig_commands: *const Command = commands;
  let mut insert_length: usize = *last_insert_len;
  let pos_end: usize = position.wrapping_add(num_bytes);
  let store_end: usize = if num_bytes >= StoreLookaheadH54() {
    position.wrapping_add(num_bytes).wrapping_sub(StoreLookaheadH54()).wrapping_add(1usize)
  } else {
    position
  };
  let random_heuristics_window_size: usize = LiteralSpreeLengthForSparseSearch(params);
  let mut apply_random_heuristics: usize = position.wrapping_add(random_heuristics_window_size);
  let kMinScore: usize =
    ((30i32 * 8i32) as (usize)).wrapping_mul(::core::mem::size_of::<usize>()).wrapping_add(100usize);
  PrepareDistanceCacheH54(hasher, dist_cache);
  while position.wrapping_add(HashTypeLengthH54()) < pos_end {
    let mut max_length: usize = pos_end.wrapping_sub(position);
    let mut max_distance: usize = brotli_min_size_t(position, max_backward_limit);
    let mut sr: HasherSearchResult;
    sr.len = 0usize;
    sr.len_x_code = 0usize;
    sr.distance = 0usize;
    sr.score = kMinScore;
    if FindLongestMatchH54(hasher,
                           dictionary,
                           dictionary_hash,
                           ringbuffer,
                           ringbuffer_mask,
                           dist_cache,
                           position,
                           max_length,
                           max_distance,
                           &mut sr) != 0 {
      let mut delayed_backward_references_in_row: i32 = 0i32;
      max_length = max_length.wrapping_sub(1 as (usize));
      'break86: loop {
        'continue87: loop {
          {
            let cost_diff_lazy: usize = 175usize;
            let mut is_match_found: i32;
            let mut sr2: HasherSearchResult;
            sr2.len = if (*params).quality < 5i32 {
              brotli_min_size_t(sr.len.wrapping_sub(1usize), max_length)
            } else {
              0usize
            };
            sr2.len_x_code = 0usize;
            sr2.distance = 0usize;
            sr2.score = kMinScore;
            max_distance = brotli_min_size_t(position.wrapping_add(1usize), max_backward_limit);
            is_match_found = FindLongestMatchH54(hasher,
                                                 dictionary,
                                                 dictionary_hash,
                                                 ringbuffer,
                                                 ringbuffer_mask,
                                                 dist_cache,
                                                 position.wrapping_add(1usize),
                                                 max_length,
                                                 max_distance,
                                                 &mut sr2);
            if is_match_found != 0 && (sr2.score >= sr.score.wrapping_add(cost_diff_lazy)) {
              position = position.wrapping_add(1 as (usize));
              insert_length = insert_length.wrapping_add(1 as (usize));
              sr = sr2;
              if {
                   delayed_backward_references_in_row = delayed_backward_references_in_row + 1;
                   delayed_backward_references_in_row
                 } < 4i32 &&
                 (position.wrapping_add(HashTypeLengthH54()) < pos_end) {
                {
                  break 'continue87;
                }
              }
            }
            {
              {
                break 'break86;
              }
            }
          }
          break;
        }
        max_length = max_length.wrapping_sub(1 as (usize));
      }
      apply_random_heuristics = position.wrapping_add((2usize).wrapping_mul(sr.len))
        .wrapping_add(random_heuristics_window_size);
      max_distance = brotli_min_size_t(position, max_backward_limit);
      {
        let mut distance_code: usize = ComputeDistanceCode(sr.distance, max_distance, dist_cache);
        if sr.distance <= max_distance && (distance_code > 0usize) {
          dist_cache[(3usize)] = dist_cache[(2usize)];
          dist_cache[(2usize)] = dist_cache[(1usize)];
          dist_cache[(1usize)] = dist_cache[(0usize)];
          dist_cache[(0usize)] = sr.distance as (i32);
          PrepareDistanceCacheH54(hasher, dist_cache);
        }
        InitCommand({
                      let _old = commands;
                      commands = commands[(1 as (usize))..];
                      _old
                    },
                    insert_length,
                    sr.len,
                    sr.len ^ sr.len_x_code,
                    distance_code);
      }
      *num_literals = (*num_literals).wrapping_add(insert_length);
      insert_length = 0usize;
      StoreRangeH54(hasher,
                    ringbuffer,
                    ringbuffer_mask,
                    position.wrapping_add(2usize),
                    brotli_min_size_t(position.wrapping_add(sr.len), store_end));
      position = position.wrapping_add(sr.len);
    } else {
      insert_length = insert_length.wrapping_add(1 as (usize));
      position = position.wrapping_add(1 as (usize));
      if position > apply_random_heuristics {
        if position >
           apply_random_heuristics.wrapping_add((4usize)
                                                  .wrapping_mul(random_heuristics_window_size)) {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH54().wrapping_sub(1usize), 4usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(16usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH54(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(4usize);
            }
            position = position.wrapping_add(4usize);
          }
        } else {
          let kMargin: usize = brotli_max_size_t(StoreLookaheadH54().wrapping_sub(1usize), 2usize);
          let mut pos_jump: usize = brotli_min_size_t(position.wrapping_add(8usize),
                                                      pos_end.wrapping_sub(kMargin));
          while position < pos_jump {
            {
              StoreH54(hasher, ringbuffer, ringbuffer_mask, position);
              insert_length = insert_length.wrapping_add(2usize);
            }
            position = position.wrapping_add(2usize);
          }
        }
      }
    }
  }
  insert_length = insert_length.wrapping_add(pos_end.wrapping_sub(position));
  *last_insert_len = insert_length;
  *num_commands =
    (*num_commands).wrapping_add(((commands as (isize)).wrapping_sub(orig_commands as (isize)) /
                                  ::core::mem::size_of::<*const Command>() as (isize)) as
                                 (usize));
}


pub fn BrotliCreateBackwardReferences(mut dictionary: &[BrotliDictionary],
                                      mut num_bytes: usize,
                                      mut position: usize,
                                      mut ringbuffer: &[u8],
                                      mut ringbuffer_mask: usize,
                                      mut params: &[BrotliEncoderParams],
                                      mut hasher: &mut [u8],
                                      mut dist_cache: &mut [i32],
                                      mut last_insert_len: &mut [usize],
                                      mut commands: &mut [Command],
                                      mut num_commands: &mut [usize],
                                      mut num_literals: &mut [usize]) {
  let mut hasher_type: i32 = (*params).hasher.type_;
  if hasher_type == 2i32 {
    CreateBackwardReferencesH2(dictionary,
                               kStaticDictionaryHash.as_ptr(),
                               num_bytes,
                               position,
                               ringbuffer,
                               ringbuffer_mask,
                               params,
                               hasher,
                               dist_cache,
                               last_insert_len,
                               commands,
                               num_commands,
                               num_literals);
  }
  if hasher_type == 3i32 {
    CreateBackwardReferencesH3(dictionary,
                               kStaticDictionaryHash.as_ptr(),
                               num_bytes,
                               position,
                               ringbuffer,
                               ringbuffer_mask,
                               params,
                               hasher,
                               dist_cache,
                               last_insert_len,
                               commands,
                               num_commands,
                               num_literals);
  }
  if hasher_type == 4i32 {
    CreateBackwardReferencesH4(dictionary,
                               kStaticDictionaryHash.as_ptr(),
                               num_bytes,
                               position,
                               ringbuffer,
                               ringbuffer_mask,
                               params,
                               hasher,
                               dist_cache,
                               last_insert_len,
                               commands,
                               num_commands,
                               num_literals);
  }
  if hasher_type == 5i32 {
    CreateBackwardReferencesH5(dictionary,
                               kStaticDictionaryHash.as_ptr(),
                               num_bytes,
                               position,
                               ringbuffer,
                               ringbuffer_mask,
                               params,
                               hasher,
                               dist_cache,
                               last_insert_len,
                               commands,
                               num_commands,
                               num_literals);
  }
  if hasher_type == 6i32 {
    CreateBackwardReferencesH6(dictionary,
                               kStaticDictionaryHash.as_ptr(),
                               num_bytes,
                               position,
                               ringbuffer,
                               ringbuffer_mask,
                               params,
                               hasher,
                               dist_cache,
                               last_insert_len,
                               commands,
                               num_commands,
                               num_literals);
  }
  if hasher_type == 40i32 {
    CreateBackwardReferencesH40(dictionary,
                                kStaticDictionaryHash.as_ptr(),
                                num_bytes,
                                position,
                                ringbuffer,
                                ringbuffer_mask,
                                params,
                                hasher,
                                dist_cache,
                                last_insert_len,
                                commands,
                                num_commands,
                                num_literals);
  }
  if hasher_type == 41i32 {
    CreateBackwardReferencesH41(dictionary,
                                kStaticDictionaryHash.as_ptr(),
                                num_bytes,
                                position,
                                ringbuffer,
                                ringbuffer_mask,
                                params,
                                hasher,
                                dist_cache,
                                last_insert_len,
                                commands,
                                num_commands,
                                num_literals);
  }
  if hasher_type == 42i32 {
    CreateBackwardReferencesH42(dictionary,
                                kStaticDictionaryHash.as_ptr(),
                                num_bytes,
                                position,
                                ringbuffer,
                                ringbuffer_mask,
                                params,
                                hasher,
                                dist_cache,
                                last_insert_len,
                                commands,
                                num_commands,
                                num_literals);
  }
  if hasher_type == 54i32 {
    CreateBackwardReferencesH54(dictionary,
                                kStaticDictionaryHash.as_ptr(),
                                num_bytes,
                                position,
                                ringbuffer,
                                ringbuffer_mask,
                                params,
                                hasher,
                                dist_cache,
                                last_insert_len,
                                commands,
                                num_commands,
                                num_literals);
  }
}
*/
