use std::cmp::min;

pub struct LineDetails<'a> {
    pub line_number: usize,
    pub line_offset: usize,

    pub line_start: usize,
    pub line_end: usize,

    pub line_text: &'a str
}

impl<'a> LineDetails<'a> {
    pub fn marker(&self) -> String {
        let mut result = "".to_string();

        for (i, c) in self.line_text.chars().enumerate() {
            if i > self.line_offset {
                break
            }

            if c.is_whitespace() {
                result.push(c)
            } else {
                // Assuming no unicode.
                result.push(' ')
            }
        }

        result.push('^');

        result
    }

    pub fn from_offset(source: &'a str, offset: usize) -> LineDetails<'a> {
        let offset = min(source.len(), offset);

        let source_offset = source.as_ptr() as usize;

        let mut count = 0;
        let mut line_number = 0;
        let mut line_offset = 0;
        let mut last_line = 0;
        let mut last_line_start = source;

        let mut input = source;

        while let Some(c) = input.chars().next() {
            let next = &input[c.len_utf8()..];

            // Weird iteration just for the pointer checking here.
            let start = input.as_ptr() as usize - source_offset;
            let end = next.as_ptr() as usize - source_offset;

            if end > offset && offset >= start {
                break
            }

            if c == '\n' {
                last_line = count + c.len_utf8();
                last_line_start = next;
                line_number += 1;
                line_offset = 0;
            } else {
                line_offset += 1;
            }

            count += c.len_utf8();
            input = next;
        }

        while let Some(c) = input.chars().next() {
            if c == '\n' {
                break
            }

            count += c.len_utf8();
            input = &input[c.len_utf8()..];
        }

        LineDetails {
            line_number,
            line_offset,
            line_start: last_line,
            line_end: count,
            line_text: &last_line_start[..count - last_line],
        }
    }
}
