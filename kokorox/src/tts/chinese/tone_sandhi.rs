//! Mandarin tone sandhi rules
//!
//! Adapted from misaki's tone_sandhi.py (originally from PaddleSpeech)

use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    /// Words that must have neutral tone on the last syllable
    static ref MUST_NEURAL_TONE_WORDS: HashSet<&'static str> = {
        let words = [
            "麻烦", "麻利", "鸳鸯", "高粱", "骨头", "骆驼", "马虎", "首饰", "馒头", "馄饨", "风筝",
            "难为", "队伍", "阔气", "闺女", "门道", "锄头", "铺盖", "铃铛", "铁匠", "钥匙", "里脊",
            "里头", "部分", "那么", "道士", "造化", "迷糊", "连累", "这么", "这个", "运气", "过去",
            "软和", "转悠", "踏实", "跳蚤", "跟头", "趔趄", "财主", "豆腐", "讲究", "记性", "记号",
            "认识", "规矩", "见识", "裁缝", "补丁", "衣裳", "衣服", "衙门", "街坊", "行李", "行当",
            "蛤蟆", "蘑菇", "薄荷", "葫芦", "葡萄", "萝卜", "荸荠", "苗条", "苗头", "苍蝇", "芝麻",
            "舒服", "舒坦", "舌头", "自在", "膏药", "脾气", "脑袋", "脊梁", "能耐", "胳膊", "胭脂",
            "胡萝", "胡琴", "胡同", "聪明", "耽误", "耽搁", "耷拉", "耳朵", "老爷", "老实", "老婆",
            "戏弄", "将军", "翻腾", "罗嗦", "罐头", "编辑", "结实", "红火", "累赘", "糨糊", "糊涂",
            "精神", "粮食", "簸箕", "篱笆", "算计", "算盘", "答应", "笤帚", "笑语", "笑话", "窟窿",
            "窝囊", "窗户", "稳当", "稀罕", "称呼", "秧歌", "秀气", "秀才", "福气", "祖宗", "砚台",
            "码头", "石榴", "石头", "石匠", "知识", "眼睛", "眯缝", "眨巴", "眉毛", "相声", "盘算",
            "白净", "痢疾", "痛快", "疟疾", "疙瘩", "疏忽", "畜生", "生意", "甘蔗", "琵琶", "琢磨",
            "琉璃", "玻璃", "玫瑰", "玄乎", "狐狸", "状元", "特务", "牲口", "牙碜", "牌楼", "爽快",
            "爱人", "热闹", "烧饼", "烟筒", "烂糊", "点心", "炊帚", "灯笼", "火候", "漂亮", "滑溜",
            "溜达", "温和", "清楚", "消息", "浪头", "活泼", "比方", "正经", "欺负", "模糊", "槟榔",
            "棺材", "棒槌", "棉花", "核桃", "栅栏", "柴火", "架势", "枕头", "枇杷", "机灵", "本事",
            "木头", "木匠", "朋友", "月饼", "月亮", "暖和", "明白", "时候", "新鲜", "故事", "收拾",
            "收成", "提防", "挖苦", "挑剔", "指甲", "指头", "拾掇", "拳头", "拨弄", "招牌", "招呼",
            "抬举", "护士", "折腾", "扫帚", "打量", "打算", "打扮", "打听", "打发", "扎实", "扁担",
            "戒指", "懒得", "意识", "意思", "悟性", "怪物", "思量", "怎么", "念头", "念叨", "别人",
            "快活", "忙活", "志气", "心思", "得罪", "张罗", "弟兄", "开通", "应酬", "庄稼", "干事",
            "帮手", "帐篷", "希罕", "师父", "师傅", "巴结", "巴掌", "差事", "工夫", "岁数", "屁股",
            "尾巴", "少爷", "小气", "小伙", "将就", "对头", "对付", "寡妇", "家伙", "客气", "实在",
            "官司", "学问", "字号", "嫁妆", "媳妇", "媒人", "婆家", "娘家", "委屈", "姑娘", "姐夫",
            "妯娌", "妥当", "妖精", "奴才", "女婿", "头发", "太阳", "大爷", "大方", "大意", "大夫",
            "多少", "多么", "外甥", "壮实", "地道", "地方", "在乎", "困难", "嘴巴", "嘱咐", "嘟囔",
            "嘀咕", "喜欢", "喇嘛", "喇叭", "商量", "唾沫", "哑巴", "哈欠", "哆嗦", "咳嗽", "和尚",
            "告诉", "告示", "含糊", "吓唬", "后头", "名字", "名堂", "合同", "吆喝", "叫唤", "口袋",
            "厚道", "厉害", "千斤", "包袱", "包涵", "匀称", "勤快", "动静", "动弹", "功夫", "力气",
            "前头", "刺猬", "刺激", "别扭", "利落", "利索", "利害", "分析", "出息", "凑合", "凉快",
            "冷战", "冤枉", "冒失", "养活", "关系", "先生", "兄弟", "便宜", "使唤", "佩服", "作坊",
            "体面", "位置", "似的", "伙计", "休息", "什么", "人家", "亲戚", "亲家", "交情", "云彩",
            "事情", "买卖", "主意", "丫头", "丧气", "两口", "东西", "东家", "世故", "不由", "下水",
            "下巴", "上头", "上司", "丈夫", "丈人", "一辈", "那个", "菩萨", "父亲", "母亲", "咕噜",
            "邋遢", "费用", "冤家", "甜头", "介绍", "荒唐", "大人", "泥鳅", "幸福", "熟悉", "计划",
            "扑腾", "蜡烛", "姥爷", "照顾", "喉咙", "吉他", "弄堂", "蚂蚱", "凤凰", "拖沓", "寒碜",
            "糟蹋", "倒腾", "报复", "逻辑", "盘缠", "喽啰", "牢骚", "咖喱", "扫把", "惦记"
        ];
        words.iter().cloned().collect()
    };

    /// Words that must NOT have neutral tone
    static ref MUST_NOT_NEURAL_TONE_WORDS: HashSet<&'static str> = {
        let words = [
            "男子", "女子", "分子", "原子", "量子", "莲子", "石子", "瓜子", "电子", "人人", "虎虎",
            "幺幺", "干嘛", "学子", "哈哈", "数数", "袅袅", "局地", "以下", "娃哈哈", "花花草草", "留得",
            "耕地", "想想", "熙熙", "攘攘", "卵子", "死死", "冉冉", "恳恳", "佼佼", "吵吵", "打打",
            "考考", "整整", "莘莘", "落地", "算子", "家家户户", "青青"
        ];
        words.iter().cloned().collect()
    };

    /// Punctuation characters
    static ref PUNC: HashSet<char> = {
        let s = "、：，；。？！\"\"''':,;.?!";
        s.chars().collect()
    };
    
    /// Sentence-final particles that get neutral tone
    static ref FINAL_PARTICLES: HashSet<char> = {
        "吧呢啊呐噻嘛吖嗨呐哦哒滴哩哟喽啰耶喔诶".chars().collect()
    };
    
    /// "De" particles that get neutral tone
    static ref DE_PARTICLES: HashSet<char> = {
        "的地得".chars().collect()
    };
}

/// Apply neural (neutral) tone sandhi
fn apply_neural_sandhi(word: &str, pos: &str, finals: &mut Vec<String>) {
    if MUST_NOT_NEURAL_TONE_WORDS.contains(word) {
        return;
    }
    
    let chars: Vec<char> = word.chars().collect();
    
    // Reduplication: e.g. 奶奶, 试试
    for (j, ch) in chars.iter().enumerate() {
        if j > 0 && *ch == chars[j - 1] && (pos.starts_with('n') || pos.starts_with('v') || pos.starts_with('a')) {
            if let Some(f) = finals.get_mut(j) {
                *f = set_tone(f, 5);
            }
        }
    }
    
    // Sentence-final particles
    if !chars.is_empty() {
        let last_char = chars[chars.len() - 1];
        if FINAL_PARTICLES.contains(&last_char) || DE_PARTICLES.contains(&last_char) {
            if let Some(f) = finals.last_mut() {
                *f = set_tone(f, 5);
            }
        }
    }
    
    // 了着过 as aspect markers
    if chars.len() == 1 && "了着过".contains(chars[0]) && ["ul", "uz", "ug"].contains(&pos) {
        if let Some(f) = finals.last_mut() {
            *f = set_tone(f, 5);
        }
    }
    
    // 们子 suffix
    if chars.len() > 1 && "们子".contains(chars[chars.len() - 1]) && (pos == "r" || pos == "n") {
        if let Some(f) = finals.last_mut() {
            *f = set_tone(f, 5);
        }
    }
    
    // 上下 as location suffix
    if chars.len() > 1 && "上下".contains(chars[chars.len() - 1]) && ["s", "l", "f"].contains(&pos) {
        if let Some(f) = finals.last_mut() {
            *f = set_tone(f, 5);
        }
    }
    
    // 来去 as directional complement
    if chars.len() > 1 && "来去".contains(chars[chars.len() - 1]) {
        if chars.len() >= 2 && "上下进出回过起开".contains(chars[chars.len() - 2]) {
            if let Some(f) = finals.last_mut() {
                *f = set_tone(f, 5);
            }
        }
    }
    
    // 个 as measure word
    if let Some(ge_idx) = word.find('个') {
        let ge_char_idx = word[..ge_idx].chars().count();
        if ge_char_idx > 0 {
            let prev_char = chars[ge_char_idx - 1];
            if prev_char.is_ascii_digit() || "几有两半多各整每做是".contains(prev_char) {
                if let Some(f) = finals.get_mut(ge_char_idx) {
                    *f = set_tone(f, 5);
                }
            }
        } else if word == "个" {
            if let Some(f) = finals.first_mut() {
                *f = set_tone(f, 5);
            }
        }
    }
    
    // Must neural tone words
    if MUST_NEURAL_TONE_WORDS.contains(word) {
        if let Some(f) = finals.last_mut() {
            *f = set_tone(f, 5);
        }
    }
    
    // Check last two chars
    if chars.len() >= 2 {
        let last_two: String = chars[chars.len()-2..].iter().collect();
        if MUST_NEURAL_TONE_WORDS.contains(last_two.as_str()) {
            if let Some(f) = finals.last_mut() {
                *f = set_tone(f, 5);
            }
        }
    }
}

/// Apply 不 (bu) sandhi
fn apply_bu_sandhi(word: &str, finals: &mut Vec<String>) {
    let chars: Vec<char> = word.chars().collect();
    
    // 看不懂 pattern - bu becomes neutral
    if chars.len() == 3 && chars[1] == '不' {
        if let Some(f) = finals.get_mut(1) {
            *f = set_tone(f, 5);
        }
    } else {
        // 不 before tone 4 becomes tone 2
        for (i, ch) in chars.iter().enumerate() {
            if *ch == '不' && i + 1 < chars.len() {
                if let Some(next_final) = finals.get(i + 1) {
                    if get_tone(next_final) == 4 {
                        if let Some(f) = finals.get_mut(i) {
                            *f = set_tone(f, 2);
                        }
                    }
                }
            }
        }
    }
}

/// Apply 一 (yi) sandhi  
fn apply_yi_sandhi(word: &str, finals: &mut Vec<String>) {
    let chars: Vec<char> = word.chars().collect();
    
    // Check if it's a number sequence (一零零)
    let all_numeric = chars.iter().all(|c| c.is_ascii_digit() || *c == '一');
    if word.contains('一') && all_numeric {
        return;  // Keep original tone in number sequences
    }
    
    // 看一看 pattern - yi becomes neutral
    if chars.len() == 3 && chars[1] == '一' && chars[0] == chars[2] {
        if let Some(f) = finals.get_mut(1) {
            *f = set_tone(f, 5);
        }
        return;
    }
    
    // 第一 - yi stays tone 1 (ordinal)
    if word.starts_with("第一") {
        return;
    }
    
    // Apply yi sandhi rules
    for (i, ch) in chars.iter().enumerate() {
        if *ch == '一' && i + 1 < chars.len() {
            if let Some(next_final) = finals.get(i + 1) {
                let next_tone = get_tone(next_final);
                if next_tone == 4 || next_tone == 5 {
                    // Before tone 4 or neutral -> yi becomes tone 2
                    if let Some(f) = finals.get_mut(i) {
                        *f = set_tone(f, 2);
                    }
                } else if !PUNC.contains(&chars[i + 1]) {
                    // Before other tones (not punctuation) -> yi becomes tone 4
                    if let Some(f) = finals.get_mut(i) {
                        *f = set_tone(f, 4);
                    }
                }
            }
        }
    }
}

/// Apply third tone sandhi (two consecutive third tones)
fn apply_three_sandhi(word: &str, finals: &mut Vec<String>) {
    let chars: Vec<char> = word.chars().collect();
    
    // Check if all tones are third tone
    let all_third = finals.iter().all(|f| get_tone(f) == 3);
    
    match chars.len() {
        2 if all_third => {
            // Two consecutive third tones: first becomes second
            if let Some(f) = finals.first_mut() {
                *f = set_tone(f, 2);
            }
        }
        3 => {
            if all_third {
                // Three third tones - depends on word structure
                // Default: first two become second
                if let Some(f) = finals.get_mut(0) {
                    *f = set_tone(f, 2);
                }
                if let Some(f) = finals.get_mut(1) {
                    *f = set_tone(f, 2);
                }
            } else {
                // Check for patterns like 所有人
                if finals.len() >= 2 && get_tone(&finals[0]) == 3 && get_tone(&finals[1]) == 3 {
                    if let Some(f) = finals.first_mut() {
                        *f = set_tone(f, 2);
                    }
                }
            }
        }
        4 if all_third => {
            // Four third tones: split into 2+2, each pair's first becomes second
            if let Some(f) = finals.get_mut(0) {
                *f = set_tone(f, 2);
            }
            if let Some(f) = finals.get_mut(2) {
                *f = set_tone(f, 2);
            }
        }
        _ => {}
    }
}

/// Get tone number from pinyin final (e.g., "ang1" -> 1)
fn get_tone(final_str: &str) -> u8 {
    final_str.chars()
        .last()
        .and_then(|c| c.to_digit(10))
        .map(|d| d as u8)
        .unwrap_or(5)
}

/// Set tone number in pinyin final
fn set_tone(final_str: &str, tone: u8) -> String {
    let base = final_str.trim_end_matches(|c: char| c.is_ascii_digit());
    format!("{}{}", base, tone)
}

/// Apply all tone sandhi rules
pub fn apply_tone_sandhi(word: &str, pos: &str, pinyins: &[String]) -> Vec<String> {
    let mut finals: Vec<String> = pinyins.to_vec();
    
    apply_bu_sandhi(word, &mut finals);
    apply_yi_sandhi(word, &mut finals);
    apply_neural_sandhi(word, pos, &mut finals);
    apply_three_sandhi(word, &mut finals);
    
    finals
}

/// Pre-merge words for better tone sandhi handling
/// This merges things like 不+verb, 一+noun, reduplication
pub fn pre_merge_for_modify(words_with_pos: &[(String, String)]) -> Vec<(String, String)> {
    let mut result: Vec<(String, String)> = Vec::new();
    let mut skip_next = false;
    
    for (i, (word, pos)) in words_with_pos.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        
        // Merge 不 with following word
        if word == "不" && i + 1 < words_with_pos.len() {
            let (next_word, next_pos) = &words_with_pos[i + 1];
            if next_pos != "x" && next_pos != "eng" {
                result.push((format!("{}{}", word, next_word), next_pos.clone()));
                skip_next = true;
                continue;
            }
        }
        
        // Merge 一 with following word
        if word == "一" && i + 1 < words_with_pos.len() {
            let (next_word, next_pos) = &words_with_pos[i + 1];
            if next_pos != "x" && next_pos != "eng" {
                result.push((format!("{}{}", word, next_word), next_pos.clone()));
                skip_next = true;
                continue;
            }
        }
        
        // Merge 儿 suffix
        if word == "儿" && !result.is_empty() {
            if let Some(last) = result.last_mut() {
                if last.1 != "x" && last.1 != "eng" {
                    last.0.push_str(word);
                    continue;
                }
            }
        }
        
        // Merge reduplication (e.g., 看 看 -> 看看)
        if !result.is_empty() {
            if let Some(last) = result.last_mut() {
                if &last.0 == word && pos != "x" && pos != "eng" {
                    last.0.push_str(word);
                    continue;
                }
            }
        }
        
        result.push((word.clone(), pos.clone()));
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bu_sandhi() {
        let mut finals = vec!["bu4".to_string(), "shi4".to_string()];
        apply_bu_sandhi("不是", &mut finals);
        assert_eq!(get_tone(&finals[0]), 2);  // bu before tone 4 -> tone 2
    }
    
    #[test]
    fn test_yi_sandhi() {
        let mut finals = vec!["yi1".to_string(), "ge4".to_string()];
        apply_yi_sandhi("一个", &mut finals);
        assert_eq!(get_tone(&finals[0]), 2);  // yi before tone 4 -> tone 2
    }
    
    #[test]
    fn test_three_sandhi() {
        let mut finals = vec!["ni3".to_string(), "hao3".to_string()];
        apply_three_sandhi("你好", &mut finals);
        assert_eq!(get_tone(&finals[0]), 2);  // First of two tone 3 -> tone 2
    }
    
    #[test]
    fn test_get_set_tone() {
        assert_eq!(get_tone("ang1"), 1);
        assert_eq!(get_tone("en5"), 5);
        assert_eq!(set_tone("ang1", 3), "ang3");
    }
}
