use std::fmt::{Debug, Display, Error, Formatter};
use std::iter::Iterator;
use std::collections::VecDeque;

use sqlparser::{parser::Parser, dialect::SQLiteDialect};
use sqlparser::ast::{Statement, SetExpr, Expr, OrderByExpr, Value, BinaryOperator};

use crate::Context;

use crate::lang::{IteratorItem, Op, PseudoOp};
use crate::lang::{RuntimeError, RuntimeOp, RuntimeMsg, TypePrimitive};
use crate::processing::general::FileIter;
use crate::Item;

#[derive(Debug)]
pub struct RawSqlQuery {
    context: Option<Context>,
    file_iter: Option<FileIter>,
    match_rule: Option<MatchRule>,
    sort_by: Option<SortRule>,
    items_buffer: VecDeque<IteratorItem>,
    raw_query: String,
    has_tried: bool,
}

impl RawSqlQuery {
    pub fn emit(query_str: &str) -> Result<Self, RuntimeMsg> {
        let mut statements = Parser::parse_sql(&SQLiteDialect{}, query_str).map_err(|e| RuntimeMsg(format!("Could not parse SQL query: {}", e)))?;
        if statements.len() == 1 {
            if let Statement::Query(mut query) = statements.remove(0) {
                let matching = if let SetExpr::Select(select) = *query.body {
                    if let Some(selection) = select.selection {
                        Some(MatchRule::from_parsed(selection)?)
                    } else {
                        None
                    }
                } else {
                    return Err(RuntimeMsg("Unsupported SELECT syntax in SQL".to_owned()));
                };
                let ordering = if !query.order_by.is_empty() {
                    Some(SortRule::from_parsed(query.order_by.remove(0))?)
                } else {
                    None
                };
                Ok(Self {
                    context: None,
                    file_iter: None,
                    match_rule: matching,
                    sort_by: ordering,
                    items_buffer: VecDeque::new(),
                    raw_query: query_str.to_owned(),
                    has_tried: false,
                })
            } else {
                Err(RuntimeMsg("Expected SQL SELECT statement".to_owned()))
            }
        } else {
            Err(RuntimeMsg(format!("Expected exactly 1 SQL SELECT statement, got {} statements", statements.len())))
        }
    }

    #[inline]
    fn matches_filters(&self, item: &Item) -> bool {
        if let Some(match_rule) = &self.match_rule {
            match_rule.is_match(item)
        } else {
            true
        }
    }
}

impl Display for RawSqlQuery {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "sql(`{}`)", self.raw_query)
    }
}

impl std::clone::Clone for RawSqlQuery {
    fn clone(&self) -> Self {
        Self {
            context: None,
            file_iter: None,
            match_rule: self.match_rule.clone(),
            sort_by: self.sort_by.clone(),
            items_buffer: VecDeque::with_capacity(self.items_buffer.len()),
            raw_query: self.raw_query.clone(),
            has_tried: self.has_tried,
        }
    }
}

impl Iterator for RawSqlQuery {
    type Item = IteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.file_iter.is_none() {
            if self.has_tried {
                return None;
            } else {
                self.has_tried = true;
            }
            let iter = self.context.as_mut().unwrap().filesystem.raw(
                None,
                None,
                true,
            );
            self.file_iter = Some(match iter {
                Ok(x) => x,
                Err(e) => return Some(Err(e.with(RuntimeOp(PseudoOp::from_printable(self))))),
            });
        }
        if let Some(sort_by) = &self.sort_by {
            let old_len = self.items_buffer.len();
            while let Some(item) = self.file_iter.as_mut().unwrap().next() {
                match item {
                    Ok(item) => {
                        // apply filter
                        if self.matches_filters(&item) {
                            self.items_buffer.push_back(Ok(item));
                        }
                    },
                    Err(e) => self.items_buffer.push_back(Err(RuntimeError {
                        line: 0,
                        op: PseudoOp::from_printable(self),
                        msg: e,
                    }))
                }
            }
            let new_len = self.items_buffer.len();
            if old_len != new_len {
                // file_iter was just completed, so buffer needs sorting
                sort_by.sort_vecdeque(&mut self.items_buffer);
            }
            self.items_buffer.pop_front()
        } else {
            while let Some(item) = self.file_iter.as_mut().unwrap().next() {
                match item {
                    Ok(item) => {
                        // apply filter
                        if self.matches_filters(&item) {
                            return Some(Ok(item));
                        }
                    },
                    Err(e) => return Some(Err(RuntimeError {
                        line: 0,
                        op: PseudoOp::from_printable(self),
                        msg: e,
                    }))
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.file_iter.as_ref().map(|x| x.size_hint()).unwrap_or_default()
    }
}

impl Op for RawSqlQuery {
    fn enter(&mut self, ctx: Context) {
        self.context = Some(ctx)
    }

    fn escape(&mut self) -> Context {
        self.context.take().unwrap()
    }

    fn is_resetable(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn dup(&self) -> Box<dyn Op> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
enum MatchRule {
    Like { field: String, pattern: LikePattern, negated: bool },
    CompareVal { field: String, value: TypePrimitive, comparison: [i8; 2] },
    CompareFields { field_a: String, field_b: String, comparison: [i8; 2] },
    And { a: Box<MatchRule>, b: Box<MatchRule> },
    Or { a: Box<MatchRule>, b: Box<MatchRule> },
}

impl MatchRule {
    #[inline]
    fn is_match(&self, item: &Item) -> bool {
        match self {
            Self::Like { field, pattern, negated } => {
                if let Some(TypePrimitive::String(val)) = item.field(field) {
                    pattern.is_match(val) != *negated
                } else {
                    *negated
                }
            },
            Self::CompareVal { field, value, comparison } => {
                if let Some(val) = item.field(field) {
                    match val.compare(value) {
                        Ok(cmp) => comparison[0] == cmp || comparison[1] == cmp,
                        Err(_) => comparison[0] != 0 && comparison[1] != 0,
                    }
                } else {
                    match TypePrimitive::Empty.compare(value) {
                        Ok(cmp) => comparison[0] == cmp || comparison[1] == cmp,
                        Err(_) => comparison[0] != 0 && comparison[1] != 0,
                    }
                }
            },
            Self::CompareFields { field_a, field_b, comparison} => {
                if let Some(val_a) = item.field(field_a) {
                    if let Some(val_b) = item.field(field_b) {
                        match val_a.compare(val_b) {
                            Ok(cmp) => comparison[0] == cmp || comparison[1] == cmp,
                            Err(_) => comparison[0] != 0 && comparison[1] != 0,
                        }
                    } else {
                        match val_a.compare(&TypePrimitive::Empty) {
                            Ok(cmp) => comparison[0] == cmp || comparison[1] == cmp,
                            Err(_) => comparison[0] != 0 && comparison[1] != 0,
                        }
                    }
                } else {
                    if let Some(val_b) = item.field(field_b) {
                        match TypePrimitive::Empty.compare(val_b) {
                            Ok(cmp) => comparison[0] == cmp || comparison[1] == cmp,
                            Err(_) => comparison[0] != 0 && comparison[1] != 0,
                        }
                    } else {
                        match TypePrimitive::Empty.compare(&TypePrimitive::Empty) {
                            Ok(cmp) => comparison[0] == cmp || comparison[1] == cmp,
                            Err(_) => comparison[0] != 0 && comparison[1] != 0,
                        }
                    }
                }
            },
            Self::And { a, b } => {
                a.is_match(item) && b.is_match(item)
            },
            Self::Or { a, b } => {
                a.is_match(item) || b.is_match(item)
            },
        }
    }

    #[inline]
    fn from_parsed(expr: Expr) -> Result<Self, RuntimeMsg> {
        match expr {
            Expr::IsFalse(x) => if let Expr::Identifier(id) = *x {
                Ok(Self::CompareVal{ field: id.value, value:TypePrimitive::Bool(false), comparison: [0, 0] })
            } else {
                Err(RuntimeMsg(format!("Unsupported SQL IS FALSE syntax: {}", x)))
            },
            Expr::IsNotFalse(x) => if let Expr::Identifier(id) = *x {
                Ok(Self::CompareVal{ field: id.value, value:TypePrimitive::Bool(false), comparison: [1, -1] })
            } else {
                Err(RuntimeMsg(format!("Unsupported SQL IS NOT FALSE syntax: {}", x)))
            },
            Expr::IsTrue(x) => if let Expr::Identifier(id) = *x {
                Ok(Self::CompareVal{ field: id.value, value:TypePrimitive::Bool(true), comparison: [0, 0] })
            } else {
                Err(RuntimeMsg(format!("Unsupported SQL IS TRUE syntax: {}", x)))
            },
            Expr::IsNotTrue(x) => if let Expr::Identifier(id) = *x {
                Ok(Self::CompareVal{ field: id.value, value:TypePrimitive::Bool(true), comparison: [1, -1] })
            } else {
                Err(RuntimeMsg(format!("Unsupported SQL IS NOT TRUE syntax: {}", x)))
            },
            Expr::IsNull(x) => if let Expr::Identifier(id) = *x {
                Ok(Self::CompareVal{ field: id.value, value:TypePrimitive::Empty, comparison: [0, 0] })
            } else {
                Err(RuntimeMsg(format!("Unsupported SQL IS NULL syntax: {}", x)))
            },
            Expr::IsNotNull(x) => if let Expr::Identifier(id) = *x {
                Ok(Self::CompareVal{ field: id.value, value:TypePrimitive::Empty, comparison: [1, -1] })
            } else {
                Err(RuntimeMsg(format!("Unsupported SQL IS NOT NULL syntax: {}", x)))
            },
            Expr::Like { negated, expr, pattern, .. } => match (*expr, *pattern) {
                (Expr::Identifier(expr), Expr::Value(Value::SingleQuotedString(pattern))) =>
                    Ok(Self::Like{ field: expr.value, negated: negated, pattern: LikePattern::from_string(pattern) }),
                (x, y) => Err(RuntimeMsg(format!("Unsupported SQL LIKE syntax: {} LIKE {}", x, y)))
            },
            Expr::ILike { negated, expr, pattern, .. } => match (*expr, *pattern) {
                (Expr::Identifier(expr), Expr::Value(Value::SingleQuotedString(pattern))) =>
                    Ok(Self::Like{ field: expr.value, negated: negated, pattern: LikePattern::from_string(pattern) }),
                (x, y) => Err(RuntimeMsg(format!("Unsupported SQL ILIKE syntax: {} ILIKE {}", x, y)))
            },
            Expr::Nested(x) => Self::from_parsed(*x),
            Expr::BinaryOp { left, op, right } => {
                if let BinaryOperator::And = op {
                    Ok(Self::And { a: Box::new(Self::from_parsed(*left)?), b: Box::new(Self::from_parsed(*right)?) })
                } else if let BinaryOperator::Or = op {
                    Ok(Self::Or { a: Box::new(Self::from_parsed(*left)?), b: Box::new(Self::from_parsed(*right)?) })
                } else {
                    match (*left, *right) {
                        (Expr::Identifier(left), Expr::Value(right)) =>
                            Ok(Self::CompareVal {
                                field: left.value,
                                value: value_to_primitive(right)?,
                                comparison: binary_op_to_compare(op)?,
                            }),
                        (Expr::Identifier(left), Expr::Identifier(right)) =>
                            Ok(Self::CompareFields {
                                field_a: left.value,
                                field_b: right.value,
                                comparison: binary_op_to_compare(op)?,
                            }),
                        (x, y) => Err(RuntimeMsg(format!("Unsupported SQL operator syntax: {} {} {}", x, op, y)))
                    }
                }
            },
            x => Err(RuntimeMsg(format!("Unsupported SQL WHERE syntax: {}", x)))
        }
    }
}

#[inline]
fn binary_op_to_compare(op: BinaryOperator) -> Result<[i8; 2], RuntimeMsg> {
    match op {
        BinaryOperator::Gt => Ok([1, 1]),
        BinaryOperator::Lt => Ok([-1, -1]),
        BinaryOperator::GtEq => Ok([1, 0]),
        BinaryOperator::LtEq => Ok([-1, 0]),
        BinaryOperator::Eq => Ok([0, 0]),
        BinaryOperator::NotEq => Ok([-1, 1]),
        x => Err(RuntimeMsg(format!("Unsupported SQL operator syntax: {}", x)))
    }
}

#[inline]
fn value_to_primitive(val: Value) -> Result<TypePrimitive, RuntimeMsg> {
    match val {
        Value::Number(s, _) => Ok(TypePrimitive::parse(s)),
        Value::SingleQuotedString(s) => Ok(TypePrimitive::String(s)),
        Value::DoubleQuotedString(s) => Ok(TypePrimitive::String(s)),
        Value::Boolean(b) => Ok(TypePrimitive::Bool(b)),
        Value::Null => Ok(TypePrimitive::Empty),
        x => Err(RuntimeMsg(format!("Unsupported SQL operator syntax: {}", x)))
    }
}

#[derive(Debug, Clone)]
enum LikePattern {
    EndsWith(String),
    StartWith(String),
    Contains(String),
    Is(String),
}

impl LikePattern {
    #[inline]
    fn is_match(&self, text: &str) -> bool {
        match self {
            Self::EndsWith(p) => text.to_lowercase().ends_with(p),
            Self::StartWith(p) => text.to_lowercase().starts_with(p),
            Self::Contains(p) => text.to_lowercase().contains(p),
            Self::Is(p) => &text.to_lowercase() == p,
        }
    }

    #[inline]
    fn from_string(pattern: String) -> Self {
        match (pattern.starts_with('%'), pattern.ends_with('%')) {
            (false, true) => Self::EndsWith(pattern[..pattern.len()-1].to_owned()),
            (true, false) => Self::StartWith(pattern[1..].to_owned()),
            (true, true) => Self::Contains(pattern[1..pattern.len()-1].to_owned()),
            (false, false) => Self::Is(pattern),
        }
    }
}

#[derive(Debug, Clone)]
enum SortRule {
    Ascending(String),
    Descending(String),
}

impl SortRule {
    #[inline]
    fn sort_vecdeque(&self, list: &mut VecDeque<IteratorItem>) {
        let buffer = list.make_contiguous();
        match self {
            Self::Ascending(field) => {
                buffer.sort_by(|b, a| {
                    if let Ok(a) = a {
                        if let Some(a_field) = a.field(field) {
                            if let Ok(b) = b {
                                if let Some(b_field) = b.field(field) {
                                    return a_field
                                        .for_compare()
                                        .partial_cmp(&b_field.for_compare())
                                        .unwrap_or(std::cmp::Ordering::Equal);
                                }
                            }
                        }
                    }
                    std::cmp::Ordering::Equal
                });
            },
            Self::Descending(field) => {
                buffer.sort_by(|a, b| {
                    if let Ok(a) = a {
                        if let Some(a_field) = a.field(field) {
                            if let Ok(b) = b {
                                if let Some(b_field) = b.field(field) {
                                    return a_field
                                        .for_compare()
                                        .partial_cmp(&b_field.for_compare())
                                        .unwrap_or(std::cmp::Ordering::Equal);
                                }
                            }
                        }
                    }
                    std::cmp::Ordering::Equal
                });
            }
        }
    }

    fn from_parsed(order: OrderByExpr) -> Result<Self, RuntimeMsg> {
        let field = if let Expr::Identifier(id) = order.expr {
            id.value
        } else {
            return Err(RuntimeMsg(format!("Unsupported SQL syntax: ORDER BY value must be a field identifier")));
        };
        if order.asc.unwrap_or(true) {
            Ok(Self::Ascending(field))
        } else {
            Ok(Self::Descending(field))
        }
    }
}
