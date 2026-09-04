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

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParserError> {
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
        // Un '.' ou un 'e'/'E' dans le lexème signale un littéral flottant
        // (1.5, 1.5e3) ; sinon c'est un entier (42, 0xFF -> "255", 0b1010 -> "10").
        let is_float = token.lexeme.contains('.') || token.lexeme.contains(['e', 'E']);

        if is_float {
            let value = token.lexeme.parse::<f64>().map_err(|_| ParserError {
                message: format!("Nombre flottant invalide '{}'", token.lexeme),
                line: token.line,
                column: token.column,
            })?;

            Ok(Expression::Literal(Literal::Float(value)))
        } else {
            let value = token.lexeme.parse::<i64>().map_err(|_| ParserError {
                message: format!("Nombre entier invalide '{}'", token.lexeme),
                line: token.line,
                column: token.column,
            })?;

            Ok(Expression::Literal(Literal::Integer(value)))
        }
    }

    fn parse_string(&self, token: Token) -> Result<Expression, ParserError> {
        let value = token.lexeme.trim_matches('"').to_string();

        Ok(Expression::Literal(Literal::String(value)))
    }

    // ============================================================
    // EXPRESSIONS
    // ============================================================

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.ternary()
    }

    fn ternary(&mut self) -> Result<Expression, ParserError> {
        let condition = self.logical_or()?;

        if self.match_token(TokenKind::Question) {
            // Le membre "then" accepte une expression complète (ternaire imbriqué inclus).
            let then_expr = self.parse_expression()?;

            self.consume(TokenKind::Colon, "':' attendu dans l'expression ternaire")?;

            // Droit-associatif : "a ? b : c ? d : e" == "a ? b : (c ? d : e)".
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

    // Précédence bitwise façon C : | plus lâche que ^, plus lâche que &.
    // Placés entre logique (&&/||) et comparaison, comme dans la plupart
    // des langages qui ont les deux familles d'opérateurs.

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

    // << et >> : plus serrés que la comparaison, plus lâches que +/-
    // (précédence C standard).
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
            // --------------------------------------------------------
            // Appel de fonction
            //
            // foo(...)
            // a.push(...)
            // a[0](...)
            // --------------------------------------------------------
            if self.match_token(TokenKind::LeftParen) {
                expression = self.parse_call(expression)?;

                continue;
            }

            // --------------------------------------------------------
            // Indexation
            //
            // a[0]
            // matrix[0][1]
            // matrix[0][1][2]
            // --------------------------------------------------------
            if self.match_token(TokenKind::LeftBracket) {
                let index = self.parse_expression()?;

                self.consume(TokenKind::RightBracket, "']' attendu après l'index")?;

                expression = Expression::Index {
                    object: Box::new(expression),
                    index: Box::new(index),
                };

                continue;
            }

            // --------------------------------------------------------
            // Accès membre
            //
            // a.length
            // a.push
            // a.pop
            // matrix[0].length
            // matrix[0].push
            // --------------------------------------------------------
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

            TokenKind::Nil => Ok(Expression::Literal(Literal::Nil)),

            // ----------------------------------------------------
            // Parenthèses
            // ----------------------------------------------------
            TokenKind::LeftParen => {
                let expression = self.parse_expression()?;

                self.consume(TokenKind::RightParen, "')' attendu après l'expression")?;

                Ok(expression)
            }

            // ----------------------------------------------------
            // Tableau
            // ----------------------------------------------------
            TokenKind::LeftBracket => {
                let mut elements = Vec::new();

                if !self.check(TokenKind::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);

                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }

                        // Autorise :
                        //
                        // [1, 2, 3,]
                        if self.check(TokenKind::RightBracket) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RightBracket, "']' attendu après le tableau")?;

                Ok(Expression::Array(elements))
            }

            // ----------------------------------------------------
            // Objet : { clé: expression, clé2: expression2, ... }
            //
            // Aucune ambiguïté avec les blocs de statements : ceux-ci ne
            // sont reconnus qu'au niveau statement() (en tout début de
            // statement), jamais ici dans primary(), qui n'est atteint
            // qu'en position d'expression (après '=', comme argument,
            // comme élément de tableau, etc.).
            // ----------------------------------------------------
            TokenKind::LeftBrace => {
                let mut fields: Vec<(String, Expression)> = Vec::new();

                if !self.check(TokenKind::RightBrace) {
                    loop {
                        // La clé peut être un identifiant ({ name: ... })
                        // ou une chaîne ({ "name": ... }), pratique pour
                        // les clés qui ne sont pas des identifiants valides.
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

                        // Autorise la virgule finale : { a: 1, b: 2, }
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

    // ============================================================
    // STATEMENTS
    // ============================================================

    fn statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        // Position du premier token de CE statement, capturée avant tout
        // dispatch — c'est elle qui enveloppera chaque statement produit
        // ci-dessous, quel que soit son type.
        // let line = self.peek().line;
        // let column = self.peek().column;

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
        } else if self.match_token(TokenKind::If) {
            vec![self.parse_if_statement()?]
        } else if self.match_token(TokenKind::While) {
            vec![self.parse_while_statement()?]
        } else if self.match_token(TokenKind::For) {
            vec![self.parse_for_statement()?]
        } else if self.match_token(TokenKind::LeftBrace) {
            vec![Statement::Block(self.parse_block_statement()?)]
        } else {
            vec![self.parse_expression_or_assignment()?]
        };

        // ';' optionnel
        self.match_token(TokenKind::Semicolon);

        // Enveloppe CHAQUE statement produit (un `let a = 1, b = 2;`
        // produit plusieurs statements pour un seul point de départ — ils
        // partagent tous la même position, ce qui est correct : c'est la
        // ligne où la déclaration commence qui importe pour le diagnostic).
        let positioned = statements
            .into_iter()
            .map(|statement| Statement::Positioned {
                // line,
                // column,
                statement: Box::new(statement),
            })
            .collect();

        Ok(positioned)
    }

    // ============================================================
    // EXPRESSION / ASSIGNMENT
    // ============================================================

    fn parse_expression_or_assignment(&mut self) -> Result<Statement, ParserError> {
        let expression = self.parse_expression()?;

        if self.match_token(TokenKind::Equal) {
            let value = self.parse_expression()?;

            let target = self.expression_to_assignment_target(expression)?;

            return Ok(Statement::Assignment { target, value });
        }

        // Affectations composées : x += v  ==  x = x + v
        //
        // ⚠️ Pour une cible Index ou Member, `object`/`index` sont évalués
        // DEUX FOIS dans le bytecode généré (une fois pour lire l'ancienne
        // valeur, une fois pour écrire la nouvelle) : `arr[f()] += 1`
        // appelle f() deux fois. Sans effet pour une variable simple.
        if let Some(operator) = self.match_compound_assignment() {
            let value_expr = self.parse_expression()?;

            let target = self.expression_to_assignment_target(expression.clone())?;

            let left = Self::assignment_target_to_expression(&target);

            let value = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(value_expr),
            };

            return Ok(Statement::Assignment { target, value });
        }

        Ok(Statement::Expression { expression })
    }

    /// Consomme le token courant s'il s'agit d'une affectation composée
    /// (+=, -=, *=, /=, %=) et retourne l'opérateur binaire équivalent.
    fn match_compound_assignment(&mut self) -> Option<BinaryOp> {
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

    /// Reconstruit l'expression correspondant à une cible d'affectation,
    /// pour former le côté gauche de la désucrisation `x op= v -> x = x op v`.
    fn assignment_target_to_expression(target: &AssignmentTarget) -> Expression {
        match target {
            AssignmentTarget::Variable(name) => Expression::Variable(name.clone()),

            AssignmentTarget::Index { object, index } => Expression::Index {
                object: object.clone(),
                index: index.clone(),
            },

            AssignmentTarget::Member { object, name } => Expression::Member {
                object: object.clone(),
                name: name.clone(),
            },
        }
    }

    fn expression_to_assignment_target(
        &self,
        expression: Expression,
    ) -> Result<AssignmentTarget, ParserError> {
        match expression {
            Expression::Variable(name) => Ok(AssignmentTarget::Variable(name)),

            Expression::Index { object, index } => Ok(AssignmentTarget::Index { object, index }),

            Expression::Member { object, name } => Ok(AssignmentTarget::Member { object, name }),

            _ => {
                let token = self.peek();

                Err(ParserError {
                    message: "Cible d'affectation invalide".to_string(),
                    line: token.line,
                    column: token.column,
                })
            }
        }
    }

    fn parse_import_statement(&mut self) -> Result<Statement, ParserError> {
        let mut path = Vec::new();

        let name = self.consume(
            TokenKind::Identifier,
            "Nom de module attendu après 'import'",
        )?;

        path.push(name.lexeme);

        while self.match_token(TokenKind::Dot) {
            let name = self.consume(TokenKind::Identifier, "Nom de module attendu après '.'")?;

            path.push(name.lexeme);
        }

        Ok(Statement::Import { path })
    }

    fn parse_from_import_statement(&mut self) -> Result<Statement, ParserError> {
        let module = self.parse_module_path()?;

        self.consume(TokenKind::Import, "'import' attendu après le nom du module")?;

        let mut items = Vec::new();

        loop {
            let name = self.consume(TokenKind::Identifier, "Nom exporté attendu")?;

            let alias = if self.match_token(TokenKind::As) {
                Some(
                    self.consume(TokenKind::Identifier, "Alias attendu après 'as'")?
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

    fn parse_export_statement(&mut self) -> Result<Statement, ParserError> {
        let statement = if self.match_token(TokenKind::Let) {
            let mut declarations = self.parse_variable_declaration(true)?;

            if declarations.len() != 1 {
                return Err(ParserError {
                    message: "'export' accepte une seule déclaration".to_string(),
                    line: self.previous().line,
                    column: self.previous().column,
                });
            }

            declarations.remove(0)
        } else if self.match_token(TokenKind::Const) {
            let mut declarations = self.parse_variable_declaration(false)?;

            if declarations.len() != 1 {
                return Err(ParserError {
                    message: "'export' accepte une seule déclaration".to_string(),
                    line: self.previous().line,
                    column: self.previous().column,
                });
            }

            declarations.remove(0)
        } else if self.match_token(TokenKind::Function) {
            self.parse_function_statement()?
        } else {
            return Err(ParserError {
                message: "'export' doit être suivi de let, const ou function".to_string(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };

        Ok(Statement::Export {
            statement: Box::new(statement),
        })
    }

    fn parse_module_path(&mut self) -> Result<ModulePath, ParserError> {
        let first = self.consume(TokenKind::Identifier, "Nom de module attendu")?;

        let mut parts = vec![first.lexeme];

        while self.match_token(TokenKind::Dot) {
            let part = self.consume(TokenKind::Identifier, "Nom de module attendu après '.'")?;

            parts.push(part.lexeme);
        }

        Ok(ModulePath::new(parts))
    }

    // ============================================================
    // DECLARATION
    // ============================================================

    fn parse_variable_declaration(&mut self, mutable: bool) -> Result<Vec<Statement>, ParserError> {
        let mut declarations = Vec::new();

        loop {
            let name = self.consume(TokenKind::Identifier, "Nom de variable attendu")?;

            self.consume(TokenKind::Equal, "'=' attendu après le nom")?;

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

    fn parse_function_statement(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenKind::Identifier, "Nom de fonction attendu")?;

        self.consume(TokenKind::LeftParen, "'(' attendu après le nom de fonction")?;

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

        self.consume(TokenKind::LeftBrace, "'{' attendu avant le corps")?;

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

    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
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

    fn parse_block_statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let parsed = self.statement()?;

            statements.extend(parsed);
        }

        self.consume(TokenKind::RightBrace, "'}' attendu après le bloc")?;

        Ok(statements)
    }

    // ============================================================
    // IF
    // ============================================================

    fn parse_if_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(TokenKind::LeftParen, "'(' attendu après if")?;

        let condition = self.parse_expression()?;

        self.consume(TokenKind::RightParen, "')' attendu après la condition")?;

        self.consume(TokenKind::LeftBrace, "'{' attendu après la condition")?;

        let then_branch = self.parse_block_statement()?;

        let else_branch = if self.match_token(TokenKind::Else) {
            self.consume(TokenKind::LeftBrace, "'{' attendu après else")?;

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

    fn parse_while_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(TokenKind::LeftParen, "'(' attendu après while")?;

        let condition = self.parse_expression()?;

        self.consume(TokenKind::RightParen, "')' attendu après la condition")?;

        self.consume(TokenKind::LeftBrace, "'{' attendu après la condition")?;

        let body = self.parse_block_statement()?;

        Ok(Statement::While { condition, body })
    }

    // ============================================================
    // FOR..IN
    // ============================================================
    //
    // Syntaxe : for variable in iterable { corps }
    //
    //   for x in [1, 2, 3] { println(x); }
    //   for i in range(10) { println(i); }
    //
    // Le for classique (style C : init; condition; incrément) n'existe
    // plus dans Kastel — for..in est désormais la seule forme de boucle
    // for, à la façon de Python.

    fn parse_for_statement(&mut self) -> Result<Statement, ParserError> {
        let variable = self.consume(TokenKind::Identifier, "Nom de variable attendu après 'for'")?;

        self.consume(TokenKind::In, "'in' attendu après le nom de variable")?;

        let iterable = self.parse_expression()?;

        self.consume(TokenKind::LeftBrace, "'{' attendu avant le corps du for")?;

        let body = self.parse_block_statement()?;

        Ok(Statement::ForIn {
            variable: variable.lexeme,
            iterable,
            body,
        })
    }

}