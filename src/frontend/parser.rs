use crate::error::parse_error::ParserError;
use crate::frontend::ast::*;
use crate::frontend::token::*;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParserError>,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, Vec<ParserError>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.statement() {
                Ok(stmts) => {
                    statements.extend(stmts);
                }

                Err(error) => {
                    self.errors.push(error);
                    self.advance();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.errors.clone())
        }
    }

    // ============================================================
    // TOKEN HELPERS
    // ============================================================

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind.clone()) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }

        self.tokens[self.current + 1].kind == kind
    }

    fn consume(
        &mut self,
        kind: TokenKind,
        message: &str,
    ) -> Result<Token, ParserError> {
        if self.check(kind.clone()) {
            return Ok(self.advance().clone());
        }

        Err(ParserError {
            message: message.to_string(),
            line: self.peek().line,
            column: self.peek().column,
        })
    }

    fn parse_number(&self, token: Token) -> Result<Expression, ParserError> {
        let is_float =
            token.lexeme.contains('.')
                || token.lexeme.contains(['e', 'E']);

        if is_float {
            let value = token.lexeme.parse::<f64>().map_err(|_| ParserError {
                message: format!(
                    "Nombre flottant invalide '{}'",
                    token.lexeme
                ),
                line: token.line,
                column: token.column,
            })?;

            Ok(Expression::Literal(Literal::Float(value)))
        } else {
            let value = token.lexeme.parse::<i64>().map_err(|_| ParserError {
                message: format!(
                    "Nombre entier invalide '{}'",
                    token.lexeme
                ),
                line: token.line,
                column: token.column,
            })?;

            Ok(Expression::Literal(Literal::Integer(value)))
        }
    }

    fn parse_string(
        &self,
        token: Token,
    ) -> Result<Expression, ParserError> {
        let value = token.lexeme.trim_matches('"').to_string();

        Ok(Expression::Literal(Literal::String(value)))
    }

    // ============================================================
    // EXPRESSIONS
    // ============================================================

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
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
        if self.check(TokenKind::Identifier)
            && self.check_next(TokenKind::FatArrow)
        {
            return true;
        }

        if !self.check(TokenKind::LeftParen) {
            return false;
        }

        let mut index = self.current + 1;

        if index < self.tokens.len()
            && self.tokens[index].kind == TokenKind::RightParen
        {
            return index + 1 < self.tokens.len()
                && self.tokens[index + 1].kind
                    == TokenKind::FatArrow;
        }

        loop {
            if index >= self.tokens.len()
                || self.tokens[index].kind != TokenKind::Identifier
            {
                return false;
            }

            index += 1;

            if index >= self.tokens.len() {
                return false;
            }

            match self.tokens[index].kind {
                TokenKind::Comma => {
                    index += 1;

                    if index < self.tokens.len()
                        && self.tokens[index].kind
                            == TokenKind::RightParen
                    {
                        index += 1;

                        return index < self.tokens.len()
                            && self.tokens[index].kind
                                == TokenKind::FatArrow;
                    }
                }

                TokenKind::RightParen => {
                    index += 1;

                    return index < self.tokens.len()
                        && self.tokens[index].kind
                            == TokenKind::FatArrow;
                }

                _ => return false,
            }
        }
    }

    fn parse_arrow_parameters(
        &mut self,
    ) -> Result<Vec<String>, ParserError> {
        if self.check(TokenKind::Identifier)
            && self.check_next(TokenKind::FatArrow)
        {
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

            self.consume(
                TokenKind::Colon,
                "':' attendu dans l'expression ternaire",
            )?;

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
        ]) {
            let operator = match self.previous().kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::NotEqual => BinaryOp::NotEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,

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

        while self.match_any(&[
            TokenKind::LeftShift,
            TokenKind::RightShift,
        ]) {
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

        while self.match_any(&[
            TokenKind::Plus,
            TokenKind::Minus,
        ]) {
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

        while self.match_any(&[
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
        ]) {
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

                self.consume(
                    TokenKind::RightBracket,
                    "']' attendu après l'index",
                )?;

                expression = Expression::Index {
                    object: Box::new(expression),
                    index: Box::new(index),
                };

                continue;
            }

            if self.match_token(TokenKind::Dot) {
                let name = self.consume(
                    TokenKind::Identifier,
                    "Nom de membre attendu après '.'",
                )?;

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

    fn parse_call(
        &mut self,
        callee: Expression,
    ) -> Result<Expression, ParserError> {
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
            "')' attendu après les arguments",
        )?;

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

            TokenKind::Identifier => {
                Ok(Expression::Variable(token.lexeme))
            }

            TokenKind::True => {
                Ok(Expression::Literal(Literal::Bool(true)))
            }

            TokenKind::False => {
                Ok(Expression::Literal(Literal::Bool(false)))
            }

            TokenKind::Nil => {
                Ok(Expression::Literal(Literal::Nil))
            }

            TokenKind::Function => self.parse_function_expression(),

            TokenKind::LeftParen => {
                let expression = self.parse_expression()?;

                self.consume(
                    TokenKind::RightParen,
                    "')' attendu après l'expression",
                )?;

                Ok(expression)
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

                self.consume(
                    TokenKind::RightBracket,
                    "']' attendu après le tableau",
                )?;

                Ok(Expression::Array(elements))
            }

            TokenKind::LeftBrace => {
                let mut fields: Vec<(String, Expression)> =
                    Vec::new();

                if !self.check(TokenKind::RightBrace) {
                    loop {
                        let key = if self.check(TokenKind::String) {
                            self.advance().lexeme.clone()
                        } else {
                            self.consume(
                                TokenKind::Identifier,
                                "nom de champ attendu",
                            )?
                            .lexeme
                        };

                        self.consume(
                            TokenKind::Colon,
                            "':' attendu après le nom du champ",
                        )?;

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

                self.consume(
                    TokenKind::RightBrace,
                    "'}' attendu après l'objet",
                )?;

                Ok(Expression::Object(fields))
            }

            _ => Err(ParserError {
                message: "Expression invalide".to_string(),
                line: token.line,
                column: token.column,
            }),
        }
    }

    fn parse_function_expression(
        &mut self,
    ) -> Result<Expression, ParserError> {
        self.consume(
            TokenKind::LeftParen,
            "'(' attendu après 'function' dans une fonction anonyme",
        )?;

        let mut params = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let param = self.consume(
                    TokenKind::Identifier,
                    "Nom de paramètre attendu",
                )?;

                params.push(param.lexeme);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightParen,
            "')' attendu après les paramètres",
        )?;

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu avant le corps de la fonction anonyme",
        )?;

        let body = self.parse_block_statement()?;

        Ok(Expression::Function { params, body })
    }

    // ============================================================
    // STATEMENTS
    // ============================================================

    fn statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let line = self.peek().line;
        let column = self.peek().column;

        let statements = if self.match_token(TokenKind::Import) {
            vec![self.parse_import_statement()?]
        } else if self.match_token(TokenKind::From) {
            vec![self.parse_from_import_statement()?]
        } else if self.match_token(TokenKind::Export) {
            vec![self.parse_export_statement()?]
        } else if self.match_token(TokenKind::Let) {
            self.parse_variable_declaration(true)?
        } else if self.match_token(TokenKind::Const) {
            self.parse_variable_declaration(false)?
        } else if self.match_token(TokenKind::Function) {
            vec![self.parse_function_statement()?]
        } else if self.match_token(TokenKind::Return) {
            vec![self.parse_return_statement()?]
        } else if self.match_token(TokenKind::Break) {
            vec![Statement::Break]
        } else if self.match_token(TokenKind::Continue) {
            vec![Statement::Continue]
        } else if self.match_token(TokenKind::Match) {
            vec![self.parse_match_statement()?]
        } else if self.match_token(TokenKind::Try) {
            vec![self.parse_try_statement()?]
        } else if self.match_token(TokenKind::Throw) {
            vec![self.parse_throw_statement()?]
        } else if self.match_token(TokenKind::If) {
            vec![self.parse_if_statement()?]
        } else if self.match_token(TokenKind::While) {
            vec![self.parse_while_statement()?]
        } else if self.match_token(TokenKind::For) {
            vec![self.parse_for_statement()?]
        } else if self.match_token(TokenKind::LeftBrace) {
            vec![Statement::Block(
                self.parse_block_statement()?,
            )]
        } else {
            vec![self.parse_expression_or_assignment()?]
        };

        self.match_token(TokenKind::Semicolon);

        let positioned = statements
            .into_iter()
            .map(|statement| Statement::Positioned {
                line,
                column,
                statement: Box::new(statement),
            })
            .collect();

        Ok(positioned)
    }

    // ============================================================
    // THROW
    // ============================================================

    fn parse_throw_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        if self.check(TokenKind::Semicolon)
            || self.check(TokenKind::RightBrace)
            || self.is_at_end()
        {
            return Err(ParserError {
                message: "Expression attendue après 'throw'".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        }

        let value = self.parse_expression()?;

        Ok(Statement::Throw { value })
    }

    // ============================================================
    // TRY / CATCH / FINALLY
    // ============================================================

    fn parse_try_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        /*
         * try {
         *     ...
         * }
         */
        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu après 'try'",
        )?;

        let try_body = self.parse_block_statement()?;

        /*
         * catch optionnel.
         *
         * catch (e) {
         *     ...
         * }
         */
        let mut catch_name = None;
        let mut catch_body = None;

        if self.match_token(TokenKind::Catch) {
            self.consume(
                TokenKind::LeftParen,
                "'(' attendu après 'catch'",
            )?;

            let error_name = self.consume(
                TokenKind::Identifier,
                "Nom de variable attendu dans 'catch'",
            )?;

            self.consume(
                TokenKind::RightParen,
                "')' attendu après le nom de l'erreur",
            )?;

            self.consume(
                TokenKind::LeftBrace,
                "'{' attendu après 'catch(...)'",
            )?;

            let body = self.parse_block_statement()?;

            catch_name = Some(error_name.lexeme);
            catch_body = Some(body);
        }

        /*
         * finally optionnel.
         *
         * finally {
         *     ...
         * }
         */
        let finally_body = if self.match_token(TokenKind::Finally) {
            self.consume(
                TokenKind::LeftBrace,
                "'{' attendu après 'finally'",
            )?;

            Some(self.parse_block_statement()?)
        } else {
            None
        };

        /*
         * Il faut au moins catch ou finally.
         *
         * Ceci est invalide :
         *
         * try {
         *     ...
         * }
         */
        if catch_body.is_none() && finally_body.is_none() {
            return Err(ParserError {
                message:
                    "'try' doit être suivi de 'catch' ou 'finally'"
                        .to_string(),
                line: self.previous().line,
                column: self.previous().column,
            });
        }

        Ok(Statement::Try {
            try_body,
            catch_name,
            catch_body,
            finally_body,
        })
    }

    // ============================================================
    // EXPRESSION / ASSIGNMENT
    // ============================================================

    fn parse_expression_or_assignment(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let expression = self.parse_expression()?;

        if self.match_token(TokenKind::Equal) {
            let value = self.parse_expression()?;

            let target =
                self.expression_to_assignment_target(expression)?;

            return Ok(Statement::Assignment { target, value });
        }

        if let Some(operator) = self.match_compound_assignment() {
            let value_expr = self.parse_expression()?;

            let target =
                self.expression_to_assignment_target(expression.clone())?;

            let left =
                Self::assignment_target_to_expression(&target);

            let value = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(value_expr),
            };

            return Ok(Statement::Assignment { target, value });
        }

        Ok(Statement::Expression { expression })
    }

    fn match_compound_assignment(
        &mut self,
    ) -> Option<BinaryOp> {
        let operator = match self.peek().kind {
            TokenKind::PlusEqual => BinaryOp::Add,
            TokenKind::MinusEqual => BinaryOp::Subtract,
            TokenKind::StarEqual => BinaryOp::Multiply,
            TokenKind::SlashEqual => BinaryOp::Divide,
            TokenKind::PercentEqual => BinaryOp::Modulo,
            _ => return None,
        };

        self.advance();

        Some(operator)
    }

    fn assignment_target_to_expression(
        target: &AssignmentTarget,
    ) -> Expression {
        match target {
            AssignmentTarget::Variable(name) => {
                Expression::Variable(name.clone())
            }

            AssignmentTarget::Index { object, index } => {
                Expression::Index {
                    object: object.clone(),
                    index: index.clone(),
                }
            }

            AssignmentTarget::Member { object, name } => {
                Expression::Member {
                    object: object.clone(),
                    name: name.clone(),
                }
            }
        }
    }

    fn expression_to_assignment_target(
        &self,
        expression: Expression,
    ) -> Result<AssignmentTarget, ParserError> {
        match expression {
            Expression::Variable(name) => {
                Ok(AssignmentTarget::Variable(name))
            }

            Expression::Index { object, index } => {
                Ok(AssignmentTarget::Index { object, index })
            }

            Expression::Member { object, name } => {
                Ok(AssignmentTarget::Member { object, name })
            }

            _ => {
                let token = self.peek();

                Err(ParserError {
                    message:
                        "Cible d'affectation invalide".to_string(),
                    line: token.line,
                    column: token.column,
                })
            }
        }
    }

    // ============================================================
    // IMPORT
    // ============================================================

    fn parse_import_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let mut path = Vec::new();

        let name = self.consume(
            TokenKind::Identifier,
            "Nom de module attendu après 'import'",
        )?;

        path.push(name.lexeme);

        while self.match_token(TokenKind::Dot) {
            let name = self.consume(
                TokenKind::Identifier,
                "Nom de module attendu après '.'",
            )?;

            path.push(name.lexeme);
        }

        Ok(Statement::Import { path })
    }

    fn parse_from_import_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let module = self.parse_module_path()?;

        self.consume(
            TokenKind::Import,
            "'import' attendu après le nom du module",
        )?;

        let mut items = Vec::new();

        loop {
            let name = self.consume(
                TokenKind::Identifier,
                "Nom exporté attendu",
            )?;

            let alias = if self.match_token(TokenKind::As) {
                Some(
                    self.consume(
                        TokenKind::Identifier,
                        "Alias attendu après 'as'",
                    )?
                    .lexeme,
                )
            } else {
                None
            };

            items.push(ImportItem {
                name: name.lexeme,
                alias,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(Statement::FromImport { module, items })
    }

    fn parse_export_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let statement = if self.match_token(TokenKind::Let) {
            let mut declarations =
                self.parse_variable_declaration(true)?;

            if declarations.len() != 1 {
                return Err(ParserError {
                    message:
                        "'export' accepte une seule déclaration"
                            .to_string(),
                    line: self.previous().line,
                    column: self.previous().column,
                });
            }

            declarations.remove(0)
        } else if self.match_token(TokenKind::Const) {
            let mut declarations =
                self.parse_variable_declaration(false)?;

            if declarations.len() != 1 {
                return Err(ParserError {
                    message:
                        "'export' accepte une seule déclaration"
                            .to_string(),
                    line: self.previous().line,
                    column: self.previous().column,
                });
            }

            declarations.remove(0)
        } else if self.match_token(TokenKind::Function) {
            self.parse_function_statement()?
        } else {
            return Err(ParserError {
                message:
                    "'export' doit être suivi de let, const ou function"
                        .to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };

        Ok(Statement::Export {
            statement: Box::new(statement),
        })
    }

    fn parse_module_path(
        &mut self,
    ) -> Result<ModulePath, ParserError> {
        let first = self.consume(
            TokenKind::Identifier,
            "Nom de module attendu",
        )?;

        let mut parts = vec![first.lexeme];

        while self.match_token(TokenKind::Dot) {
            let part = self.consume(
                TokenKind::Identifier,
                "Nom de module attendu après '.'",
            )?;

            parts.push(part.lexeme);
        }

        Ok(ModulePath::new(parts))
    }

    // ============================================================
    // DECLARATION
    // ============================================================

    fn parse_variable_declaration(
        &mut self,
        mutable: bool,
    ) -> Result<Vec<Statement>, ParserError> {
        let mut declarations = Vec::new();

        loop {
            let name = self.consume(
                TokenKind::Identifier,
                "Nom de variable attendu",
            )?;

            self.consume(
                TokenKind::Equal,
                "'=' attendu après le nom",
            )?;

            let value = self.parse_expression()?;

            declarations.push(Statement::Let {
                name: name.lexeme,
                value,
                mutable,
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(declarations)
    }

    // ============================================================
    // FUNCTION
    // ============================================================

    fn parse_function_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let name = self.consume(
            TokenKind::Identifier,
            "Nom de fonction attendu",
        )?;

        self.consume(
            TokenKind::LeftParen,
            "'(' attendu après le nom de fonction",
        )?;

        let mut params = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                let param = self.consume(
                    TokenKind::Identifier,
                    "Nom de paramètre attendu",
                )?;

                params.push(param.lexeme);

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(
            TokenKind::RightParen,
            "')' attendu après les paramètres",
        )?;

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu avant le corps",
        )?;

        let body = self.parse_block_statement()?;

        Ok(Statement::Function {
            name: name.lexeme,
            params,
            body,
        })
    }

    // ============================================================
    // RETURN
    // ============================================================

    fn parse_return_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let value = if self.check(TokenKind::Semicolon)
            || self.check(TokenKind::RightBrace)
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Return { value })
    }

    // ============================================================
    // BLOCK
    // ============================================================

    fn parse_block_statement(
        &mut self,
    ) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace)
            && !self.is_at_end()
        {
            let parsed = self.statement()?;

            statements.extend(parsed);
        }

        self.consume(
            TokenKind::RightBrace,
            "'}' attendu après le bloc",
        )?;

        Ok(statements)
    }

    // ============================================================
    // IF
    // ============================================================

    fn parse_if_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let condition = self.parse_expression()?;

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu après la condition",
        )?;

        let then_branch = self.parse_block_statement()?;

        let else_branch =
            if self.match_token(TokenKind::Else) {
                self.consume(
                    TokenKind::LeftBrace,
                    "'{' attendu après else",
                )?;

                Some(self.parse_block_statement()?)
            } else {
                None
            };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    // ============================================================
    // WHILE
    // ============================================================

    fn parse_while_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let condition = self.parse_expression()?;

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu après la condition",
        )?;

        let body = self.parse_block_statement()?;

        Ok(Statement::While { condition, body })
    }

    // ============================================================
    // FOR..IN
    // ============================================================

    fn parse_for_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let variable = self.consume(
            TokenKind::Identifier,
            "Nom de variable attendu après 'for'",
        )?;

        self.consume(
            TokenKind::In,
            "'in' attendu après le nom de variable",
        )?;

        let iterable = self.parse_expression()?;

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu avant le corps du for",
        )?;

        let body = self.parse_block_statement()?;

        Ok(Statement::ForIn {
            variable: variable.lexeme,
            iterable,
            body,
        })
    }

    // ============================================================
    // MATCH
    // ============================================================

    fn parse_match_statement(
        &mut self,
    ) -> Result<Statement, ParserError> {
        let value = self.parse_expression()?;

        self.consume(
            TokenKind::LeftBrace,
            "'{' attendu après l'expression match",
        )?;

        let mut arms = Vec::new();

        while !self.check(TokenKind::RightBrace)
            && !self.is_at_end()
        {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(TokenKind::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(
                TokenKind::FatArrow,
                "'=>' attendu après le pattern",
            )?;

            let body = if self.match_token(TokenKind::LeftBrace) {
                self.parse_block_statement()?
            } else {
                let expression = self.parse_expression()?;

                vec![Statement::Expression { expression }]
            };

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            self.match_token(TokenKind::Comma);
        }

        self.consume(
            TokenKind::RightBrace,
            "'}' attendu après les arms du match",
        )?;

        if arms.is_empty() {
            return Err(ParserError {
                message:
                    "Un match doit contenir au moins un arm"
                        .to_string(),
                line: self.previous().line,
                column: self.previous().column,
            });
        }

        Ok(Statement::Match { value, arms })
    }

    fn parse_pattern(
        &mut self,
    ) -> Result<Pattern, ParserError> {
        let mut patterns = vec![self.parse_pattern_atom()?];

        while self.match_token(TokenKind::Pipe) {
            patterns.push(self.parse_pattern_atom()?);
        }

        if patterns.len() == 1 {
            Ok(patterns.remove(0))
        } else {
            Ok(Pattern::Or(patterns))
        }
    }

    fn parse_pattern_atom(
        &mut self,
    ) -> Result<Pattern, ParserError> {
        let token = self.peek().clone();

        let mut pattern = match token.kind {
            // ========================================================
            // WILDCARD / BINDING
            // ========================================================
            TokenKind::Identifier => {
                let token = self.advance().clone();

                if token.lexeme == "_" {
                    Pattern::Wildcard
                } else {
                    Pattern::Binding(token.lexeme)
                }
            }

            // ========================================================
            // LITTÉRAUX
            // ========================================================
            TokenKind::Number => {
                let token = self.advance().clone();

                let is_float =
                    token.lexeme.contains('.')
                        || token.lexeme.contains(['e', 'E']);

                if is_float {
                    let value =
                        token.lexeme.parse::<f64>().map_err(|_| {
                            ParserError {
                                message: format!(
                                    "Nombre flottant invalide '{}'",
                                    token.lexeme
                                ),
                                line: token.line,
                                column: token.column,
                            }
                        })?;

                    Pattern::Literal(Literal::Float(value))
                } else {
                    let value =
                        token.lexeme.parse::<i64>().map_err(|_| {
                            ParserError {
                                message: format!(
                                    "Nombre entier invalide '{}'",
                                    token.lexeme
                                ),
                                line: token.line,
                                column: token.column,
                            }
                        })?;

                    Pattern::Literal(Literal::Integer(value))
                }
            }

            TokenKind::String => {
                let token = self.advance().clone();

                Pattern::Literal(Literal::String(token.lexeme))
            }

            TokenKind::True => {
                self.advance();

                Pattern::Literal(Literal::Bool(true))
            }

            TokenKind::False => {
                self.advance();

                Pattern::Literal(Literal::Bool(false))
            }

            TokenKind::Nil => {
                self.advance();

                Pattern::Literal(Literal::Nil)
            }

            // ========================================================
            // TABLEAU
            // ========================================================
            TokenKind::LeftBracket => {
                self.advance();

                let mut patterns = Vec::new();

                if !self.check(TokenKind::RightBracket) {
                    loop {
                        patterns.push(self.parse_pattern()?);

                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }

                        if self.check(TokenKind::RightBracket) {
                            break;
                        }
                    }
                }

                self.consume(
                    TokenKind::RightBracket,
                    "']' attendu après le pattern tableau",
                )?;

                Pattern::Array(patterns)
            }

            // ========================================================
            // PARENTHESES
            // ========================================================
            TokenKind::LeftParen => {
                self.advance();

                let pattern = self.parse_pattern()?;

                self.consume(
                    TokenKind::RightParen,
                    "')' attendu après le pattern",
                )?;

                pattern
            }

            // ========================================================
            // NOMBRE NÉGATIF
            // ========================================================
            TokenKind::Minus => {
                self.advance();

                let token = self.consume(
                    TokenKind::Number,
                    "Nombre attendu après '-' dans un pattern",
                )?;

                let is_float =
                    token.lexeme.contains('.')
                        || token.lexeme.contains(['e', 'E']);

                if is_float {
                    let value =
                        token.lexeme.parse::<f64>().map_err(|_| {
                            ParserError {
                                message: format!(
                                    "Nombre flottant invalide '-{}'",
                                    token.lexeme
                                ),
                                line: token.line,
                                column: token.column,
                            }
                        })?;

                    Pattern::Literal(Literal::Float(-value))
                } else {
                    let value =
                        token.lexeme.parse::<i64>().map_err(|_| {
                            ParserError {
                                message: format!(
                                    "Nombre entier invalide '-{}'",
                                    token.lexeme
                                ),
                                line: token.line,
                                column: token.column,
                            }
                        })?;

                    Pattern::Literal(Literal::Integer(-value))
                }
            }

            _ => {
                return Err(ParserError {
                    message: "Pattern invalide".to_string(),
                    line: token.line,
                    column: token.column,
                });
            }
        };

        // ============================================================
        // RANGE
        // ============================================================

        if self.match_token(TokenKind::Range) {
            let end = self.parse_pattern_atom()?;

            pattern = Pattern::Range {
                start: Box::new(pattern),
                end: Box::new(end),
                inclusive: false,
            };
        } else if self.match_token(TokenKind::RangeInclusive) {
            let end = self.parse_pattern_atom()?;

            pattern = Pattern::Range {
                start: Box::new(pattern),
                end: Box::new(end),
                inclusive: true,
            };
        }

        Ok(pattern)
    }
}