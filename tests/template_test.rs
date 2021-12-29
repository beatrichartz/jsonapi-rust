#[macro_use]
extern crate jsonapi;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

use jsonapi::model::*;
use jsonapi::template::*;

mod helper;
use helper::read_json_file;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Book {
    title: String,
    first_chapter: ResourceIdentifier,
    chapters: Vec<ResourceIdentifier>
}
jsonapi_template!(Book; "books"; has one first_chapter; has many chapters);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Chapter {
    id: String,
    title: String,
    ordering: i32,
}
jsonapi_model!(Chapter; "chapters");

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ChapterTemplate {
    title: String,
    ordering: i32,
}
jsonapi_template!(ChapterTemplate; "chapters");

#[test]
fn to_jsonapi_document_and_back() {
    let book = Book {
        title: "The Fellowship of the Ring".into(),
        first_chapter: ResourceIdentifier { id: "1".into(), _type: "chapters".into() },
        chapters: vec![
            ResourceIdentifier { id: "1".into(), _type: "chapters".into() },
            ResourceIdentifier { id: "2".into(), _type: "chapters".into() },
            ResourceIdentifier { id: "3".into(), _type: "chapters".into() }
        ],
    };

    let doc = book.to_jsonapi_document();
    let json = serde_json::to_string(&doc).unwrap();
    let book_doc: DocumentData = serde_json::from_str(&json)
        .expect("Book DocumentData should be created from the book json");
    let book_again = Book::from_jsonapi_document(&book_doc)
        .expect("Book should be generated from the book_doc");

    assert_eq!(book, book_again);
}


#[test]
fn test_template_vec_to_jsonapi_document() {
    let chapters = vec![
        ChapterTemplate {
            title: "The Passing of the Grey Company".into(),
            ordering: 2,
        },
        ChapterTemplate {
            title: "The Muster of Rohan".into(),
            ordering: 3,
        },
    ];

    let doc = template_vec_to_jsonapi_document(chapters);
    assert!(doc.is_valid());
}

#[test]
fn from_jsonapi_document() {
    let json = ::read_json_file("data/book_hobbit_template.json");
    let author_doc: JsonApiDocument = serde_json::from_str(&json)
        .expect("Book DocumentData should be created from the book json");

    match author_doc {
        JsonApiDocument::Error(_) => panic!("Expected no error"),
        JsonApiDocument::Data(doc) => {
            let author = Book::from_jsonapi_document(&doc)
                .expect("Book should be generated from the book document data");

            let doc_again = author.to_jsonapi_document();
            assert!(doc_again.is_valid());
        }
    }
}
