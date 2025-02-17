use std::{collections::HashMap, rc::Rc, sync::Arc};

use hemtt_common::{
    position::Position,
    reporting::{Symbol, Token},
};
use strsim::levenshtein;

use crate::definition::Definition;

type InnerDefines = HashMap<Arc<str>, (Rc<Token>, Definition)>;

#[derive(Clone, Default)]
/// `HashMap` of all current defines
pub struct Defines {
    global: InnerDefines,
    stack: Vec<(Arc<str>, InnerDefines)>,

    counter: u16,
}

/// Built-in macros that HEMTT supports, constants
const BUILTIN_CONST: [(&str, u8); 2] = [("__ARMA__", 1), ("__ARMA3__", 1)];

/// Built-in macros that HEMTT supports, generated by the preprocessor
const BUILTIN_GEN: [&str; 4] = ["__COUNTER__", "__COUNTER_RESET__", "__FILE__", "__LINE__"];

/// Built-in macros that HEMTT intentionally does not support
const BUILTIN_PROTEST: [&str; 16] = [
    "__DATE_ARR__",
    "__DATE_STR__",
    "__DATE_STR_ISO8601__",
    "__TIME__",
    "__TIME_UTC__",
    "__TIMESTAMP_UTC__",
    "__RAND_INT*__",
    "__RAND_UINT*__",
    "__GAME_VER__",
    "__GAME_VER_MAJ__",
    "__GAME_VER_MIN__",
    "__GAME_BUILD__",
    "__A3_DIAG__",
    "__A3_DEBUG__",
    "__EXEC",
    "__EVAL",
];

impl Defines {
    pub fn is_builtin(key: &str) -> bool {
        BUILTIN_GEN.contains(&key)
            || BUILTIN_PROTEST.contains(&key)
            || BUILTIN_CONST.iter().any(|(k, _)| *k == key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        if BUILTIN_GEN.contains(&key) {
            return true;
        }
        if let Some(last) = self.stack.last() {
            if *last.0 == *key {
                return false;
            }
            if last.1.contains_key(key) {
                return true;
            }
        }
        self.global.contains_key(key)
    }

    pub fn get_with_gen(
        &mut self,
        key: &Rc<Token>,
        site: Option<&Position>,
    ) -> Option<(Rc<Token>, Definition)> {
        let ident = key.to_string();
        if let Some(site) = site {
            if BUILTIN_GEN.contains(&ident.as_str()) {
                match ident.as_str() {
                    "__COUNTER__" => {
                        let counter = self.counter;
                        self.counter += 1;
                        return Some((
                            key.clone(),
                            Definition::Value(vec![Rc::new(Token::new(
                                Symbol::Digit(counter.into()),
                                key.position().clone(),
                            ))]),
                        ));
                    }
                    "__COUNTER_RESET__" => {
                        self.counter = 0;
                        return Some((key.clone(), Definition::Void));
                    }
                    "__FILE__" => {
                        return Some((
                            key.clone(),
                            Definition::Value(vec![
                                Rc::new(Token::new(Symbol::DoubleQuote, key.position().clone())),
                                Rc::new(site.path().workspace().project().map_or_else(
                                    || {
                                        Token::new(
                                            Symbol::Word(site.path().as_str().to_string()),
                                            key.position().clone(),
                                        )
                                    },
                                    |project| {
                                        Token::new(
                                            Symbol::Word(format!(
                                                "{}\\{}",
                                                project.prefix(),
                                                site.path().as_str()
                                            )),
                                            key.position().clone(),
                                        )
                                    },
                                )),
                                Rc::new(Token::new(Symbol::DoubleQuote, key.position().clone())),
                            ]),
                        ));
                    }
                    "__LINE__" => {
                        return Some((
                            key.clone(),
                            Definition::Value(vec![Rc::new(Token::new(
                                Symbol::Digit(site.start().1 .0),
                                key.position().clone(),
                            ))]),
                        ));
                    }
                    _ => unreachable!(),
                }
            }
        }
        let ret = self.get_readonly(&ident);
        if let Some((_, Definition::Function(body))) = &ret {
            if key.position().path() != body.position().path() {
                return ret;
            }
            // starts before the definition
            if key.position().start().1 .0 < body.position().start().1 .0 {
                return ret;
            }
            // starts after the definition
            if key.position().start().1 .0 > body.position().end().1 .0 {
                return ret;
            }
            // the usage is within the definition, so we can't use it
            return None;
        }
        ret
    }

    pub fn get_readonly(&self, key: &str) -> Option<(Rc<Token>, Definition)> {
        self.stack
            .last()
            .as_ref()
            .and_then(|i| i.1.get(key))
            .or_else(|| self.global.get(key))
            .cloned()
    }

    #[cfg(test)]
    pub fn get_test(&self, key: &str) -> Option<&(Rc<Token>, Definition)> {
        self.stack
            .last()
            .as_ref()
            .and_then(|i| i.1.get(key))
            .or_else(|| self.global.get(key))
    }

    pub fn insert(
        &mut self,
        key: &str,
        value: (Rc<Token>, Definition),
    ) -> Option<(Rc<Token>, Definition)> {
        if let Some(stack) = self.stack.last_mut() {
            stack.1.insert(Arc::from(key), value)
        } else {
            self.global.insert(Arc::from(key), value)
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<(Rc<Token>, Definition)> {
        if let Some(scope) = self.stack.last_mut() {
            scope.1.remove(key)
        } else {
            self.global.remove(key)
        }
    }

    pub fn push(&mut self, name: &str, args: InnerDefines) {
        self.stack.push((Arc::from(name), args));
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub const fn stack(&self) -> &Vec<(Arc<str>, InnerDefines)> {
        &self.stack
    }

    pub const fn global(&self) -> &InnerDefines {
        &self.global
    }

    pub fn similar_function(&self, search: &str, args: Option<usize>) -> Vec<&Arc<str>> {
        let mut similar = self
            .global
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Function(func) = def else {
                    return false;
                };
                args.map_or(true, |args| func.args().len() == args)
            })
            .map(|(name, _)| (name, levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }

    pub fn similar_values(&self, search: &str) -> Vec<&Arc<str>> {
        let mut similar = self
            .global
            .iter()
            .filter(|(_, (_, def))| {
                let Definition::Value(_) = def else {
                    return false;
                };
                true
            })
            .map(|(name, _)| (name, levenshtein(name, search)))
            .collect::<Vec<_>>();
        similar.sort_by_key(|(_, v)| *v);
        similar.retain(|s| s.1 <= 3);
        if similar.len() > 3 {
            similar.truncate(3);
        }
        similar.into_iter().map(|(n, _)| n).collect::<Vec<_>>()
    }
}
