/// A crappy Uri parser.
/// More forgiving than regular ones so it can accept raw filepaths.
pub(crate) struct Uri<S>(S);

impl<'a> Uri<&'a str> {
    pub fn new(s: &'a str) -> Self {
        Self(s)
    }

    pub fn scheme(&self) -> Option<&'a str> {
        self.0.find("//").map(|end| self.0.get(0..end).unwrap())
    }

    pub fn path(&self) -> &'a str {
        match self.0.find("//") {
            Some(end) => {
                // proper URI
                if let Some(query_start) = self.0.find('?') {
                    self.0.get(end + 2..query_start).unwrap()
                } else if let Some(frag_start) = self.0.find('#') {
                    self.0.get(end + 2..frag_start).unwrap()
                } else {
                    self.0.get(end + 2..).unwrap()
                }
            }
            None => self.0,
        }
    }

    pub fn without_scheme(&self) -> &'a str {
        match self.0.find("//") {
            Some(end) => self.0.get(end + 2..).unwrap(),
            None => self.0,
        }
    }

    #[allow(dead_code)]
    pub fn uri(&self) -> &'a str {
        self.0
    }
}
