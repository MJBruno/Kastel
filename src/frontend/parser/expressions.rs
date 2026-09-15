use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::lexer::token::TokenKind;
 

use super::Parser;

#[allow(dead_code)]
impl Parser {
    // ============================================================
    // EXPRESSIONS
    // ============================================================

    pub(super) fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.arrow_function()
    }

    fn arrow_function(&mut self) -> Result<Expression, ParserError> {
        if !self.is_arrow_function_start() {
            return self.ternary();
        }

        let params = self.parse_arrow_parameters()?;

        self.consume(
            TokenKind::FatArrow,
            "'=>' attendu après les paramètres de la fonction fléchée",
        )?;

        if self.match_token(TokenKind::LeftBrace) {
            let body = self.parse_block_statement()?;

            return Ok(Expression::Function { params, body });
        }

        let expression = self.parse_expression()?;

        Ok(Expression::Function {
            params,
            body: vec![Statement::Return {
                value: Some(expression),
            }],
        })
    }

    fn is_arrow_function_start(&self) -> bool {
        if self.check(TokenKind::Identifier) && self.check_next(TokenKind::FatArrow) {
            return true;
        }

        if !self.check(TokenKind::LeftParen) {
            return false;
        }

        let mut index = self.current + 1;

        if index < self.tokens.len() && self.tokens[index].kind == TokenKind::RightParen {
            return index + 1 < self.tokens.len()
                && self.tokens[index + 1].kind == TokenKind::FatArrow;
        }

        loop {
            if index >= self.tokens.len() || self.tokens[index].kind != TokenKind::Identifier {
                return false;
            }

            index += 1;

            if index >= self.tokens.len() {
                return false;
            }

            match self.tokens[index].kind {
                TokenKind::Comma => {
                    index += 1;

                    if index < self.tokens.len() && self.tokens[index].kind == TokenKind::RightParen
                    {
                        index += 1;

                        return index < self.tokens.len()
                            && self.tokens[index].kind == TokenKind::FatArrow;
                    }
                }

                TokenKind::RightParen => {
                    index += 1;

                    return index < self.tokens.len()
                        && self.tokens[index].kind == TokenKind::FatArrow;
                }

                _ => return false,
            }
        }
    }

    fn parse_arrow_parameters(&mut self) -> Result<Vec<String>, ParserError> {
        if self.check(TokenKind::Identifier) && self.check_next(TokenKind::FatArrow) {
            let parameter = self.advance().clone();

            return Ok(vec![parameter.lexeme]);
        }

        self.consume(
            TokenKind::LeftParen,
            "'(' attendu pour les paramètres de la fonction fléchée",
        )?;

        let mut params = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let parameter = self.consume(
                    TokenKind::Identifier,
                    "Nom de paramètre attendu dans la fonction fléchée",
                )?;

                params.push(parameter.lexeme);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }

                if self.check(TokenKind::RightParen) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightParen,
            "')' attendu après les paramètres de la fonction fléchée",
        )?;

        Ok(params)
    }

    fn ternary(&mut self) -> Result<Expression, ParserError> {
        let condition = self.logical_or()?;

        if self.match_token(TokenKind::Question) {
            let then_expr = self.parse_expression()?;

            self.consume(TokenKind::Colon, "':' attendu dans l'expression ternaire")?;

            let else_expr = self.ternary()?;

            return Ok(Expression::Ternary {
                condition: Box::new(condition),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            });
        }

        Ok(condition)
    }

    fn logical_or(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.logical_and()?;

        while self.match_token(TokenKind::Or) {
            let right = self.logical_and()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.bitwise_or()?;

        while self.match_token(TokenKind::And) {
            let right = self.bitwise_or()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn bitwise_or(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.bitwise_xor()?;

        while self.match_token(TokenKind::Pipe) {
            let right = self.bitwise_xor()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::BitOr,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn bitwise_xor(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.bitwise_and()?;

        while self.match_token(TokenKind::Caret) {
            let right = self.bitwise_and()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::BitXor,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn bitwise_and(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.comparison()?;

        while self.match_token(TokenKind::Ampersand) {
            let right = self.comparison()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOp::BitAnd,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.shift()?;

        while self.match_any(&[
            TokenKind::EqualEqual,
            TokenKind::NotEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Is,
        ]) {
            let operator = match self.previous().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                TokenKind::Is => BinaryOp::Is,

                _ => unreachable!(),
            };

            let right = self.shift()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn shift(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.term()?;

        while self.match_any(&[TokenKind::LeftShift, TokenKind::RightShift]) {
            let operator = match self.previous().kind {
                TokenKind::LeftShift => BinaryOp::ShiftLeft,
                TokenKind::RightShift => BinaryOp::ShiftRight,

                _ => unreachable!(),
            };

            let right = self.term()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.factor()?;

        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = match self.previous().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,

                _ => unreachable!(),
            };

            let right = self.factor()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.unary()?;

        while self.match_any(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let operator = match self.previous().kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,

                _ => unreachable!(),
            };

            let right = self.unary()?;

            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, ParserError> {
        if self.match_token(TokenKind::Minus) {
            let operand = self.unary()?;

            return Ok(Expression::Unary {
                operator: UnaryOp::Negate,
                right: Box::new(operand),
            });
        }

        if self.match_token(TokenKind::Not) {
            let operand = self.unary()?;

            return Ok(Expression::Unary {
                operator: UnaryOp::Not,
                right: Box::new(operand),
            });
        }

        if self.match_token(TokenKind::Tilde) {
            let operand = self.unary()?;

            return Ok(Expression::Unary {
                operator: UnaryOp::BitNot,
                right: Box::new(operand),
            });
        }

        self.call()
    }

    // ============================================================
    // CALL / INDEX
    // ============================================================

    fn call(&mut self) -> Result<Expression, ParserError> {
        let mut expression = self.primary()?;

        loop {
            if self.match_token(TokenKind::LeftParen) {
                expression = self.parse_call(expression)?;
                continue;
            }

            if self.match_token(TokenKind::LeftBracket) {
                let index = self.parse_expression()?;

                self.consume(TokenKind::RightBracket, "']' attendu après l'index")?;

                expression = Expression::Index {
                    object: Box::new(expression),
                    index: Box::new(index),
                };

                continue;
            }

            if self.match_token(TokenKind::Dot) {
                let name =
                    self.consume(TokenKind::Identifier, "Nom de membre attendu après '.'")?;

                expression = Expression::Member {
                    object: Box::new(expression),
                    name: name.lexeme,
                };

                continue;
            }

            break;
        }

        Ok(expression)
    }

    fn parse_call(&mut self, callee: Expression) -> Result<Expression, ParserError> {
        let mut arguments = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "')' attendu après les arguments")?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    // ============================================================
    // PRIMARY
    // ============================================================

    fn primary(&mut self) -> Result<Expression, ParserError> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Number => self.parse_number(token),

            TokenKind::String => self.parse_string(token),

            TokenKind::Identifier => Ok(Expression::Variable(token.lexeme)),

            TokenKind::True => Ok(Expression::Literal(Literal::Bool(true))),

            TokenKind::False => Ok(Expression::Literal(Literal::Bool(false))),

            TokenKind::None => Ok(Expression::Literal(Literal::None)),

            TokenKind::This => Ok(Expression::This),

            TokenKind::New => self.parse_new_expression(),

            TokenKind::Base => Ok(Expression::Base),

            TokenKind::Function => self.parse_function_expression(),

            TokenKind::LeftParen => {
                // Tuple vide : `()`.
                //
                // (is_arrow_function_start() a déjà intercepté plus haut
                // le cas `() => ...`, donc si on arrive ici avec des
                // parenthèses vides, ce ne peut être qu'un tuple vide.)
                if self.check(TokenKind::RightParen) {
                    self.advance();

                    return Ok(Expression::Tuple(Vec::new()));
                }

                let first = self.parse_expression()?;

                // Une virgule après la première expression signale un
                // tuple : `(1, 2, 3)` ou `(1,)` (tuple à un élément).
                // Sans virgule, ce sont de simples parenthèses de
                // groupement : `(1 + 2)` reste l'expression `1 + 2`.
                if self.match_token(TokenKind::Comma) {
                    let mut elements = vec![first];

                    if !self.check(TokenKind::RightParen) {
                        loop {
                            elements.push(self.parse_expression()?);

                            if !self.match_token(TokenKind::Comma) {
                                break;
                            }

                            if self.check(TokenKind::RightParen) {
                                break;
                            }
                        }
                    }

                    self.consume(TokenKind::RightParen, "')' attendu après le tuple")?;

                    return Ok(Expression::Tuple(elements));
                }

                self.consume(TokenKind::RightParen, "')' attendu après l'expression")?;

                Ok(first)
            }

            TokenKind::LeftBracket => {
                let mut elements = Vec::new();

                if !self.check(TokenKind::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);

                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }

                        if self.check(TokenKind::RightBracket) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RightBracket, "']' attendu après le tableau")?;

                Ok(Expression::Array(elements))
            }

            TokenKind::LeftBrace => {
                let mut fields: Vec<(String, Expression)> = Vec::new();

                if !self.check(TokenKind::RightBrace) {
                    loop {
                        let key = if self.check(TokenKind::String) {
                            self.advance().lexeme.clone()
                        } else {
                            self.consume(TokenKind::Identifier, "nom de champ attendu")?
                                .lexeme
                        };

                        self.consume(TokenKind::Colon, "':' attendu après le nom du champ")?;

                        let value = self.parse_expression()?;

                        fields.push((key, value));

                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }

                        if self.check(TokenKind::RightBrace) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RightBrace, "'}' attendu après l'objet")?;

                Ok(Expression::Object(fields))
            }

            _ => Err(ParserError {
                message: "Expression invalide".to_string(),
                line: token.line,
                column: token.column,
            }),
        }
    }
    fn parse_new_expression(&mut self) -> Result<Expression, ParserError> {
        let class_name =
            self.consume(TokenKind::Identifier, "Nom de classe attendu après 'new'")?;

        self.consume(TokenKind::LeftParen, "'(' attendu après le nom de classe")?;

        let mut arguments = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightParen,
            "')' attendu après les arguments de construction",
        )?;

        Ok(Expression::New {
            class_name: class_name.lexeme,
            arguments,
        })
    }
    fn parse_function_expression(&mut self) -> Result<Expression, ParserError> {
        self.consume(
            TokenKind::LeftParen,
            "'(' attendu après 'function' dans une fonction anonyme",
        )?;

        let mut params = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let param = self.consume(TokenKind::Identifier, "Nom de paramètre attendu")?;

                params.push(param.lexeme);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "')' attendu après les paramètres")?;

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu avant le corps de la fonction anonyme",
        )?;

        let body = self.parse_block_statement()?;

        Ok(Expression::Function { params, body })
    }
}
