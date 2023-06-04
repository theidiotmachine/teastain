use std::{path::Path, collections::HashMap, env};

use document::{Document, DocumentElement};
use odt::parse;


mod odt;
mod document;

fn loc_string(
    chapter_title: &String, 
    chapter_number: i32, 
    section_number: i32, 
    paragraph_number: i32,
    number_of_chapters: i32
) -> String {
    let mut out = String::from("");
    if chapter_title != "" {
        out += chapter_title;
        out += ", ";
    } else if number_of_chapters > 1 {
        out += "Chapter ";
        out += &chapter_number.to_string();
        out += ", ";
    }
    out += "Section ";
    out += &section_number.to_string();
    out += ", ";
    out += "Paragraph ";
    out += &paragraph_number.to_string();

    out
}

fn sentence_string(words: &Vec<String>) -> String {
    words.concat()
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
enum Action{
    Filtering,
    Beginning,
    WeakImmediacy,
    PotentialAdverb,
    Adverb,
    Contraction,
    SubjectiveAdjective,
}

struct ActionTrigger{
    pub action: Action,
    pub trigger: Vec<String>
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialEq)]
enum PartOfSpeech{
    Adjective,
    Adverb,
    Noun,
}

#[derive(Debug)]
struct DictionaryElem{
    pub part_of_speech: PartOfSpeech
}

fn init_dictionary() -> HashMap<String,DictionaryElem> {
    let mut out = HashMap::new();
    
    out.insert("curmudgeonly".to_string(), DictionaryElem{part_of_speech: PartOfSpeech::Adjective});
    out.insert("family".to_string(), DictionaryElem{part_of_speech: PartOfSpeech::Noun});

    out
}

fn init_database() -> HashMap<String, Vec<ActionTrigger>> {
    let mut out: HashMap<String, Vec<ActionTrigger>> = HashMap::new();
    out.insert("could".to_string(), 
        vec![
            ActionTrigger{action: Action::Filtering, trigger: vec!["hear".to_string()]},
            ActionTrigger{action: Action::Filtering, trigger: vec!["see".to_string()]},
            ActionTrigger{action: Action::Filtering, trigger: vec!["taste".to_string()]},
        ]
    );
    out.insert("heard".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("listened".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("looked".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("saw".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("seemed".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("smelt".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("spotted".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("tasted".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);
    out.insert("watched".to_string(), vec![ActionTrigger{action: Action::Filtering, trigger: vec![]}]);

    out.insert("commenced".to_string(), vec![ActionTrigger{action: Action::Beginning, trigger: vec![]}]);
    out.insert("began".to_string(), vec![ActionTrigger{action: Action::Beginning, trigger: vec![]}]);
    out.insert("initiated".to_string(), vec![ActionTrigger{action: Action::Beginning, trigger: vec![]}]);
    out.insert("started".to_string(), vec![ActionTrigger{action: Action::Beginning, trigger: vec![]}]);

    out.insert("immediately".to_string(), vec![ActionTrigger{action: Action::WeakImmediacy, trigger: vec![]}]);
    out.insert("just".to_string(), vec![ActionTrigger{action: Action::WeakImmediacy, trigger: vec![
        "then".to_string()
    ]}]);
    out.insert("suddenly".to_string(), vec![ActionTrigger{action: Action::WeakImmediacy, trigger: vec![]}]);

    out.insert("are".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "not".to_string()
        ]},
    ]);
    out.insert("can".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "not".to_string()
        ]},
    ]);
    out.insert("do".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "not".to_string()
        ]},
    ]);
    out.insert("has".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "not".to_string()
        ]},
    ]);
    out.insert("have".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "not".to_string()
        ]},
    ]);
    out.insert("i".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "am".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "have".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "will".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "would".to_string()
        ]},
    ]);
    out.insert("it".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "is".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "will".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "would".to_string()
        ]}
    ]);
    out.insert("there".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "is".to_string()
        ]},
    ]);
    out.insert("they".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "are".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "have".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "will".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "would".to_string()
        ]},
    ]);
    out.insert("you".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "are".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "have".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "will".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "would".to_string()
        ]},
    ]);
    out.insert("we".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "are".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "have".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "will".to_string()
        ]},
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "would".to_string()
        ]},
    ]);
    out.insert("will".to_string(), vec![
        ActionTrigger{action: Action::Contraction, trigger: vec![
            "not".to_string()
        ]},
    ]);

    out.insert("amazing".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("beautiful".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("bad".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("excellent".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("fantastic".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("good".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("great".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("lovely".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);
    out.insert("wonderful".to_string(), vec![ActionTrigger{action: Action::SubjectiveAdjective, trigger: vec![]}]);

    out
}

struct ActionTriggerWithHistory {
    action_trigger: ActionTrigger,
    history: Vec<String>
}

fn process_action(
    action: Action,
    sentence: &Vec<String>,
    history: &Vec<String>,
    chapter_title: &String, 
    chapter_number: i32, 
    section_number: i32, 
    paragraph_number: i32,
    number_of_chapters: i32
) {
    println!();
    match action{
        Action::Filtering => {
            println!("{}, possible filtering ({:?})\n{:?}", loc_string(
                    &chapter_title, chapter_number, section_number, paragraph_number,
                    number_of_chapters
                ), 
                sentence_string(history), sentence_string(sentence));
        }
        Action::Beginning => {
            println!("{}, beginning ({:?})\n{:?}", loc_string(
                    &chapter_title, chapter_number, section_number, paragraph_number,
                    number_of_chapters
                ), 
                sentence_string(history), sentence_string(sentence));
        }
        Action::WeakImmediacy=> {
            println!("{}, weak immediacy ({:?})\n{:?}", loc_string(
                    &chapter_title, chapter_number, section_number, paragraph_number,
                    number_of_chapters
                ), 
                sentence_string(history), sentence_string(sentence));
        }
        Action::PotentialAdverb => {
            println!("{}, potential adverb ({:?})\n{:?}", loc_string(
                    &chapter_title, chapter_number, section_number, paragraph_number,
                    number_of_chapters
                ), 
                sentence_string(history), sentence_string(sentence));
        }
        Action::Adverb => {
            println!("{}, adverb ({:?})\n{:?}", loc_string(
                    &chapter_title, chapter_number, section_number, paragraph_number,
                    number_of_chapters
                ), 
                sentence_string(history), sentence_string(sentence));
        }
        Action::Contraction => {
            println!("{}, missed contraction ({:?})\n{:?}", loc_string(
                    &chapter_title, chapter_number, section_number, paragraph_number,
                    number_of_chapters
                ), 
                sentence_string(history), sentence_string(sentence));
        }
        Action::SubjectiveAdjective => {
            println!("{}, subjective adjective ({:?})\n{:?}", loc_string(
                    &chapter_title, chapter_number, section_number, paragraph_number,
                    number_of_chapters
                ), 
                sentence_string(history), sentence_string(sentence));
        }
    }
    
}

fn skip_action_in_direct_speech(action: Action) -> bool {
    action != Action::Contraction
}

fn add_history_to_state(
    word: &String, 
    state: &mut HashMap<String, Vec<ActionTriggerWithHistory>>, 
){
    for state_elem in state {
        for a_t_w_h in state_elem.1 {
            a_t_w_h.history.push(word.to_string());
        }
    }
}

fn process_sentence(
    sentence: &Vec<String>, 
    database: &HashMap<String, Vec<ActionTrigger>>, 
    dictionary: &HashMap<String, DictionaryElem>,
    state: &mut HashMap<String, Vec<ActionTriggerWithHistory>>, 
    chapter_title: &String, 
    chapter_number: i32, 
    section_number: i32, 
    paragraph_number: i32, 
    document: &Document,
    direct_speech: bool
) {
    for word in sentence {
        if word.trim() == "" {
            add_history_to_state(word, state);
            continue;
        }

        let lowercase_word = word.to_lowercase();

        let database_entry = database.get(word);
        match database_entry {
            Some(action_triggers) => {
                for action_trigger in action_triggers {
                    if direct_speech && skip_action_in_direct_speech(action_trigger.action) {
                        continue;
                    }
                    let trigger = &action_trigger.trigger;
                    if state.contains_key(&lowercase_word) {
                        let v = state.get_mut(&lowercase_word).unwrap();
                        v.push(ActionTriggerWithHistory{
                            action_trigger: ActionTrigger{ 
                                action: action_trigger.action, 
                                trigger: trigger.to_vec()
                            },
                            history : vec![]
                        })
                    } else {
                        state.insert(lowercase_word.to_string(), vec![
                            ActionTriggerWithHistory{
                                action_trigger: ActionTrigger{ 
                                    action: action_trigger.action, 
                                    trigger: trigger.to_vec()
                                },
                            history : vec![]
                        }]);
                    }
                }
            }
            None => ()
        }

        //state check
        if let Some(state_entry) = state.get(&lowercase_word) {
            let mut new_state: HashMap<String, Vec<ActionTriggerWithHistory>> = HashMap::new();
            for a_t_w_h in state_entry {
                let mut history = a_t_w_h.history.clone();
                history.push(word.to_string());
                if a_t_w_h.action_trigger.trigger.len() == 0 {    
                    process_action(a_t_w_h.action_trigger.action, sentence, &history,
                        chapter_title, chapter_number, section_number, paragraph_number, document.number_of_chapters);
                } else {
                    let bits = a_t_w_h.action_trigger.trigger.split_first().unwrap();
                    if new_state.contains_key(bits.0) {
                        let v: &mut Vec<ActionTriggerWithHistory> = new_state.get_mut(bits.0).unwrap();
                        v.push(ActionTriggerWithHistory{
                            action_trigger: ActionTrigger{ 
                                action: a_t_w_h.action_trigger.action, 
                                trigger: bits.1.to_vec()
                            },
                            history: history
                        })
                    } else {
                        new_state.insert(bits.0.to_string(), vec![
                            ActionTriggerWithHistory{
                                action_trigger: ActionTrigger{ 
                                    action: a_t_w_h.action_trigger.action, 
                                    trigger: bits.1.to_vec()
                                },
                                history: history
                        }]);
                    }
                }
            }
            *state = new_state;
        } else {
            state.clear();
        }

        if !direct_speech && word.ends_with("ly") {
            let o_dictionary_elem = dictionary.get(word);
            match o_dictionary_elem {
                Some(dictionary_elem) => {
                    if dictionary_elem.part_of_speech == PartOfSpeech::Adverb {
                        process_action(Action::Adverb, sentence, &vec![word.to_string()],
                    chapter_title, chapter_number, section_number, paragraph_number, document.number_of_chapters);
                    }
                }
                None => process_action(Action::PotentialAdverb, sentence, &vec![word.to_string()],
                    chapter_title, chapter_number, section_number, paragraph_number, document.number_of_chapters)
            }
            
        }
    }
}

fn score(document: &Document) {
    let mut chapter_number= 1;
    let mut section_number = 1;
    let mut paragraph_number = 1;
    let mut chapter_title = String::from("");
    let database = init_database();
    let dictionary = init_dictionary();

    let elements = &document.elements;
    let mut state: HashMap<String, Vec<ActionTriggerWithHistory>> = HashMap::new();
    for element in elements {
        match element {
            DocumentElement::ChapterEnd => {
                chapter_number += 1;
                section_number = 1;
                paragraph_number = 1;
            }
            DocumentElement::SectionEnd => {
                section_number += 1;
                paragraph_number = 1;
            }
            DocumentElement::ParagraphEnd => {
                paragraph_number += 1;
            }
            DocumentElement::ChapterName(_) => (),
            DocumentElement::ChapterTitle(this_chapter_title) => {
                chapter_title = this_chapter_title.clone();
            },
            DocumentElement::DirectSpeech(sentence) => 
                process_sentence(sentence, &database, &dictionary, &mut state, &chapter_title, chapter_number, section_number, paragraph_number, document, true),
            DocumentElement::Sentence(sentence) =>
                process_sentence(sentence, &database, &dictionary, &mut state, &chapter_title, chapter_number, section_number, paragraph_number, document, false),
        }
    }
}



fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Must supply a file");
        return;
    } else {
        let file_name = &args[1];
        let path = Path::new(file_name);
        let document = parse(&path);
        score(&document);
    }
}
