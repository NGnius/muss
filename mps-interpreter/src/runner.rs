use std::io::Read;
use std::iter::Iterator;

use super::lang::{MpsLanguageDictionary, MpsLanguageError};
use super::tokens::{MpsTokenReader, MpsTokenizer};
use super::{MpsContext, MpsInterpretor, MpsItem};

pub struct MpsRunnerSettings<T: MpsTokenReader> {
    pub vocabulary: MpsLanguageDictionary,
    pub tokenizer: T,
    pub context: Option<MpsContext>,
}

impl<T: MpsTokenReader> MpsRunnerSettings<T> {
    pub fn default_with_tokenizer(token_reader: T) -> Self {
        let mut vocab = MpsLanguageDictionary::default();
        super::interpretor::standard_vocab(&mut vocab);
        Self {
            vocabulary: vocab,
            tokenizer: token_reader,
            context: None,
        }
    }
}

/// A wrapper around MpsInterpretor which provides a simpler (and more powerful) interface.
pub struct MpsRunner<T: MpsTokenReader> {
    interpretor: MpsInterpretor<T>,
    new_statement: bool,
}

impl<T: MpsTokenReader> MpsRunner<T> {
    pub fn with_settings(settings: MpsRunnerSettings<T>) -> Self {
        let mut interpretor = MpsInterpretor::with_vocab(settings.tokenizer, settings.vocabulary);
        if let Some(ctx) = settings.context {
            interpretor.context(ctx);
        }
        Self {
            interpretor: interpretor,
            new_statement: true,
        }
    }

    pub fn is_new_statement(&self) -> bool {
        self.new_statement
    }
}

impl<R: Read> MpsRunner<MpsTokenizer<R>> {
    pub fn with_stream(stream: R) -> Self {
        let tokenizer = MpsTokenizer::new(stream);
        Self {
            interpretor: MpsInterpretor::with_standard_vocab(tokenizer),
            new_statement: true,
        }
    }
}

impl<T: MpsTokenReader> Iterator for MpsRunner<T> {
    type Item = Result<MpsItem, Box<dyn MpsLanguageError>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = self.interpretor.next();
        self.new_statement = false;
        while item.is_none() && !self.interpretor.is_done() {
            item = self.interpretor.next();
            self.new_statement = true;
        }
        item
    }
}
