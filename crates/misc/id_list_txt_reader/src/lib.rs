use std::io::BufRead;

#[derive(Debug)]
pub struct IdListTxtReader<R: BufRead> {
    reader: R,
    only_comments: bool,
}

impl<R> IdListTxtReader<R>
where
    R: BufRead,
{
    pub fn into_inner(self) -> R {
        self.reader
    }
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            only_comments: false,
        }
    }
    pub fn only_comments(self, only_comments: bool) -> Self {
        Self {
            only_comments,
            ..self
        }
    }
}

impl<R> Iterator for IdListTxtReader<R>
where
    R: BufRead,
{
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        if self.reader.read_line(&mut line).ok()? > 0 {
            if (line.contains('#') || line.contains("//")) && self.only_comments {
                Some(line.replacen('#', "", 1).replacen("//", "", 1))
            } else {
                Some(line)
            }
        } else {
            None
        }
    }
}
