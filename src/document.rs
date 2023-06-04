#[derive(Debug)]
pub enum DocumentElement{
    ChapterTitle(String),
    ChapterName(String),
    DirectSpeech(Vec<String>),
    Sentence(Vec<String>),
    ParagraphEnd,
    ChapterEnd,
    SectionEnd,
}

pub struct Document{
    pub elements: Vec<DocumentElement>,
    pub number_of_chapters: i32,
}
