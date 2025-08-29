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
        if self.current_slice.starts_with(text) {
            self.current_slice = self.current_slice.strip_prefix(text).unwrap_or("");
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
