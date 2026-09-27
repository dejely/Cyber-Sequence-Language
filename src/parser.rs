use crate::token::{ Literal, Token, TokenType };

pub struct Parser {
    tokens: Vec<Tokens>,
    current usize,
}

impl Parser {

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0, 
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String>{
        self.expression()
    }

    pub fn expression(&mut self) -> Result<Expr, String>{
        self.equality()
    }

    pub fn equality(&mut self) -> Result<Expr, String>{
        let mut expr = self.comparison()?;

        while self.match_token(&[
            TokenType::Equal,
            TokenType::BangEqual,
        ]){
            let operator = self.previous().clone();
            let right = self.comparison()?;

                expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub fn comparison(&mut self) -> Result<Expr, String>{
        let mut expr = self.comparison()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]){
            let operator = self.previous().clone();
            let right = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr);
    }

}
