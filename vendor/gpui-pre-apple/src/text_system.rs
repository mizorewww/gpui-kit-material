use anyhow::anyhow;
use collections::{HashMap, HashSet};
use core_foundation::{
    array::{CFArray, CFArrayRef},
    attributed_string::CFMutableAttributedString,
    base::{CFRange, TCFType},
    number::CFNumber,
    string::CFString,
};
use core_graphics::{
    base::{CGFloat, CGGlyph, kCGImageAlphaPremultipliedLast},
    color_space::CGColorSpace,
    context::{CGContext, CGTextDrawingMode},
    geometry::CGPoint,
};
use core_text::{
    font::CTFont,
    font_collection::CTFontCollectionRef,
    font_descriptor::{
        CTFontDescriptor, kCTFontSlantTrait, kCTFontSymbolicTrait, kCTFontWeightTrait,
        kCTFontWidthTrait,
    },
    line::CTLine,
    string_attributes::kCTFontAttributeName,
};
use font_kit::{
    font::Font as FontKitFont,
    handle::Handle,
    hinting::HintingOptions,
    metrics::Metrics,
    properties::{Style as FontkitStyle, Weight as FontkitWeight},
    source::SystemSource,
    sources::mem::MemSource,
};
use gpui::{
    Bounds, DevicePixels, Font, FontFallbacks, FontFeatures, FontId, FontMetrics, FontRun,
    FontStyle, FontWeight, GlyphId, LineLayout, Pixels, PlatformTextSystem, RenderGlyphParams,
    Result, SUBPIXEL_VARIANTS_X, ShapedGlyph, ShapedRun, SharedString, Size, TextRenderingMode,
    point, px, size, swap_rgba_pa_to_bgra,
};
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use pathfinder_geometry::{
    rect::{RectF, RectI},
    transform2d::Transform2F,
    vector::Vector2F,
};
use smallvec::SmallVec;
use std::{borrow::Cow, char, convert::TryFrom, sync::Arc};

#[cfg(target_os = "macos")]
use {
    core_foundation::base::CFType,
    gpui::{Hsla, Rgba},
    std::sync::OnceLock,
};

use crate::open_type::apply_features_and_fallbacks;

#[allow(non_upper_case_globals)]
const kCGImageAlphaOnly: u32 = 7;

/// Apple text system using CoreText for font shaping.
pub struct AppleTextSystem(RwLock<AppleTextSystemState>);

#[derive(Clone, PartialEq, Eq, Hash)]
struct FontKey {
    font_family: SharedString,
    font_features: FontFeatures,
    font_fallbacks: Option<FontFallbacks>,
}

struct AppleTextSystemState {
    memory_source: MemSource,
    system_source: SystemSource,
    fonts: Vec<FontKitFont>,
    bitmap_color_fonts: HashSet<FontId>,
    font_selections: HashMap<Font, FontId>,
    font_ids_by_postscript_name: HashMap<String, FontId>,
    font_ids_by_font_key: HashMap<FontKey, SmallVec<[FontId; 4]>>,
    postscript_names_by_font_id: HashMap<FontId, String>,
}

impl AppleTextSystem {
    /// Create a new AppleTextSystem.
    pub fn new() -> Self {
        Self(RwLock::new(AppleTextSystemState {
            memory_source: MemSource::empty(),
            system_source: SystemSource::new(),
            fonts: Vec::new(),
            bitmap_color_fonts: HashSet::default(),
            font_selections: HashMap::default(),
            font_ids_by_postscript_name: HashMap::default(),
            font_ids_by_font_key: HashMap::default(),
            postscript_names_by_font_id: HashMap::default(),
        }))
    }
}

impl Default for AppleTextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformTextSystem for AppleTextSystem {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        self.0.write().add_fonts(fonts)
    }

    fn all_font_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let collection = core_text::font_collection::create_for_all_families();
        // NOTE: We intentionally avoid using `collection.get_descriptors()` here because
        // it has a memory leak bug in core-text v21.0.0. The upstream code uses
        // `wrap_under_get_rule` but `CTFontCollectionCreateMatchingFontDescriptors`
        // follows the Create Rule (caller owns the result), so it should use
        // `wrap_under_create_rule`. We call the function directly with correct memory management.
        unsafe extern "C" {
            fn CTFontCollectionCreateMatchingFontDescriptors(
                collection: CTFontCollectionRef,
            ) -> CFArrayRef;
        }
        let descriptors: Option<CFArray<CTFontDescriptor>> = unsafe {
            let array_ref =
                CTFontCollectionCreateMatchingFontDescriptors(collection.as_concrete_TypeRef());
            if array_ref.is_null() {
                None
            } else {
                Some(CFArray::wrap_under_create_rule(array_ref))
            }
        };
        let Some(descriptors) = descriptors else {
            return names;
        };
        for descriptor in descriptors.into_iter() {
            names.extend(lenient_font_attributes::family_name(&descriptor));
        }
        if let Ok(fonts_in_memory) = self.0.read().memory_source.all_families() {
            names.extend(fonts_in_memory);
        }
        names
    }

    fn font_id(&self, font: &Font) -> Result<FontId> {
        let lock = self.0.upgradable_read();
        if let Some(font_id) = lock.font_selections.get(font) {
            Ok(*font_id)
        } else {
            let mut lock = RwLockUpgradableReadGuard::upgrade(lock);
            let font_key = FontKey {
                font_family: font.family.clone(),
                font_features: font.features.clone(),
                font_fallbacks: font.fallbacks.clone(),
            };
            let candidates = if let Some(font_ids) = lock.font_ids_by_font_key.get(&font_key) {
                font_ids.as_slice()
            } else {
                let font_ids =
                    lock.load_family(&font.family, &font.features, font.fallbacks.as_ref())?;
                lock.font_ids_by_font_key.insert(font_key.clone(), font_ids);
                lock.font_ids_by_font_key[&font_key].as_ref()
            };

            let candidate_properties = candidates
                .iter()
                .map(|font_id| lock.fonts[font_id.0].properties())
                .collect::<SmallVec<[_; 4]>>();

            let ix = font_kit::matching::find_best_match(
                &candidate_properties,
                &font_kit::properties::Properties {
                    style: fontkit_style(font.style),
                    weight: fontkit_weight(font.weight),
                    stretch: Default::default(),
                },
            )?;

            let font_id = candidates[ix];
            lock.font_selections.insert(font.clone(), font_id);
            Ok(font_id)
        }
    }

    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        font_kit_metrics_to_metrics(self.0.read().fonts[font_id.0].metrics())
    }

    fn typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Bounds<f32>> {
        Ok(bounds_from_rect(
            self.0.read().fonts[font_id.0].typographic_bounds(glyph_id.0)?,
        ))
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        self.0.read().advance(font_id, glyph_id)
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        self.0.read().glyph_for_char(font_id, ch)
    }

    fn glyph_raster_bounds(&self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        self.0.read().raster_bounds(params)
    }

    fn rasterize_glyph(
        &self,
        glyph_id: &RenderGlyphParams,
        raster_bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        self.0.read().rasterize_glyph(glyph_id, raster_bounds)
    }

    fn layout_line(&self, text: &str, font_size: Pixels, font_runs: &[FontRun]) -> LineLayout {
        self.0.write().layout_line(text, font_size, font_runs)
    }

    fn layout_line_with_options(
        &self,
        text: &str,
        font_size: Pixels,
        runs: &[FontRun],
        options: gpui::LineLayoutOptions,
    ) -> Result<gpui::DetailedLineLayout> {
        self.0
            .write()
            .layout_line_detailed(text, font_size, runs, options)
    }

    fn recommended_rendering_mode(
        &self,
        _font_id: FontId,
        _font_size: Pixels,
    ) -> TextRenderingMode {
        TextRenderingMode::Grayscale
    }

    #[cfg(target_os = "macos")]
    fn glyph_dilation_for_color(&self, color: Hsla) -> u8 {
        // When font smoothing is enabled, CoreGraphics thickens glyph strokes by an amount that
        // depends on the foreground color's luminance. We replicate the logic used by CoreGraphics
        // to select between the different levels of dilation.
        if !font_smoothing_allowed_by_user() {
            return 0;
        }
        let rgba: Rgba = color.into();
        let luminance = 0.2126 * rgba.r + 0.7152 * rgba.g + 0.0722 * rgba.b;
        let level = ((4.0 * luminance) + 0.5).floor() as i32;
        level.clamp(0, 4) as u8
    }
}

#[cfg(target_os = "macos")]
fn font_smoothing_allowed_by_user() -> bool {
    static ALLOWED: OnceLock<bool> = OnceLock::new();
    *ALLOWED.get_or_init(|| {
        use core_foundation_sys::preferences::{
            CFPreferencesCopyAppValue, kCFPreferencesCurrentApplication,
        };

        let key = CFString::new("AppleFontSmoothing");
        let value_ref = unsafe {
            CFPreferencesCopyAppValue(key.as_concrete_TypeRef(), kCFPreferencesCurrentApplication)
        };
        if value_ref.is_null() {
            return true;
        }
        let value = unsafe { CFType::wrap_under_create_rule(value_ref) };
        let Some(number) = value.downcast_into::<CFNumber>() else {
            return true;
        };
        // Only an explicit value of `0` means that font smoothing is disabled.
        number.to_i64() != Some(0)
    })
}

impl AppleTextSystemState {
    fn add_fonts(&mut self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        let fonts = fonts
            .into_iter()
            .map(|bytes| match bytes {
                Cow::Borrowed(embedded_font) => {
                    let data_provider = unsafe {
                        core_graphics::data_provider::CGDataProvider::from_slice(embedded_font)
                    };
                    let font = core_graphics::font::CGFont::from_data_provider(data_provider)
                        .map_err(|()| anyhow!("Could not load an embedded font."))?;
                    let font = font_kit::loaders::core_text::Font::from_core_graphics_font(font);
                    Ok(Handle::from_native(&font))
                }
                Cow::Owned(bytes) => Ok(Handle::from_memory(Arc::new(bytes), 0)),
            })
            .collect::<Result<Vec<_>>>()?;
        self.memory_source.add_fonts(fonts.into_iter())?;
        // Previously selected fonts may carry a cascade built before a newly
        // registered app-private fallback existed. Rebuild selections without
        // removing font IDs still referenced by already-shaped frames.
        self.font_selections.clear();
        self.font_ids_by_font_key.clear();
        Ok(())
    }

    fn load_family(
        &mut self,
        name: &str,
        features: &FontFeatures,
        fallbacks: Option<&FontFallbacks>,
    ) -> Result<SmallVec<[FontId; 4]>> {
        let name = gpui::font_name_with_fallbacks(name, ".AppleSystemUIFont");

        let mut font_ids = SmallVec::new();
        let mut postscript_names_seen = HashSet::default();
        let family = self
            .memory_source
            .select_family_by_name(name)
            .or_else(|_| self.system_source.select_family_by_name(name))?;
        for font in family.fonts() {
            let mut font = font.load()?;
            let concrete_fallbacks = fallbacks
                .into_iter()
                .flat_map(|fallbacks| fallbacks.fallback_list())
                .map(|name| {
                    self.memory_source
                        .select_best_match(
                            &[font_kit::family_name::FamilyName::Title(name.clone())],
                            &font.properties(),
                        )
                        .ok()
                        .and_then(|handle| handle.load().ok())
                })
                .collect::<Vec<_>>();

            apply_features_and_fallbacks(&mut font, features, fallbacks, &concrete_fallbacks)?;
            // A valid non-Latin font need not contain 'm'. Text metrics callers
            // use the platform fallback when their Latin measurement probe is absent.

            // We've seen a number of panics in production caused by calling font.properties()
            // which unwraps a downcast to CFNumber. This is an attempt to avoid the panic,
            // and to try and identify the incalcitrant font.
            let traits = font.native_font().all_traits();
            if unsafe {
                !(traits
                    .get(kCTFontSymbolicTrait)
                    .downcast::<CFNumber>()
                    .is_some()
                    && traits
                        .get(kCTFontWidthTrait)
                        .downcast::<CFNumber>()
                        .is_some()
                    && traits
                        .get(kCTFontWeightTrait)
                        .downcast::<CFNumber>()
                        .is_some()
                    && traits
                        .get(kCTFontSlantTrait)
                        .downcast::<CFNumber>()
                        .is_some())
            } {
                log::error!(
                    "Failed to read traits for font {:?} (PostScript name {:?})",
                    font.full_name(),
                    font.postscript_name(),
                );
                continue;
            }

            let Some(postscript_name) = font.postscript_name() else {
                log::warn!(
                    "font {:?} in family {:?} has no PostScript name; skipping",
                    font.full_name(),
                    name,
                );
                continue;
            };
            // Dedup is scoped to this single `load_family` call (issue #55472).
            // The same family can be reloaded later under a different `FontKey`
            // (different features/fallbacks); a global check against
            // `font_ids_by_postscript_name` would skip every already-registered
            // font and leave the second call's `font_ids` empty.
            if !postscript_names_seen.insert(postscript_name.clone()) {
                log::warn!(
                    "skipping duplicate font {:?} with PostScript name {:?} \
                     in family {:?}",
                    font.full_name(),
                    postscript_name,
                    name,
                );
                continue;
            }
            let font_id = FontId(self.fonts.len());
            font_ids.push(font_id);
            self.font_ids_by_postscript_name
                .insert(postscript_name.clone(), font_id);
            self.postscript_names_by_font_id
                .insert(font_id, postscript_name);
            if has_sbix_table(&font) {
                self.bitmap_color_fonts.insert(font_id);
            }
            self.fonts.push(font);
        }
        Ok(font_ids)
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        Ok(size_from_vector2f(
            self.fonts[font_id.0].advance(glyph_id.0)?,
        ))
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        self.fonts[font_id.0].glyph_for_char(ch).map(GlyphId)
    }

    fn id_for_native_font(&mut self, requested_font: CTFont) -> FontId {
        let postscript_name = requested_font.postscript_name();
        if let Some(font_id) = self.font_ids_by_postscript_name.get(&postscript_name) {
            *font_id
        } else {
            let font_id = FontId(self.fonts.len());
            self.font_ids_by_postscript_name
                .insert(postscript_name.clone(), font_id);
            self.postscript_names_by_font_id
                .insert(font_id, postscript_name);
            let font =
                font_kit::font::Font::from_core_graphics_font(requested_font.copy_to_CGFont());
            if has_sbix_table(&font) {
                self.bitmap_color_fonts.insert(font_id);
            }
            self.fonts.push(font);
            font_id
        }
    }

    fn is_emoji(&self, font_id: FontId) -> bool {
        self.bitmap_color_fonts.contains(&font_id)
            || self
                .postscript_names_by_font_id
                .get(&font_id)
                .is_some_and(|postscript_name| {
                    postscript_name == "AppleColorEmoji" || postscript_name == ".AppleColorEmojiUI"
                })
    }

    fn raster_bounds(&self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        let font = &self.fonts[params.font_id.0];
        let scale = Transform2F::from_scale(params.scale_factor);
        let bounds: Bounds<DevicePixels> = bounds_from_rect_i(font.raster_bounds(
            params.glyph_id.0,
            params.font_size.into(),
            scale,
            HintingOptions::None,
            font_kit::canvas::RasterizationOptions::GrayscaleAa,
        )?);

        // Expand the bounds by 1 pixel on each side to give CG room for anti-aliasing.
        Ok(bounds.dilate(DevicePixels(1)))
    }

    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        glyph_bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        if glyph_bounds.size.width.0 == 0 || glyph_bounds.size.height.0 == 0 {
            anyhow::bail!("glyph bounds are empty");
        } else {
            // Add an extra pixel when the subpixel variant isn't zero to make room for anti-aliasing.
            let mut bitmap_size = glyph_bounds.size;
            if params.subpixel_variant.x > 0 {
                bitmap_size.width += DevicePixels(1);
            }
            if params.subpixel_variant.y > 0 {
                bitmap_size.height += DevicePixels(1);
            }
            let bitmap_size = bitmap_size;

            let mut bytes;
            let cx;
            if params.is_emoji {
                bytes = vec![0; bitmap_size.width.0 as usize * 4 * bitmap_size.height.0 as usize];
                cx = CGContext::create_bitmap_context(
                    Some(bytes.as_mut_ptr() as *mut _),
                    bitmap_size.width.0 as usize,
                    bitmap_size.height.0 as usize,
                    8,
                    bitmap_size.width.0 as usize * 4,
                    &CGColorSpace::create_device_rgb(),
                    kCGImageAlphaPremultipliedLast,
                );
            } else {
                bytes = vec![0; bitmap_size.width.0 as usize * bitmap_size.height.0 as usize];
                cx = CGContext::create_bitmap_context(
                    Some(bytes.as_mut_ptr() as *mut _),
                    bitmap_size.width.0 as usize,
                    bitmap_size.height.0 as usize,
                    8,
                    bitmap_size.width.0 as usize,
                    &CGColorSpace::create_device_gray(),
                    kCGImageAlphaOnly,
                );
            }

            // Move the origin to bottom left and account for scaling, this
            // makes drawing text consistent with the font-kit's raster_bounds.
            cx.translate(
                -glyph_bounds.origin.x.0 as CGFloat,
                (glyph_bounds.origin.y.0 + glyph_bounds.size.height.0) as CGFloat,
            );
            cx.scale(
                params.scale_factor as CGFloat,
                params.scale_factor as CGFloat,
            );

            let subpixel_shift = params
                .subpixel_variant
                .map(|v| v as f32 / SUBPIXEL_VARIANTS_X as f32);
            cx.set_text_drawing_mode(CGTextDrawingMode::CGTextFill);
            cx.set_allows_antialiasing(true);
            cx.set_should_antialias(true);
            cx.set_allows_font_subpixel_positioning(true);
            cx.set_should_subpixel_position_fonts(true);
            cx.set_allows_font_subpixel_quantization(false);
            cx.set_should_subpixel_quantize_fonts(false);

            if cfg!(target_os = "macos") && params.dilation > 0 {
                let luminance = params.dilation as f64 * 0.25;
                cx.set_should_smooth_fonts(true);
                cx.set_gray_fill_color(luminance, 1.0);
            } else {
                cx.set_gray_fill_color(0.0, 1.0);
            }
            self.fonts[params.font_id.0]
                .native_font()
                .clone_with_font_size(f32::from(params.font_size) as CGFloat)
                .draw_glyphs(
                    &[params.glyph_id.0 as CGGlyph],
                    &[CGPoint::new(
                        (subpixel_shift.x / params.scale_factor) as CGFloat,
                        (subpixel_shift.y / params.scale_factor) as CGFloat,
                    )],
                    cx,
                );

            if params.is_emoji {
                // Convert from RGBA with premultiplied alpha to BGRA with straight alpha.
                for pixel in bytes.chunks_exact_mut(4) {
                    swap_rgba_pa_to_bgra(pixel);
                }
            }

            Ok((bitmap_size, bytes))
        }
    }

    fn layout_line(&mut self, text: &str, font_size: Pixels, font_runs: &[FontRun]) -> LineLayout {
        // Construct the attributed string, converting UTF8 ranges to UTF16 ranges.
        let mut string = CFMutableAttributedString::new();
        let mut max_ascent = 0.0f32;
        let mut max_descent = 0.0f32;

        {
            let mut text = text;
            let mut break_ligature = true;
            for run in font_runs {
                let text_run;
                (text_run, text) = text.split_at(run.len);

                let utf16_start = string.char_len(); // insert at end of string
                // note: replace_str may silently ignore codepoints it dislikes (e.g., BOM at start of string)
                string.replace_str(&CFString::new(text_run), CFRange::init(utf16_start, 0));
                let utf16_end = string.char_len();

                let length = utf16_end - utf16_start;
                let cf_range = CFRange::init(utf16_start, length);
                let font = &self.fonts[run.font_id.0];

                let font_metrics = font.metrics();
                let font_scale = f32::from(font_size) / font_metrics.units_per_em as f32;
                max_ascent = max_ascent.max(font_metrics.ascent * font_scale);
                max_descent = max_descent.max(-font_metrics.descent * font_scale);

                let font_size = if break_ligature {
                    px(f32::from(font_size).next_up())
                } else {
                    font_size
                };
                unsafe {
                    string.set_attribute(
                        cf_range,
                        kCTFontAttributeName,
                        &font.native_font().clone_with_font_size(font_size.into()),
                    );
                }
                break_ligature = !break_ligature;
            }
        }
        // Retrieve the glyphs from the shaped line, converting UTF16 offsets to UTF8 offsets.
        let line = CTLine::new_with_attributed_string(string.as_concrete_TypeRef());
        let glyph_runs = line.glyph_runs();
        let mut runs = <Vec<ShapedRun>>::with_capacity(glyph_runs.len() as usize);
        let mut ix_converter = StringIndexConverter::new(text);
        for run in glyph_runs.into_iter() {
            let attributes = run.attributes().unwrap();
            let font = unsafe {
                attributes
                    .get(kCTFontAttributeName)
                    .downcast::<CTFont>()
                    .unwrap()
            };
            let font_id = self.id_for_native_font(font);

            let glyphs = match runs.last_mut() {
                Some(run) if run.font_id == font_id => &mut run.glyphs,
                _ => {
                    runs.push(ShapedRun {
                        font_id,
                        glyphs: Vec::with_capacity(run.glyph_count().try_into().unwrap_or(0)),
                    });
                    &mut runs.last_mut().unwrap().glyphs
                }
            };
            for ((&glyph_id, position), &glyph_utf16_ix) in run
                .glyphs()
                .iter()
                .zip(run.positions().iter())
                .zip(run.string_indices().iter())
            {
                let glyph_utf16_ix = usize::try_from(glyph_utf16_ix).unwrap();
                if ix_converter.utf16_ix > glyph_utf16_ix {
                    // We cannot reuse current index converter, as it can only seek forward. Restart the search.
                    ix_converter = StringIndexConverter::new(text);
                }
                ix_converter.advance_to_utf16_ix(glyph_utf16_ix);
                glyphs.push(ShapedGlyph {
                    id: GlyphId(glyph_id as u32),
                    position: point(position.x as f32, position.y as f32).map(px),
                    index: ix_converter.utf8_ix,
                    is_emoji: self.is_emoji(font_id),
                });
            }
        }
        let typographic_bounds = line.get_typographic_bounds();
        LineLayout {
            runs,
            font_size,
            width: typographic_bounds.width.into(),
            ascent: max_ascent.into(),
            descent: max_descent.into(),
            len: text.len(),
        }
    }
}

#[repr(C)]
struct DetailedParagraphSetting {
    spec: u32,
    value_size: usize,
    value: *const std::ffi::c_void,
}

// block2 deliberately does not encode Rust bool as Objective-C BOOL. This
// callback uses C99 bool, whose ABI and encoding are preserved explicitly.
#[repr(transparent)]
#[derive(Clone, Copy)]
struct CoreTextCaretBool(bool);
unsafe impl objc2::encode::Encode for CoreTextCaretBool {
    const ENCODING: objc2::encode::Encoding = objc2::encode::Encoding::Bool;
}
unsafe impl objc2::encode::RefEncode for CoreTextCaretBool {
    const ENCODING_REF: objc2::encode::Encoding =
        objc2::encode::Encoding::Pointer(&<Self as objc2::encode::Encode>::ENCODING);
}

#[link(name = "CoreText", kind = "framework")]
unsafe extern "C" {
    fn CTParagraphStyleCreate(
        settings: *const DetailedParagraphSetting,
        count: usize,
    ) -> core_foundation::base::CFTypeRef;
    fn CTLineEnumerateCaretOffsets(
        line: core_text::line::CTLineRef,
        block: &block2::DynBlock<
            dyn Fn(f64, isize, CoreTextCaretBool, *mut CoreTextCaretBool) + '_,
        >,
    );
    fn CTRunGetStatus(run: core_text::run::CTRunRef) -> u32;
}

impl AppleTextSystemState {
    fn layout_line_detailed(
        &mut self,
        text: &str,
        font_size: Pixels,
        font_runs: &[FontRun],
        options: gpui::LineLayoutOptions,
    ) -> Result<gpui::DetailedLineLayout> {
        use core_foundation::base::{CFType, kCFAllocatorDefault};
        use core_foundation::string::CFStringCreateWithCharacters;
        use core_text::string_attributes::kCTParagraphStyleAttributeName;
        use gpui::{
            DetailedLineLayout, GlyphAddress, ParagraphDirection, ShapedCluster,
            merge_grapheme_clusters,
        };
        anyhow::ensure!(
            f32::from(font_size).is_finite() && font_size > px(0.),
            "invalid font size"
        );
        let base_level = match options.paragraph_direction {
            ParagraphDirection::Auto => None,
            ParagraphDirection::Ltr => Some(unicode_bidi::Level::ltr()),
            ParagraphDirection::Rtl => Some(unicode_bidi::Level::rtl()),
        };
        let bidi = unicode_bidi::BidiInfo::new(text, base_level);
        anyhow::ensure!(
            bidi.paragraphs.len() <= 1,
            "detailed layout requires one paragraph"
        );
        let resolved = bidi
            .paragraphs
            .first()
            .map(|p| p.level)
            .or(base_level)
            .unwrap_or_else(unicode_bidi::Level::ltr);
        let paragraph_direction = if resolved.is_rtl() {
            ParagraphDirection::Rtl
        } else {
            ParagraphDirection::Ltr
        };
        let levels = bidi
            .paragraphs
            .first()
            .map(|paragraph| bidi.reordered_levels(paragraph, 0..text.len()))
            .unwrap_or_default();

        // UTF-16 construction preserves the source verbatim, including an initial
        // BOM; CFString's UTF-8 constructor may consume that marker.
        let utf16: Vec<u16> = text.encode_utf16().collect();
        let string_value: CFString = unsafe {
            CFString::wrap_under_create_rule(CFStringCreateWithCharacters(
                kCFAllocatorDefault,
                utf16.as_ptr(),
                utf16.len() as _,
            ))
        };
        let mut string = CFMutableAttributedString::new();
        string.replace_str(&string_value, CFRange::init(0, 0));
        anyhow::ensure!(
            string.char_len() as usize == utf16.len(),
            "CoreText changed source length"
        );
        let mut utf16_to_utf8 = vec![None; utf16.len() + 1];
        let mut utf8_to_utf16 = vec![None; text.len() + 1];
        let mut utf16_index = 0;
        for (byte, ch) in text.char_indices() {
            utf16_to_utf8[utf16_index] = Some(byte);
            utf8_to_utf16[byte] = Some(utf16_index);
            utf16_index += ch.len_utf16();
        }
        utf16_to_utf8[utf16.len()] = Some(text.len());
        utf8_to_utf16[text.len()] = Some(utf16.len());
        let mut byte_end: usize = 0;
        let mut utf16_start = 0;
        let mut max_ascent = 0.0_f32;
        let mut max_descent = 0.0_f32;
        for run in font_runs {
            let next = byte_end
                .checked_add(run.len)
                .ok_or_else(|| anyhow!("font run length overflow"))?;
            let slice = text
                .get(byte_end..next)
                .ok_or_else(|| anyhow!("invalid font run boundary"))?;
            let length = slice.encode_utf16().count();
            let font = self
                .fonts
                .get(run.font_id.0)
                .ok_or_else(|| anyhow!("unknown font id"))?;
            let metrics = font.metrics();
            let scale = f32::from(font_size) / metrics.units_per_em as f32;
            max_ascent = max_ascent.max(metrics.ascent * scale);
            max_descent = max_descent.max(-metrics.descent * scale);
            unsafe {
                string.set_attribute(
                    CFRange::init(utf16_start as _, length as _),
                    kCTFontAttributeName,
                    &font.native_font().clone_with_font_size(font_size.into()),
                );
            }
            byte_end = next;
            utf16_start += length;
        }
        anyhow::ensure!(byte_end == text.len(), "font runs must cover the line");
        if !text.is_empty() {
            let direction: i8 = if resolved.is_rtl() { 1 } else { 0 };
            let setting = DetailedParagraphSetting {
                spec: 13,
                value_size: std::mem::size_of::<i8>(),
                value: (&direction as *const i8).cast(),
            };
            let style_ref = unsafe { CTParagraphStyleCreate(&setting, 1) };
            anyhow::ensure!(
                !style_ref.is_null(),
                "CoreText paragraph style creation failed"
            );
            let style: CFType = unsafe { CFType::wrap_under_create_rule(style_ref) };
            unsafe {
                string.set_attribute(
                    CFRange::init(0, utf16.len() as _),
                    kCTParagraphStyleAttributeName,
                    &style,
                );
            }
        }
        let line = CTLine::new_with_attributed_string(string.as_concrete_TypeRef());
        let mut runs: Vec<ShapedRun> = Vec::new();
        let mut raw = Vec::new();
        let mut starts = Vec::new();
        for native_run in line.glyph_runs().into_iter() {
            let attributes = native_run
                .attributes()
                .ok_or_else(|| anyhow!("CoreText run has no attributes"))?;
            let font = unsafe {
                attributes
                    .get(kCTFontAttributeName)
                    .downcast::<CTFont>()
                    .ok_or_else(|| anyhow!("CoreText run has no font"))?
            };
            let font_id = self.id_for_native_font(font);
            let ids = native_run.glyphs();
            let positions = native_run.positions();
            let indices = native_run.string_indices();
            let native_rtl = unsafe { CTRunGetStatus(native_run.as_concrete_TypeRef()) & 1 != 0 };
            if runs.last().is_none_or(|run| run.font_id != font_id) {
                runs.push(ShapedRun {
                    font_id,
                    glyphs: Vec::new(),
                });
            }
            let run_index = runs.len() - 1;
            for index in 0..ids.len() {
                let utf16_index = usize::try_from(indices[index])
                    .map_err(|_| anyhow!("negative CoreText source index"))?;
                let byte = utf16_to_utf8
                    .get(utf16_index)
                    .copied()
                    .flatten()
                    .ok_or_else(|| anyhow!("CoreText index splits a UTF-16 scalar"))?;
                anyhow::ensure!(byte < text.len(), "CoreText glyph outside source");
                let level = levels[byte];
                // UAX #9 rule X9 excludes these controls from reordering; a
                // native run may attach them to either adjacent direction.
                let excluded = matches!(
                    bidi.original_classes[byte],
                    unicode_bidi::BidiClass::BN
                        | unicode_bidi::BidiClass::LRE
                        | unicode_bidi::BidiClass::RLE
                        | unicode_bidi::BidiClass::LRO
                        | unicode_bidi::BidiClass::RLO
                        | unicode_bidi::BidiClass::PDF
                );
                anyhow::ensure!(
                    excluded || level.is_rtl() == native_rtl,
                    "CoreText run direction disagrees with UAX #9 embedding level at byte {byte}"
                );
                let glyph_index = runs[run_index].glyphs.len();
                runs[run_index].glyphs.push(ShapedGlyph {
                    id: GlyphId(ids[index] as u32),
                    position: point(px(positions[index].x as f32), px(positions[index].y as f32)),
                    index: byte,
                    is_emoji: self.is_emoji(font_id),
                });
                starts.push(byte);
                raw.push((
                    byte,
                    GlyphAddress {
                        run_index,
                        glyph_index,
                    },
                    px(0.)..px(0.),
                    level.number(),
                ));
            }
        }
        starts.push(text.len());
        starts.sort_unstable();
        starts.dedup();
        let mut clusters = Vec::with_capacity(raw.len());
        for (start, address, advance, bidi_level) in raw {
            let next = starts[starts.binary_search(&start).unwrap() + 1];
            clusters.push(ShapedCluster {
                source: start..next,
                glyphs: vec![address],
                advance,
                bidi_level,
            });
        }
        let mut clusters = merge_grapheme_clusters(text, clusters)?;
        // CoreText glyph advances include GPOS placement/backtracking (Arabic
        // marks can compensate in the *next* source cluster). Logical caret
        // edges give the actual indivisible cell, with explicit edge affinity
        // even at a bidi boundary; glyph positions remain untouched for paint.
        let caret_edges = std::cell::RefCell::new(vec![(None, None); utf16.len()]);
        let callback = block2::RcBlock::new(
            |offset: f64, index: isize, leading: CoreTextCaretBool, _: *mut CoreTextCaretBool| {
                if let Ok(index) = usize::try_from(index) {
                    if let Some(edges) = caret_edges.borrow_mut().get_mut(index) {
                        if leading.0 {
                            edges.0 = Some(offset);
                        } else {
                            edges.1 = Some(offset);
                        }
                    }
                }
            },
        );
        unsafe {
            CTLineEnumerateCaretOffsets(line.as_concrete_TypeRef(), &callback);
        }
        drop(callback);
        let caret_edges = caret_edges.into_inner();
        for cluster in &mut clusters {
            let start = utf8_to_utf16[cluster.source.start]
                .ok_or_else(|| anyhow!("invalid cluster start"))?;
            let end =
                utf8_to_utf16[cluster.source.end].ok_or_else(|| anyhow!("invalid cluster end"))?;
            let leading = caret_edges[start]
                .0
                .ok_or_else(|| anyhow!("CoreText omitted cluster leading caret"))?;
            let trailing = caret_edges[end - 1]
                .1
                .ok_or_else(|| anyhow!("CoreText omitted cluster trailing caret"))?;
            cluster.advance = px(leading.min(trailing) as f32)..px(leading.max(trailing) as f32);
        }
        let result = DetailedLineLayout {
            layout: Arc::new(LineLayout {
                runs,
                font_size,
                width: line.get_typographic_bounds().width.into(),
                ascent: max_ascent.into(),
                descent: max_descent.into(),
                len: text.len(),
            }),
            clusters,
            paragraph_direction,
        };
        result.validate(text, options)?;
        Ok(result)
    }
}

#[derive(Debug, Clone)]
struct StringIndexConverter<'a> {
    text: &'a str,
    /// Index in UTF-8 bytes
    utf8_ix: usize,
    /// Index in UTF-16 code units
    utf16_ix: usize,
}

impl<'a> StringIndexConverter<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            utf8_ix: 0,
            utf16_ix: 0,
        }
    }

    fn advance_to_utf16_ix(&mut self, utf16_target: usize) {
        for (ix, c) in self.text[self.utf8_ix..].char_indices() {
            if self.utf16_ix >= utf16_target {
                self.utf8_ix += ix;
                return;
            }
            self.utf16_ix += c.len_utf16();
        }
        self.utf8_ix = self.text.len();
    }
}

fn font_kit_metrics_to_metrics(metrics: Metrics) -> FontMetrics {
    FontMetrics {
        units_per_em: metrics.units_per_em,
        ascent: metrics.ascent,
        descent: metrics.descent,
        line_gap: metrics.line_gap,
        underline_position: metrics.underline_position,
        underline_thickness: metrics.underline_thickness,
        cap_height: metrics.cap_height,
        x_height: metrics.x_height,
        bounding_box: bounds_from_rect(metrics.bounding_box),
    }
}

fn bounds_from_rect(rect: RectF) -> Bounds<f32> {
    Bounds {
        origin: point(rect.origin_x(), rect.origin_y()),
        size: size(rect.width(), rect.height()),
    }
}

fn bounds_from_rect_i(rect: RectI) -> Bounds<DevicePixels> {
    Bounds {
        origin: point(DevicePixels(rect.origin_x()), DevicePixels(rect.origin_y())),
        size: size(DevicePixels(rect.width()), DevicePixels(rect.height())),
    }
}

// impl From<Vector2I> for Size<DevicePixels> {
//     fn from(value: Vector2I) -> Self {
//         size(value.x().into(), value.y().into())
//     }
// }

// impl From<RectI> for Bounds<i32> {
//     fn from(rect: RectI) -> Self {
//         Bounds {
//             origin: point(rect.origin_x(), rect.origin_y()),
//             size: size(rect.width(), rect.height()),
//         }
//     }
// }

// impl From<Point<u32>> for Vector2I {
//     fn from(size: Point<u32>) -> Self {
//         Vector2I::new(size.x as i32, size.y as i32)
//     }
// }

fn size_from_vector2f(vec: Vector2F) -> Size<f32> {
    size(vec.x(), vec.y())
}

fn fontkit_weight(value: FontWeight) -> FontkitWeight {
    FontkitWeight(value.0)
}

fn fontkit_style(style: FontStyle) -> FontkitStyle {
    match style {
        FontStyle::Normal => FontkitStyle::Normal,
        FontStyle::Italic => FontkitStyle::Italic,
        FontStyle::Oblique => FontkitStyle::Oblique,
    }
}

// Some fonts may have no attributes despite `core_text` requiring them (and panicking).
// This is the same version as `core_text` has without `expect` calls.
mod lenient_font_attributes {
    use core_foundation::{
        base::{CFRetain, CFType, TCFType},
        string::{CFString, CFStringRef},
    };
    use core_text::font_descriptor::{
        CTFontDescriptor, CTFontDescriptorCopyAttribute, kCTFontFamilyNameAttribute,
    };

    pub fn family_name(descriptor: &CTFontDescriptor) -> Option<String> {
        unsafe { get_string_attribute(descriptor, kCTFontFamilyNameAttribute) }
    }

    fn get_string_attribute(
        descriptor: &CTFontDescriptor,
        attribute: CFStringRef,
    ) -> Option<String> {
        unsafe {
            let value = CTFontDescriptorCopyAttribute(descriptor.as_concrete_TypeRef(), attribute);
            if value.is_null() {
                return None;
            }

            let value = CFType::wrap_under_create_rule(value);
            assert!(value.instance_of::<CFString>());
            let s = wrap_under_get_rule(value.as_CFTypeRef() as CFStringRef);
            Some(s.to_string())
        }
    }

    unsafe fn wrap_under_get_rule(reference: CFStringRef) -> CFString {
        unsafe {
            assert!(!reference.is_null(), "Attempted to create a NULL object.");
            let reference = CFRetain(reference as *const ::std::os::raw::c_void) as CFStringRef;
            TCFType::wrap_under_create_rule(reference)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::AppleTextSystem;
    use gpui::{FontFallbacks, FontFeatures, FontRun, GlyphId, PlatformTextSystem, font, px};
    use std::sync::Arc;

    #[test]
    fn test_reload_family_with_features_and_fallbacks() {
        let fonts = AppleTextSystem::new();
        let mut font = font("Helvetica");
        let original_id = fonts.font_id(&font).unwrap();
        font.features = FontFeatures::disable_ligatures();
        font.fallbacks = Some(FontFallbacks(Arc::new(vec!["Times".into()])));
        let configured_id = fonts.font_id(&font).unwrap();
        assert_ne!(original_id, configured_id);
        assert_eq!(fonts.font_id(&font).unwrap(), configured_id);
        assert!(fonts.glyph_for_char(configured_id, 'm').is_some());
        let layout = fonts.layout_line(
            "office",
            px(16.),
            &[FontRun {
                font_id: configured_id,
                len: 6,
            }],
        );
        assert_eq!(layout.len, 6);
        assert!(!layout.runs.is_empty());
    }

    #[test]
    fn test_layout_line_bom_char() {
        let fonts = AppleTextSystem::new();
        let font_id = fonts.font_id(&font("Helvetica")).unwrap();
        let line = "\u{feff}";
        let mut style = FontRun {
            font_id,
            len: line.len(),
        };

        let layout = fonts.layout_line(line, px(16.), &[style]);
        assert_eq!(layout.len, line.len());
        assert!(layout.runs.is_empty());

        let line = "a\u{feff}b";
        style.len = line.len();
        let layout = fonts.layout_line(line, px(16.), &[style]);
        assert_eq!(layout.len, line.len());
        assert_eq!(layout.runs.len(), 1);
        assert_eq!(layout.runs[0].glyphs.len(), 2);
        assert_eq!(layout.runs[0].glyphs[0].id, GlyphId(68u32)); // a
        // There's no glyph for \u{feff}
        assert_eq!(layout.runs[0].glyphs[1].id, GlyphId(69u32)); // b

        let line = "\u{feff}ab";
        let font_runs = &[
            FontRun {
                len: "\u{feff}".len(),
                font_id,
            },
            FontRun {
                len: "ab".len(),
                font_id,
            },
        ];
        let layout = fonts.layout_line(line, px(16.), font_runs);
        assert_eq!(layout.len, line.len());
        assert_eq!(layout.runs.len(), 1);
        assert_eq!(layout.runs[0].glyphs.len(), 2);
        // There's no glyph for \u{feff}
        assert_eq!(layout.runs[0].glyphs[0].id, GlyphId(68u32)); // a
        assert_eq!(layout.runs[0].glyphs[1].id, GlyphId(69u32)); // b
    }

    #[test]
    fn test_layout_line_zwnj_insertion() {
        let fonts = AppleTextSystem::new();
        let font_id = fonts.font_id(&font("Helvetica")).unwrap();

        let text = "hello world";
        let font_runs = &[
            FontRun { font_id, len: 5 }, // "hello"
            FontRun { font_id, len: 6 }, // " world"
        ];

        let layout = fonts.layout_line(text, px(16.), font_runs);
        assert_eq!(layout.len, text.len());

        for run in &layout.runs {
            for glyph in &run.glyphs {
                assert!(
                    glyph.index < text.len(),
                    "Glyph index {} is out of bounds for text length {}",
                    glyph.index,
                    text.len()
                );
            }
        }

        // Test with different font runs - should not insert ZWNJ
        let font_id2 = fonts.font_id(&font("Times")).unwrap_or(font_id);
        let font_runs_different = &[
            FontRun { font_id, len: 5 }, // "hello"
            // " world"
            FontRun {
                font_id: font_id2,
                len: 6,
            },
        ];

        let layout2 = fonts.layout_line(text, px(16.), font_runs_different);
        assert_eq!(layout2.len, text.len());

        for run in &layout2.runs {
            for glyph in &run.glyphs {
                assert!(
                    glyph.index < text.len(),
                    "Glyph index {} is out of bounds for text length {}",
                    glyph.index,
                    text.len()
                );
            }
        }
    }

    #[test]
    fn test_layout_line_zwnj_edge_cases() {
        let fonts = AppleTextSystem::new();
        let font_id = fonts.font_id(&font("Helvetica")).unwrap();

        let text = "hello";
        let font_runs = &[FontRun { font_id, len: 5 }];
        let layout = fonts.layout_line(text, px(16.), font_runs);
        assert_eq!(layout.len, text.len());

        let text = "abc";
        let font_runs = &[
            FontRun { font_id, len: 1 }, // "a"
            FontRun { font_id, len: 1 }, // "b"
            FontRun { font_id, len: 1 }, // "c"
        ];
        let layout = fonts.layout_line(text, px(16.), font_runs);
        assert_eq!(layout.len, text.len());

        for run in &layout.runs {
            for glyph in &run.glyphs {
                assert!(
                    glyph.index < text.len(),
                    "Glyph index {} is out of bounds for text length {}",
                    glyph.index,
                    text.len()
                );
            }
        }

        // Test with empty text
        let text = "";
        let font_runs = &[];
        let layout = fonts.layout_line(text, px(16.), font_runs);
        assert_eq!(layout.len, 0);
        assert!(layout.runs.is_empty());
    }
}

// Inspect capability once when assigning a font ID; do not copy bitmap tables
// per glyph, and do not infer arbitrary color-font support from a family name.
fn has_sbix_table(font: &FontKitFont) -> bool {
    font.native_font()
        .get_available_font_tables()
        .is_some_and(|tables| {
            tables
                .iter()
                .any(|tag| *tag == u32::from_be_bytes(*b"sbix"))
        })
}
