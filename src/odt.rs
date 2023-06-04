use std::{io::Read, fs::File, path::Path, collections::HashMap};

use quick_xml::{Reader, events::{Event, BytesStart}, name::QName};
use zip::ZipArchive;
use unicode_segmentation::UnicodeSegmentation;

use crate::document::Document;
use crate::document::DocumentElement;

#[derive(Debug)]
#[derive(PartialEq)]
enum TextAlign{
    Left,
    Centre
}

#[derive(Debug)]
enum DocumentElementODT{
    ParagraphStyle(TextAlign),
    ParagraphEnd,
    ChapterEnd,
    Text(String),
}


#[derive(Debug)]
pub struct ParagraphStyleODT{
    pub begins_in_page_break: bool,
    pub center_aligned: bool,
}

#[derive(Debug)] 
pub struct TextStyleODT{

}

fn paragraph_is_section_break(line: &String) -> bool {
    let trimmed_line = line.trim();
    trimmed_line == "* * *" || trimmed_line == "#"
}

fn process_text_p_odt(
    e: BytesStart,
    paragraph_styles: &HashMap<String, ParagraphStyleODT>,
    document: &mut Vec<DocumentElementODT>
) {
    for a in e.attributes() {
        let att = a.unwrap();
        let att_name = att.key;
        if att_name == QName(b"text:style-name") {
            let style_name = att.unescape_value().unwrap().into_owned().to_string();
            let paragraph_style = paragraph_styles.get(&style_name).unwrap();
            if paragraph_style.begins_in_page_break {
                document.push(DocumentElementODT::ChapterEnd);
            }
            if paragraph_style.center_aligned {
                document.push(DocumentElementODT::ParagraphStyle(TextAlign::Centre));
            } else {
                document.push(DocumentElementODT::ParagraphStyle(TextAlign::Left));
            }
        } 
    }
}

fn parse_odt(path: &Path) -> Vec<DocumentElementODT> {
    let file = File::open(path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    let mut xml_data = String::new();
    let content_name = "content.xml";
    
    let mut document: Vec<DocumentElementODT> = vec![];

    let mut paragraph_styles: HashMap<String, ParagraphStyleODT> = HashMap::new();
    let mut text_styles = HashMap::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i).unwrap();
        if c_file.name() == content_name {
            c_file.read_to_string(&mut xml_data).unwrap();
            break;
        }
    }

    let mut reader = Reader::from_str(&xml_data);

    let mut buf = Vec::new();
    let mut current_style = String::from("");

    if xml_data.len() > 0 {
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,

                Err(e) => panic!(
                    "Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ),

                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap().into_owned();
                    //we seem to get stray '\n's sometimes. We can ignore them
                    //because ODT tells there are user-inserted paragraph ends 
                    //by inserting text:p s
                    if text != "\n" {
                        document.push(DocumentElementODT::Text(text));
                    }
                }
                    
                Ok(Event::Empty(e)) => {
                    match e.name().as_ref() {
                        b"style:paragraph-properties" => {
                            for a in e.attributes() {
                                let att = a.unwrap();
                                let att_name = att.key;
                                if att_name == QName(b"fo:text-align") {
                                    let align = att.unescape_value().unwrap();
                                    if align == "center" {
                                        paragraph_styles.get_mut(&current_style).unwrap().center_aligned = true;
                                    }
                                } else if att_name == QName(b"fo:break-before") {
                                    let break_before = att.unescape_value().unwrap();
                                    if break_before == "page" {
                                        paragraph_styles.get_mut(&current_style).unwrap().begins_in_page_break = true;
                                    }
                                }
                            }
                        },
                        b"text:p" => process_text_p_odt(e, &paragraph_styles, &mut document),
                        _ => {},
                    }
                }

                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"style:style" => {
                            let mut style_name: String = String::from("");
                            let mut style_family = String::from("");
                            for a in e.attributes() {
                                let att = a.unwrap();
                                let att_name = att.key;
                                if att_name == QName(b"style:name") {
                                    style_name = att.unescape_value().unwrap().into_owned().to_string();
                                } else if att_name == QName(b"style:family") {
                                    style_family = att.unescape_value().unwrap().into_owned().to_string();
                                }
                            }
                            current_style = style_name.clone();
                            if style_family == "paragraph" {
                                paragraph_styles.insert(style_name, ParagraphStyleODT{
                                    begins_in_page_break: false, 
                                    center_aligned: false
                                });
                            } else if style_family == "text" {
                                text_styles.insert(style_name, TextStyleODT{});
                            }
                        },
                        b"text:p" => process_text_p_odt(e, &paragraph_styles, &mut document),
                        _ => {},
                    }
                }

                Ok(Event::End(e)) => {
                    match e.name().as_ref() {
                        b"text:p" => document.push(DocumentElementODT::ParagraphEnd),
                        _ => {}
                    }
                }

                _ => (),
            }
        }
    }

    document
}

fn parse_paragraph(paragraph: &String) -> Vec<DocumentElement>{
    let words = paragraph.split_word_bounds();
    let mut out = vec![];

    let mut sentence: Vec<String> = vec![];
    let mut direct_speech = false;
    for word in words {
        //skip leading space
        if word.trim() == "" && sentence.len() == 0 {
            continue;
        }
        if word == "‘" {
            direct_speech = true;
        } else if word == "’" && direct_speech {
            out.push(DocumentElement::DirectSpeech(sentence));
            direct_speech = false;
            sentence = vec![];
        } else if (word == "." || word == "!" || word == "?" || word == "…") && !direct_speech {
            sentence.push(word.to_string());
            out.push(DocumentElement::Sentence(sentence));
            sentence = vec![];
        } else {
            sentence.push(word.to_string());
        }
    }
    if sentence.len() > 0 {
        out.push(DocumentElement::Sentence(sentence));
    }
    out.push(DocumentElement::ParagraphEnd);
    out
}

pub fn parse(path: &Path) -> Document {
    let doc_in = parse_odt(&path);
    //let doc_in: &Vec<DocumentElementODT>
    let mut doc_out: Vec<DocumentElement> = vec![];
    let mut number_of_chapters = 1;

    let mut paragraph_text = String::from("");

    let mut after_page_break = true;
    let mut after_chapter_title = false;
    let mut chapter_title = false;
    let mut chapter_name = false;

    for element in doc_in {
        match element {
            DocumentElementODT::ParagraphStyle(align) => {
                if after_page_break && align == TextAlign::Centre {
                    chapter_title = true;
                } else if after_chapter_title && align == TextAlign::Centre {
                    chapter_name = true;
                    after_chapter_title = false;
                }
            },

            DocumentElementODT::ParagraphEnd => {
                if paragraph_is_section_break(&paragraph_text) {
                    doc_out.push(DocumentElement::SectionEnd);   
                    chapter_title = false;
                    after_chapter_title = false;
                    chapter_name = false;
                } else if chapter_title {
                    doc_out.push(DocumentElement::ChapterTitle(paragraph_text));   
                    after_chapter_title = true; 
                    chapter_title = false;
                } else if chapter_name {
                    doc_out.push(DocumentElement::ChapterName(paragraph_text));   
                    chapter_name = false;
                } else {
                    let mut sentences = parse_paragraph(&paragraph_text);
                    doc_out.append(&mut sentences);
                }
                
                after_page_break = false;
                paragraph_text = String::from("");
            }

            DocumentElementODT::ChapterEnd => {
                after_page_break = true;
                doc_out.push(DocumentElement::ChapterEnd);
                number_of_chapters += 1;
            }

            DocumentElementODT::Text(this_text) => {
                paragraph_text = paragraph_text + &this_text;
            }
        }
    }

    return Document{ elements: doc_out, number_of_chapters: number_of_chapters };
}