extern crate xml5ever;
extern crate tendril;

use std::{env, iter};
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

use tendril::{ByteTendril, ReadExt};

use xml5ever::rcdom::{NodeEnum, RcDom};

fn main() {
    let mut args = env::args();
    let db_path = args.next().expect("missing XML dump argument");
    let dump_path = args.next().expect("missing target directory argument");
    let mut input = ByteTendril::new();
    File::open(db_path)
        .expect("could not open XML dump")
        .read_to_tendril(&mut input)
        .expect("could not read XML dump to tendril");
    let input = input.try_reinterpret().expect("failed to reinterpret XML dump as UTF-8");
    let xml: RcDom = xml5ever::parse_xml(iter::once(input), Default::default());
    let doc = xml.document.borrow();
    let root = doc.children.iter().next().expect("XML document has no root");
    let dump_dir = Path::new(&dump_path);
    for elt in &root.borrow().children {
        match elt.borrow().node {
            NodeEnum::Element(ref qual_name, _) => {
                if *qual_name.local == *"page" {
                    let mut title = None;
                    let mut text = None;
                    for page_info in &elt.borrow().children {
                        if let NodeEnum::Element(ref qual_name, _) = page_info.borrow().node {
                            if *qual_name.local == *"title" {
                                for title_info in &page_info.borrow().children {
                                    if let NodeEnum::Text(ref title_text) = title_info.borrow().node {
                                        title = Some(title_text.as_ref().to_owned());
                                    }
                                }
                            } else if *qual_name.local == *"revision" {
                                for revision_info in &page_info.borrow().children {
                                    if let NodeEnum::Element(ref qual_name, _) = revision_info.borrow().node {
                                        if *qual_name.local == *"text" {
                                            for text_info in &revision_info.borrow().children {
                                                if let NodeEnum::Text(ref text_text) = text_info.borrow().node {
                                                    text = Some(text_text.as_ref().to_owned());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    //println!("");
                    let full_title = title.expect("missing title");
                    //println!("= {} =", full_title);
                    //println!("");
                    let text = text.expect(&format!("missing text for page {:?}", full_title));
                    //println!("{}", text);
                    let namespace;
                    let title;
                    let name_parts: Vec<_> = full_title.splitn(2, ':').collect();
                    if name_parts.len() == 1 {
                        namespace = "Main".to_owned();
                        title = full_title.clone();
                    } else {
                        assert_eq!(name_parts.len(), 2);
                        namespace = name_parts[0].to_owned();
                        title = name_parts[1].to_owned();
                    }
                    if title.contains('/') {
                        println!("\nSkipping subpage {}:{}\n\n{}", namespace, title, text);
                        continue;
                    }
                    let namespace_dir = dump_dir.join(&namespace);
                    fs::create_dir_all(&namespace_dir).expect(&format!("could not create directory for {:?} namespace", namespace));
                    fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(namespace_dir.join(format!("{}.wiki", title)))
                        .expect(&format!("could not open {}/{}.wiki", namespace, title))
                        .write_all(format!("{}\n", text).as_bytes())
                        .expect(&format!("could not write to {}/{}.wiki", namespace, title));
                } else {
                    println!("Found non-page node {:?}", qual_name);
                }
            }
            _ => ()
        }
    }
}
