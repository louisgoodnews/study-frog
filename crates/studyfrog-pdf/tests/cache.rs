#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for PDF excerpt rasterization and LRU caching.

mod common;

use common::{bytes, three};
use studyfrog_pdf::cache::{
    CropCacheKey, RasterCrop, cache_clear, cache_get, cache_get_or_rasterize, cache_len, cache_put,
    crop_cache_filename, make_crop_cache_key, new_crop_cache, raster_crop_from_pixels,
    raster_crop_from_png, raster_crop_to_pixels, raster_crop_to_png, rasterize_crop,
};
use studyfrog_pdf::crop::PdfCropAnchor;

fn sample_key(id: u32) -> CropCacheKey {
    CropCacheKey {
        path: format!("doc{id}.pdf"),
        page: id,
        x: 10 * id,
        y: 20 * id,
        w: 100,
        h: 50,
        scale_milli: 1000,
    }
}

fn sample_crop(val: u8, w: u32, h: u32) -> RasterCrop {
    RasterCrop {
        width: w,
        height: h,
        rgba: vec![val; (w * h * 4) as usize],
    }
}

#[test]
fn lru_cache_put_get_and_len() {
    let mut cache = new_crop_cache(2);
    assert_eq!(cache_len(&cache), 0);

    let k1 = sample_key(1);
    let k2 = sample_key(2);
    cache_put(&mut cache, k1.clone(), sample_crop(10, 2, 2));
    assert_eq!(cache_len(&cache), 1);
    assert_eq!(cache_get(&mut cache, &k1).unwrap().rgba[0], 10);

    cache_put(&mut cache, k2.clone(), sample_crop(20, 2, 2));
    assert_eq!(cache_len(&cache), 2);
    assert_eq!(cache_get(&mut cache, &k2).unwrap().rgba[0], 20);
}

#[test]
fn lru_cache_eviction_order() {
    let mut cache = new_crop_cache(2);
    let (k1, k2, k3) = (sample_key(1), sample_key(2), sample_key(3));

    cache_put(&mut cache, k1.clone(), sample_crop(1, 1, 1));
    cache_put(&mut cache, k2.clone(), sample_crop(2, 1, 1));
    // k1 is LRU, k2 is MRU. Inserting k3 evicts k1.
    cache_put(&mut cache, k3.clone(), sample_crop(3, 1, 1));

    assert_eq!(cache_len(&cache), 2);
    assert!(cache_get(&mut cache, &k1).is_none());
    assert!(cache_get(&mut cache, &k2).is_some());
    assert!(cache_get(&mut cache, &k3).is_some());
}

#[test]
fn lru_cache_access_promotes_to_mru() {
    let mut cache = new_crop_cache(2);
    let (k1, k2, k3) = (sample_key(1), sample_key(2), sample_key(3));

    cache_put(&mut cache, k1.clone(), sample_crop(1, 1, 1));
    cache_put(&mut cache, k2.clone(), sample_crop(2, 1, 1));

    // Access k1 -> promotes k1 to MRU, making k2 the LRU
    assert!(cache_get(&mut cache, &k1).is_some());

    // Inserting k3 must evict k2, not k1
    cache_put(&mut cache, k3.clone(), sample_crop(3, 1, 1));
    assert!(cache_get(&mut cache, &k1).is_some());
    assert!(cache_get(&mut cache, &k2).is_none());
    assert!(cache_get(&mut cache, &k3).is_some());
}

#[test]
fn lru_cache_update_existing_key_and_zero_capacity() {
    let mut cache = new_crop_cache(2);
    let (k1, k2, k3) = (sample_key(1), sample_key(2), sample_key(3));

    cache_put(&mut cache, k1.clone(), sample_crop(1, 1, 1));
    cache_put(&mut cache, k2.clone(), sample_crop(2, 1, 1));
    // Update k1 -> updates value and moves to front
    cache_put(&mut cache, k1.clone(), sample_crop(99, 1, 1));
    assert_eq!(cache_len(&cache), 2);

    // k2 is LRU; inserting k3 should evict k2
    cache_put(&mut cache, k3, sample_crop(3, 1, 1));
    assert_eq!(cache_get(&mut cache, &k1).unwrap().rgba[0], 99);
    assert!(cache_get(&mut cache, &k2).is_none());

    // Zero capacity cache retains nothing
    let mut zero_cache = new_crop_cache(0);
    cache_put(&mut zero_cache, k1.clone(), sample_crop(1, 1, 1));
    assert_eq!(cache_len(&zero_cache), 0);
    assert!(cache_get(&mut zero_cache, &k1).is_none());
}

#[test]
fn lru_cache_clear() {
    let mut cache = new_crop_cache(5);
    cache_put(&mut cache, sample_key(1), sample_crop(1, 1, 1));
    cache_put(&mut cache, sample_key(2), sample_crop(2, 1, 1));
    assert_eq!(cache_len(&cache), 2);
    cache_clear(&mut cache);
    assert_eq!(cache_len(&cache), 0);
    assert!(cache_get(&mut cache, &sample_key(1)).is_none());
}

#[test]
fn make_crop_cache_key_conversions() {
    let anchor = PdfCropAnchor {
        path: "notes/sample.pdf".to_string(),
        page: 4,
        x: 10.4,
        y: 20.6,
        w: 100.2,
        h: 200.7,
    };
    let key = make_crop_cache_key(&anchor, 1.5);
    assert_eq!(
        key,
        CropCacheKey {
            path: "notes/sample.pdf".to_string(),
            page: 4,
            x: 10,
            y: 21,
            w: 100,
            h: 201,
            scale_milli: 1500,
        }
    );

    let nan_anchor = PdfCropAnchor {
        path: "doc.pdf".to_string(),
        page: 1,
        x: f32::NAN,
        y: -5.0,
        w: 50.0,
        h: f32::INFINITY,
    };
    let key_clamped = make_crop_cache_key(&nan_anchor, -1.0);
    assert_eq!(key_clamped.x, 0);
    assert_eq!(key_clamped.y, 0);
    assert_eq!(key_clamped.scale_milli, 0);
}

#[test]
fn crop_cache_filename_deterministic() {
    let key = CropCacheKey {
        path: "textbooks/histology.pdf".to_string(),
        page: 84,
        x: 150,
        y: 220,
        w: 500,
        h: 480,
        scale_milli: 1000,
    };
    let filename = crop_cache_filename(&key);
    assert_eq!(
        filename,
        "textbooks_histology_pdf_p84_x150_y220_w500_h480_s1000.png"
    );

    let empty_key = CropCacheKey {
        path: String::new(),
        page: 1,
        x: 0,
        y: 0,
        w: 10,
        h: 10,
        scale_milli: 500,
    };
    assert_eq!(
        crop_cache_filename(&empty_key),
        "crop_p1_x0_y0_w10_h10_s500.png"
    );
}

#[test]
fn raster_crop_pixels_and_png_roundtrip() {
    let crop = sample_crop(128, 4, 4);
    let px = raster_crop_to_pixels(&crop);
    assert_eq!(px.width, 4);
    assert_eq!(px.height, 4);
    let crop2 = raster_crop_from_pixels(px);
    assert_eq!(crop, crop2);

    let png_bytes = raster_crop_to_png(&crop).expect("PNG encoding should succeed");
    assert!(!png_bytes.is_empty());
    let decoded = raster_crop_from_png(&png_bytes).expect("PNG decoding should succeed");
    assert_eq!(decoded.width, crop.width);
    assert_eq!(decoded.height, crop.height);
    assert_eq!(decoded.rgba, crop.rgba);
}

#[test]
fn rasterize_crop_and_cache_integration() {
    let pdf = bytes(three());
    let anchor = PdfCropAnchor {
        path: "three.pdf".to_string(),
        page: 1,
        x: 50.0,
        y: 50.0,
        w: 100.0,
        h: 80.0,
    };

    let crop = rasterize_crop(&pdf, &anchor, 1.0).expect("rasterize should succeed");
    assert!(crop.width > 0 && crop.height > 0);
    assert_eq!(crop.rgba.len(), (crop.width * crop.height * 4) as usize);

    // Page 0 error
    let bad_anchor = PdfCropAnchor {
        page: 0,
        ..anchor.clone()
    };
    assert!(rasterize_crop(&pdf, &bad_anchor, 1.0).is_err());

    // Cache miss followed by hit
    let mut cache = new_crop_cache(2);
    let c1 =
        cache_get_or_rasterize(&mut cache, &pdf, &anchor, 1.0).expect("should rasterize on miss");
    assert_eq!(cache_len(&cache), 1);
    assert_eq!(c1, crop);

    let c2 = cache_get_or_rasterize(&mut cache, &pdf, &anchor, 1.0).expect("should hit from cache");
    assert_eq!(cache_len(&cache), 1);
    assert_eq!(c2, crop);
}
