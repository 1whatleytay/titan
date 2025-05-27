use std::rc::Rc;

pub trait TokenCursorInsights<'b, Token> {
    fn is_adjacent(&self, token: &'b Token) -> bool;
}

pub struct BaseTokenCursor<'b, Token, TokenInsights: TokenCursorInsights<'b, Token>> {
    index: usize,
    tokens: &'b [Token],
    insights: Rc<TokenInsights>
}

impl<'b, Token, TokenInsights: TokenCursorInsights<'b, Token>> BaseTokenCursor<'b, Token, TokenInsights> {
    pub fn new(tokens: &'b [Token], insights: TokenInsights) -> Self {
        BaseTokenCursor { index: 0, tokens, insights: Rc::new(insights) }
    }

    pub fn get_position(&self) -> usize {
        self.index
    }

    pub fn set_position(&mut self, index: usize) {
        self.index = index
    }

    pub fn peek(&self) -> Option<&'b Token> {
        self.tokens.get(self.index)
    }

    pub fn next(&mut self) -> Option<&'b Token> {
        let value = self.peek();

        self.index += 1;

        value
    }

    pub fn collect_until<F>(&mut self, mut f: F) -> Vec<&'b Token>
    where
        F: FnMut(&'b Token) -> bool,
    {
        let mut result = vec![];

        while let Some(value) = self.next() {
            let do_break = f(&value);

            result.push(value);

            if do_break {
                break;
            }
        }

        result
    }

    pub fn seek_until<F>(&mut self, mut f: F) -> Option<&'b Token>
    where
        F: FnMut(&'b Token) -> bool,
    {
        while let Some(value) = self.next() {
            if f(&value) {
                return Some(value);
            }
        }

        None
    }

    pub fn next_adjacent(&mut self) -> Option<&'b Token> {
        let insights = self.insights.clone();

        self.seek_until(move |token| insights.is_adjacent(token))
    }

    pub fn collect_without<F>(&mut self, mut f: F) -> Vec<&'b Token>
    where
        F: FnMut(&'b Token) -> bool,
    {
        let mut result = vec![];

        while let Some(value) = self.peek() {
            if !f(&value) {
                self.index += 1;

                result.push(value)
            } else {
                break;
            }
        }

        result
    }

    pub fn seek_without<F>(&mut self, mut f: F) -> Option<&'b Token>
    where
        F: FnMut(&'b Token) -> bool,
    {
        while let Some(value) = self.peek() {
            if !f(&value) {
                self.index += 1
            } else {
                break;
            }
        }

        self.peek()
    }

    pub fn peek_adjacent(&mut self) -> (usize, Option<&'b Token>) {
        let position = self.get_position();

        let insights = self.insights.clone();

        let result = self.seek_without(move |token| insights.is_adjacent(token));
        let end = self.get_position();

        self.set_position(position);

        (end, result)
    }

    pub fn consume_until(&mut self, index: usize) -> Vec<&'b Token> {
        if index < self.index {
            vec![]
        } else {
            let result = self.tokens[self.index..index].iter().collect();

            self.index = index;

            result
        }
    }
}
