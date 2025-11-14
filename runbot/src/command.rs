use regex::Regex;

pub struct CommandLopper<'a> {
    current_slice: &'a str,
    next_idx: usize,
    tokens: Vec<&'a str>,
}

impl<'a> CommandLopper<'a> {
    pub fn new(tokens: Vec<&'a str>) -> Self {
        Self {
            current_slice: "",
            next_idx: 0,
            tokens,
        }
    }
}

impl CommandLopper<'_> {
    fn check_next_slice(&mut self) {
        if self.current_slice.is_empty() {
            if self.next_idx < self.tokens.len() {
                self.current_slice = self.tokens[self.next_idx];
                self.next_idx += 1;
            }
        }
    }

    pub fn next_enum(&mut self, enums: &[&str]) -> Option<String> {
        self.check_next_slice();
        if self.current_slice.is_empty() {
            return None;
        }
        for enum_str in enums {
            if self.current_slice.starts_with(enum_str) {
                self.current_slice = self.current_slice.strip_prefix(enum_str).unwrap_or("");
                return Some(enum_str.to_string());
            }
        }
        None
    }

    pub fn cut_plain_text(&mut self, text: &str) -> bool {
        self.check_next_slice();
        if self.current_slice.is_empty() {
            return false;
        }
        if self.current_slice == text {
            self.current_slice = "";
            return true;
        }
        false
    }

    pub fn next_number(&mut self) -> Option<String> {
        self.check_next_slice();
        if self.current_slice.is_empty() {
            return None;
        }
        let re = Regex::new(r"^\d+([.]\d+)?").unwrap();
        let caps = re.captures(self.current_slice);
        if caps.is_none() {
            return None;
        }
        let caps = caps.unwrap();
        let num = caps.get(0).unwrap().as_str().to_string();
        self.current_slice = self.current_slice.strip_prefix(num.as_str()).unwrap_or("");
        Some(num)
    }

    pub fn cut_text_to_space(&mut self) -> Option<String> {
        self.check_next_slice();
        if self.current_slice.is_empty() {
            return None;
        }
        let result = self.current_slice.to_string();
        self.current_slice = "";
        Some(result)
    }

    pub fn cut_text_to_end(&mut self) -> Option<String> {
        self.check_next_slice();
        if self.current_slice.is_empty() {
            return None;
        }
        let mut buffer = self.current_slice.to_string();
        self.current_slice = "";
        loop {
            self.check_next_slice();
            if self.current_slice.is_empty() {
                break;
            }
            buffer += " ";
            buffer += self.current_slice;
            self.current_slice = "";
        }
        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cut_plain_text_exact_match() {
        // 测试完全匹配：runbot 不应该匹配 run
        let mut lopper = CommandLopper::new(vec!["runbot"]);
        assert!(!lopper.cut_plain_text("run"), "runbot should not match run");
        
        // 测试完全匹配：runbot 应该匹配 runbot
        let mut lopper = CommandLopper::new(vec!["runbot"]);
        assert!(lopper.cut_plain_text("runbot"), "runbot should match runbot");
    }

    #[test]
    fn test_cut_plain_text_multi_token() {
        // 测试多 token：run bot 不应该匹配 run（因为还有剩余 token）
        let mut lopper = CommandLopper::new(vec!["run", "bot"]);
        assert!(lopper.cut_plain_text("run"), "first token should match");
        // 检查是否还有剩余 token
        assert!(!lopper.current_slice.is_empty() || lopper.next_idx < lopper.tokens.len(), 
                "should have remaining tokens");
        
        // 测试多 token：run bot 应该完全匹配 run bot
        let mut lopper = CommandLopper::new(vec!["run", "bot"]);
        assert!(lopper.cut_plain_text("run"));
        lopper.check_next_slice();
        assert!(lopper.cut_plain_text("bot"));
    }

    #[test]
    fn test_cut_plain_text_single_token() {
        // 测试单 token：run 应该匹配 run
        let mut lopper = CommandLopper::new(vec!["run"]);
        assert!(lopper.cut_plain_text("run"), "run should match run");
        assert!(lopper.current_slice.is_empty(), "should consume the token");
    }

    #[test]
    fn test_cut_plain_text_no_match() {
        // 测试不匹配的情况
        let mut lopper = CommandLopper::new(vec!["hello"]);
        assert!(!lopper.cut_plain_text("world"), "hello should not match world");
    }

    #[test]
    fn test_cut_plain_text_empty() {
        // 测试空输入
        let mut lopper = CommandLopper::new(vec![]);
        assert!(!lopper.cut_plain_text("run"), "empty should not match");
    }

    #[test]
    fn test_next_enum_prefix_match() {
        // 测试前缀匹配（用于前缀如 - / ~）
        let mut lopper = CommandLopper::new(vec!["-runbot"]);
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), Some("-".to_string()));
        assert_eq!(lopper.current_slice, "runbot");
        
        // 测试多个前缀选项
        let mut lopper = CommandLopper::new(vec!["/test"]);
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), Some("/".to_string()));
    }

    #[test]
    fn test_next_enum_no_match() {
        // 测试枚举不匹配
        let mut lopper = CommandLopper::new(vec!["runbot"]);
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), None);
    }

    #[test]
    fn test_next_number() {
        // 测试数字匹配
        let mut lopper = CommandLopper::new(vec!["123"]);
        assert_eq!(lopper.next_number(), Some("123".to_string()));
        
        // 测试小数
        let mut lopper = CommandLopper::new(vec!["123.45"]);
        assert_eq!(lopper.next_number(), Some("123.45".to_string()));
        
        // 测试非数字
        let mut lopper = CommandLopper::new(vec!["abc"]);
        assert_eq!(lopper.next_number(), None);
    }

    #[test]
    fn test_cut_text_to_space() {
        // 测试截取到空格
        let mut lopper = CommandLopper::new(vec!["hello", "world"]);
        assert_eq!(lopper.cut_text_to_space(), Some("hello".to_string()));
        assert_eq!(lopper.cut_text_to_space(), Some("world".to_string()));
    }

    #[test]
    fn test_cut_text_to_end() {
        // 测试截取到结尾
        let mut lopper = CommandLopper::new(vec!["hello", "world", "test"]);
        assert_eq!(lopper.cut_text_to_end(), Some("hello world test".to_string()));
    }

    #[test]
    fn test_complex_scenario_run_vs_runbot() {
        // 完整场景测试：runbot 不应该匹配 run
        let mut lopper = CommandLopper::new(vec!["-", "runbot"]);
        
        // 匹配前缀
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), Some("-".to_string()));
        
        // 尝试匹配 run，应该失败
        assert!(!lopper.cut_plain_text("run"), "runbot should not match run");
        
        // 重置并匹配 runbot，应该成功
        let mut lopper = CommandLopper::new(vec!["-", "runbot"]);
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), Some("-".to_string()));
        assert!(lopper.cut_plain_text("runbot"), "runbot should match runbot");
    }

    #[test]
    fn test_complex_scenario_run_bot() {
        // 完整场景测试：run bot 不应该匹配 run（单独）
        let mut lopper = CommandLopper::new(vec!["-", "run", "bot"]);
        
        // 匹配前缀
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), Some("-".to_string()));
        
        // 匹配 run
        assert!(lopper.cut_plain_text("run"));
        
        // 检查是否还有剩余（bot）
        lopper.check_next_slice();
        assert!(!lopper.current_slice.is_empty(), "should have remaining token 'bot'");
        
        // 完整匹配 run bot
        let mut lopper = CommandLopper::new(vec!["-", "run", "bot"]);
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), Some("-".to_string()));
        assert!(lopper.cut_plain_text("run"));
        lopper.check_next_slice();
        assert!(lopper.cut_plain_text("bot"));
    }

    #[test]
    fn test_whitespace_handling() {
        // 测试 split_ascii_whitespace 的行为，模拟实际使用场景
        // 输入 "run bot  " 应该被处理成 ["run", "bot"]
        
        // 模拟 trim() 和 split_ascii_whitespace() 的处理
        let input = "run bot  ";
        let tokens: Vec<&str> = input.trim().split_ascii_whitespace().collect();
        assert_eq!(tokens, vec!["run", "bot"], "trailing spaces should be ignored");
        
        // 测试匹配：run bot 应该匹配 run bot
        let mut lopper = CommandLopper::new(tokens);
        assert!(lopper.cut_plain_text("run"), "should match 'run'");
        lopper.check_next_slice();
        assert!(lopper.cut_plain_text("bot"), "should match 'bot'");
    }

    #[test]
    fn test_whitespace_variations() {
        // 测试各种空格变体
        
        // 1. 末尾有多个空格
        let input1 = "run bot   ";
        let tokens1: Vec<&str> = input1.trim().split_ascii_whitespace().collect();
        assert_eq!(tokens1, vec!["run", "bot"]);
        let mut lopper1 = CommandLopper::new(tokens1);
        assert!(lopper1.cut_plain_text("run"));
        lopper1.check_next_slice();
        assert!(lopper1.cut_plain_text("bot"));
        
        // 2. 开头有空格
        let input2 = "  run bot";
        let tokens2: Vec<&str> = input2.trim().split_ascii_whitespace().collect();
        assert_eq!(tokens2, vec!["run", "bot"]);
        let mut lopper2 = CommandLopper::new(tokens2);
        assert!(lopper2.cut_plain_text("run"));
        lopper2.check_next_slice();
        assert!(lopper2.cut_plain_text("bot"));
        
        // 3. 中间有多个空格
        let input3 = "run    bot";
        let tokens3: Vec<&str> = input3.trim().split_ascii_whitespace().collect();
        assert_eq!(tokens3, vec!["run", "bot"]);
        let mut lopper3 = CommandLopper::new(tokens3);
        assert!(lopper3.cut_plain_text("run"));
        lopper3.check_next_slice();
        assert!(lopper3.cut_plain_text("bot"));
        
        // 4. 前后都有空格
        let input4 = "  run bot  ";
        let tokens4: Vec<&str> = input4.trim().split_ascii_whitespace().collect();
        assert_eq!(tokens4, vec!["run", "bot"]);
        let mut lopper4 = CommandLopper::new(tokens4);
        assert!(lopper4.cut_plain_text("run"));
        lopper4.check_next_slice();
        assert!(lopper4.cut_plain_text("bot"));
    }

    #[test]
    fn test_whitespace_with_prefix() {
        // 测试带前缀的情况：-run bot  
        let input = "-run bot  ";
        let tokens: Vec<&str> = input.trim().split_ascii_whitespace().collect();
        assert_eq!(tokens, vec!["-run", "bot"]);
        
        // 注意：这里 "-run" 是一个完整的 token，需要先处理前缀
        // 实际使用中，前缀和命令是分开的，所以应该是 ["-", "run", "bot"]
        // 但为了测试 split_ascii_whitespace 的行为，我们测试这种情况
        let input2 = "- run bot  ";
        let tokens2: Vec<&str> = input2.trim().split_ascii_whitespace().collect();
        assert_eq!(tokens2, vec!["-", "run", "bot"]);
        
        let mut lopper = CommandLopper::new(tokens2);
        assert_eq!(lopper.next_enum(&["-", "/", "~"]), Some("-".to_string()));
        assert!(lopper.cut_plain_text("run"));
        lopper.check_next_slice();
        assert!(lopper.cut_plain_text("bot"));
    }
}
