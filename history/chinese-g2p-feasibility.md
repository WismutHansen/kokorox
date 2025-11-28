# Chinese G2P (ZHG2P) Rust Implementation Feasibility Analysis

## Overview

This document analyzes the feasibility of porting misaki's Chinese G2P (Grapheme-to-Phoneme) system to pure Rust for kokorox. This would enable native Chinese TTS support without Python dependencies.

## Current Python Implementation (misaki)

The misaki ZHG2P consists of several components:

### 1. Core Dependencies

| Python Library | Purpose | Rust Alternative | Status |
|---------------|---------|------------------|--------|
| jieba | Chinese word segmentation | jieba-rs (0.8.1) | Available |
| pypinyin | Character to pinyin conversion | pinyin (0.10.0) | Available |
| cn2an | Chinese number conversion | chinese-number (0.7.7) | Available |
| pypinyin_dict | Extended pinyin dictionary | Need to bundle data | Manual port |

### 2. Component Analysis

#### A. Word Segmentation (jieba)

**Python**: `jieba.lcut()`, `jieba.posseg.lcut()` (with POS tagging)

**Rust**: `jieba-rs` provides:
- `jieba.cut()` - basic segmentation
- `jieba.tag()` - POS tagging (via `jieba-rs` with `tfidf` feature)

**Assessment**: Fully available. jieba-rs is mature with 370k+ downloads.

```rust
use jieba_rs::Jieba;
let jieba = Jieba::new();
let words = jieba.cut("hello world", false);
let tagged = jieba.tag("hello world", false); // (word, tag) pairs
```

#### B. Pinyin Conversion (pypinyin)

**Python**: `lazy_pinyin(word, style=Style.TONE3, neutral_tone_with_five=True)`

**Rust**: `pinyin` crate provides:
- Multiple output styles (with/without tones)
- Heteronym handling

**Assessment**: Available but may need verification of tone3 style support.

```rust
use pinyin::{ToPinyin, ToPinyinMulti};
let pinyin: Vec<&str> = "hello".to_pinyin().map(|p| p.plain()).collect();
```

**Gap**: May need to verify `neutral_tone_with_five=True` equivalent.

#### C. Number Conversion (cn2an)

**Python**: `cn2an.transform(text, 'an2cn')` - Arabic to Chinese numbers

**Rust**: `chinese-number` crate provides bidirectional conversion.

**Assessment**: Fully available.

```rust
use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
let chinese = 123.to_chinese(ChineseVariant::Simple, ChineseCase::Lower, ChineseCountMethod::Low);
```

#### D. Tone Sandhi Rules

**Python**: `tone_sandhi.py` - ~400 lines of tone modification rules

**Key Rules**:
1. Third tone sandhi (two consecutive 3rd tones -> first becomes 2nd)
2. Bu (not) sandhi - changes based on following tone
3. Yi (one) sandhi - changes based on context
4. Neural tone handling for particles/suffixes

**Assessment**: Straightforward port. Rules are algorithmic, not data-dependent.

**Complexity**: Medium. Need to port:
- `must_neural_tone_words` dictionary (~200 words)
- `must_not_neural_tone_words` dictionary (~40 words)
- Sandhi application logic

#### E. Pinyin-to-IPA Transcription

**Python**: `transcription.py` - ~250 lines of pinyin to IPA mapping

**Assessment**: Direct port. Static mappings:
- `INITIAL_MAPPING` - 21 initials to IPA
- `FINAL_MAPPING` - ~40 finals to IPA
- `TONE_MAPPING` - 5 tones to IPA markers

#### F. ZH Frontend (v1.1 features)

**Python**: `zh_frontend.py` - ~200 lines

**Features**:
- Erhua (R-coloring) handling
- Custom phrase dictionary
- Bopomofo (Zhuyin) character mapping

**Assessment**: Medium complexity. Need to port:
- `ZH_MAP` dictionary (pinyin to Bopomofo)
- Erhua processing logic
- Phrase-level pinyin overrides

## Estimated Effort

| Component | Lines of Rust | Difficulty | Dependencies |
|-----------|--------------|------------|--------------|
| jieba integration | ~50 | Easy | jieba-rs |
| pinyin integration | ~100 | Medium | pinyin |
| number conversion | ~30 | Easy | chinese-number |
| tone sandhi | ~300 | Medium | None (pure Rust) |
| pinyin-to-IPA | ~200 | Easy | None (pure Rust) |
| ZH frontend | ~250 | Medium | None (pure Rust) |
| **Total** | **~930** | **Medium** | 3 crates |

## Data Files Required

1. **Phrase dictionary** (~10KB) - Custom pinyin for multi-character words
2. **Tone word lists** (~5KB) - Neural/not-neural tone words
3. **Erhua word lists** (~2KB) - must_erhua/not_erhua sets

Total embedded data: ~17KB

## Architecture Proposal

```
kokorox/src/tts/
  chinese/
    mod.rs          # Public API
    segmentation.rs # jieba-rs integration
    pinyin.rs       # pinyin crate integration
    tone_sandhi.rs  # Tone modification rules
    transcription.rs # Pinyin to IPA/Bopomofo
    frontend.rs     # High-level G2P pipeline
    data/
      phrases.json  # Custom phrase dictionary
      tone_words.json # Tone word lists
```

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| jieba-rs POS accuracy differs from Python | Medium | Test with reference sentences |
| pinyin tone format differences | High | Verify style options match |
| Missing heteronyms | Medium | Add fallback dictionary |
| Performance regression | Low | Rust should be faster |

## Recommendation

**Feasibility: HIGH**

The Chinese G2P can be implemented in pure Rust with existing crates. The main work is:

1. **Phase 1** (2-3 days): Basic pipeline
   - jieba-rs word segmentation
   - pinyin conversion
   - Simple IPA output

2. **Phase 2** (2-3 days): Tone sandhi
   - Port tone modification rules
   - Add neural tone handling

3. **Phase 3** (1-2 days): Production features
   - Erhua handling
   - Custom phrase dictionary
   - Number conversion

Total estimated time: **5-8 days**

## Alternative: FFI to Python

If time is constrained, could use PyO3 to call misaki directly:

**Pros**: Exact compatibility, faster to implement
**Cons**: Python dependency, deployment complexity

## Next Steps

1. Create proof-of-concept with jieba-rs + pinyin
2. Verify pinyin output format matches misaki
3. Port tone_sandhi.py rules
4. Integrate with existing phonemizer infrastructure
