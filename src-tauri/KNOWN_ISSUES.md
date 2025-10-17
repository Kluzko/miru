# Known Issues

## 1. ~~Enrichment Service Skips age_restriction for High-Quality Anime~~ ✅ FIXED

**Severity**: Medium  
**Component**: Data Enhancement Service  
**Status**: ✅ **FIXED**

### Description

When importing anime with high data quality from AniList (completeness_score >= 0.8), the enrichment service skips filling the `age_restriction` field, even though AniList never provides this field.

### Reproduction

```rust
// Import a high-quality anime from AniList
let result = ingestion_service.ingest_anime(
    AnimeSource::ManualImport {
        title: "Fullmetal Alchemist Brotherhood".to_string(),
    },
    IngestionOptions {
        skip_provider_fetch: false, // Allow enrichment
        ...
    },
).await;

// Result: age_restriction is None
assert_eq!(result.anime.age_restriction, None);
```

### Root Cause

File: `src/modules/data_import/domain/services/import_components/data_enhancement_service.rs`

Lines 262-270:
```rust
gaps.into_iter()
    .filter(|gap| {
        !quality_metrics
            .field_completeness
            .get(gap)
            .unwrap_or(&false)
            || quality_metrics.completeness_score < 0.8  // <-- Problem here
    })
    .collect()
```

The logic filters out data gaps (including `age_restriction`) if:
- The field is marked as complete in field_completeness map, OR
- The overall completeness_score < 0.8

For high-quality anime like FMAB:
- completeness_score = 0.95 (has score, synopsis, genres, studios, etc.)
- age_restriction gap is filtered out
- Enrichment is skipped

### Impact

- age_restriction remains None for high-quality anime
- Users cannot filter by age rating for well-known anime
- Affects ~20% of anime (top-rated, popular series)

### Workaround

None currently. age_restriction can only be populated for lower-quality anime that trigger enrichment.

### Fix Applied

**Files Modified:**
1. `data_enhancement_service.rs:277` - Added CRITICAL_GAPS list with age_restriction
2. `data_enhancement_service.rs:282-291` - Updated gap filtering to always include critical gaps
3. `data_enhancement_service.rs:95-122` - Added fallback logic to search ALL provider results for age_restriction

**Key Changes:**
- Critical fields (age_restriction) always trigger enrichment regardless of quality score
- If best match doesn't have age_restriction, check ALL search results from all providers
- This ensures we get age_restriction from Jikan even when AniList was the best match

### Testing

Real-world e2e test in `tests/stage_2_1_real_data_e2e_test.rs` now passes with age_restriction populated:

```
✓ Imported: Hagane no Renkinjutsushi: FULLMETAL ALCHEMIST
  - Age Restriction: Some(ParentalGuidance17)
✓ Age restriction populated: ParentalGuidance17 (enrichment service worked)
✓ Enrichment service is working (age_restriction populated from Jikan)
```

Test verifies:
- age_restriction is populated from Jikan even for high-quality anime from AniList
- Enrichment service now checks ALL provider results, not just best match
- Critical fields are never skipped regardless of overall data quality

### Related

- AniList API does not provide age_restriction
- Jikan (MyAnimeList) API does provide rating field
- Enrichment service is supposed to fetch from Jikan to fill this gap
- Bug prevents this for high-quality anime

---

## 2. Background Job Tests Timeout for Complex Franchises

**Severity**: Low  
**Component**: Test Suite  
**Status**: Known Limitation

### Description

E2E tests that discover relations for complex franchises (e.g., Attack on Titan with 10+ seasons) timeout after 120 seconds.

### Impact

- Cannot run full e2e test suite in CI
- Complex franchise tests must be run manually

### Workaround

- Use simpler anime for automated tests (Death Note, Steins;Gate)
- Run complex franchise tests manually with longer timeout

---

## Future Improvements

1. Fix enrichment service to always fetch age_restriction
2. Optimize relations discovery to handle large franchises faster
3. Add partial enrichment for specific fields only
