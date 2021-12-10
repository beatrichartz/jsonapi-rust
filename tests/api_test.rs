//! The purpose of these tests is to validate compliance with the JSONAPI
//! specification and to ensure that this crate reads documents properly
extern crate env_logger;
extern crate jsonapi;
extern crate serde_json;

use jsonapi::api::*;

mod helper;
use crate::helper::read_json_file;

#[test]
fn it_works_with_ids_present() {
    let _ = env_logger::try_init();
    let resource = Resource {
        _type: "test".into(),
        id: Some("123".into()),
        attributes: ResourceAttributes::new(),
        relationships: Some(Relationships::new()),
        links: None,
        meta: Some(Meta::new()),
    };

    assert_eq!(resource.id, Some("123".into()));

    let serialized = serde_json::to_string(&resource).unwrap();
    let deserialized: Resource = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.id, resource.id);

    let jsonapidocument = JsonApiDocument::Data(DocumentData {
        data: Some(PrimaryData::None),
        ..Default::default()
    });

    assert_eq!(jsonapidocument.is_valid(), true);
}

#[test]
fn it_works_with_ids_absent() {
    let _ = env_logger::try_init();
    let resource = Resource {
        _type: "test".into(),
        id: None,
        attributes: ResourceAttributes::new(),
        relationships: Some(Relationships::new()),
        links: None,
        meta: Some(Meta::new()),
    };

    assert_eq!(resource.id, None);

    let serialized = serde_json::to_string(&resource).unwrap();
    let deserialized: Resource = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.id, resource.id);

    let jsonapidocument = JsonApiDocument::Data(DocumentData {
        data: Some(PrimaryData::None),
        ..Default::default()
    });

    assert_eq!(jsonapidocument.is_valid(), true);
}

#[test]
fn jsonapi_document_can_be_valid() {
    let _ = env_logger::try_init();
    let resource = Resource {
        _type: "test".into(),
        id: Some("123".into()),
        attributes: ResourceAttributes::new(),
        relationships: Some(Relationships::new()),
        links: None,
        meta: Some(Meta::new()),
    };

    let jsonapi_document_with_data = JsonApiDocument::Data(DocumentData {
        data: Some(PrimaryData::Single(Box::new(resource))),
        ..Default::default()
    });

    assert_eq!(jsonapi_document_with_data.is_valid(), true);
}

#[test]
fn jsonapi_document_invalid_errors() {
    let _ = env_logger::try_init();

    let included_resource = Resource {
        _type: "test".into(),
        id: Some("123".into()),
        attributes: ResourceAttributes::new(),
        relationships: Some(Relationships::new()),
        links: None,
        meta: Some(Meta::new()),
    };

    let no_content_document = JsonApiDocument::Data(DocumentData {
        data: None,
        ..Default::default()
    });

    match no_content_document.validate() {
        None => assert!(false),
        Some(errors) => {
            assert!(errors.contains(&DocumentValidationError::MissingContent));
        }
    }

    let null_data_content_document = JsonApiDocument::Data(DocumentData {
        data: Some(PrimaryData::None),
        ..Default::default()
    });

    match null_data_content_document.validate() {
        None => assert!(true),
        Some(_) => assert!(false),
    }

    let included_without_data_document = JsonApiDocument::Data(DocumentData {
        included: Some(vec![included_resource]),
        ..Default::default()
    });

    match included_without_data_document.validate() {
        None => assert!(false),
        Some(errors) => {
            assert!(errors.contains(&DocumentValidationError::IncludedWithoutData,));
        }
    }
}

#[test]
fn error_from_json_string() {
    let _ = env_logger::try_init();

    let serialized = r#"
        {"id":"1", "links" : {}, "status" : "unknown", "code" : "code1", "title" : "error-title", "detail": "error-detail"}
        "#;
    let error: Result<JsonApiError, serde_json::Error> = serde_json::from_str(serialized);
    assert_eq!(error.is_ok(), true);
    match error {
        Ok(jsonapierror) => match jsonapierror.id {
            Some(id) => assert_eq!(id, "1"),
            None => assert!(false),
        },
        Err(_) => assert!(false),
    }
}

#[test]
fn single_resource_from_json_string() {
    let _ = env_logger::try_init();
    let serialized =
        r#"{ "id" :"1", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {} }"#;
    let data: Result<Resource, serde_json::Error> = serde_json::from_str(serialized);
    assert_eq!(data.is_ok(), true);
}

#[test]
fn multiple_resource_from_json_string() {
    let _ = env_logger::try_init();
    let serialized = r#"[
            { "id" :"1", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {} },
            { "id" :"2", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {} },
            { "id" :"3", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {} }
        ]"#;
    let data: Result<Resources, serde_json::Error> = serde_json::from_str(serialized);
    assert_eq!(data.is_ok(), true);
}

#[test]
fn no_data_document_from_json_string() {
    let _ = env_logger::try_init();
    let serialized = r#"{
            "data" : null
        }"#;
    let data: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(serialized);
    assert_eq!(data.is_ok(), true);
}

#[test]
fn single_data_document_from_json_string() {
    let _ = env_logger::try_init();
    let serialized = r#"{
            "data" : {
                "id" :"1", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {}
            }
        }"#;
    let data: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(serialized);
    assert_eq!(data.is_ok(), true);
}

#[test]
fn multiple_data_document_from_json_string() {
    let _ = env_logger::try_init();
    let serialized = r#"{
            "data" : [
                { "id" :"1", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {} },
                { "id" :"2", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {} },
                { "id" :"3", "type" : "post", "attributes" : {}, "relationships" : {}, "links" : {} }
            ]
        }"#;
    let data: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(serialized);
    assert_eq!(data.is_ok(), true);
}

#[test]
fn api_document_from_json_file() {
    let _ = env_logger::try_init();

    let s = crate::read_json_file("data/results.json");
    let data: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(&s);

    match data {
        Ok(res) => match res {
            JsonApiDocument::Error(_x) => assert!(false),
            JsonApiDocument::Data(x) => match x.data {
                Some(PrimaryData::Multiple(arr)) => {
                    assert_eq!(arr.len(), 1);
                }
                Some(PrimaryData::Single(_)) => {
                    println!(
                        "api_document_from_json_file : Expected one Resource in a vector, \
                                      not a direct Resource"
                    );
                    assert!(false);
                }
                Some(PrimaryData::None) => {
                    println!("api_document_from_json_file : Expected one Resource in a vector");
                    assert!(false);
                }
                None => assert!(false),
            },
        },
        Err(err) => {
            println!("api_document_from_json_file : Error: {:?}", err);
            assert!(false);
        }
    }
}

#[test]
fn api_document_collection_from_json_file() {
    let _ = env_logger::try_init();

    let s = crate::read_json_file("data/collection.json");
    let data: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(&s);

    match data {
        Ok(x) => match x {
            JsonApiDocument::Error(_) => assert!(false),
            JsonApiDocument::Data(res) => {
                match res.data {
                    Some(PrimaryData::Multiple(arr)) => {
                        assert_eq!(arr.len(), 1);
                    }
                    Some(PrimaryData::Single(_)) => {
                        println!(
                            "api_document_collection_from_json_file : Expected one Resource in \
                                      a vector, not a direct Resource"
                        );
                        assert!(false);
                    }
                    Some(PrimaryData::None) => {
                        println!(
                            "api_document_collection_from_json_file : Expected one Resource in \
                                      a vector"
                        );
                        assert!(false);
                    }
                    None => assert!(false),
                }

                match res.included {
                    Some(arr) => {
                        assert_eq!(arr.len(), 3);
                        assert_eq!(arr[0].id, Some("9".into()));
                        assert_eq!(arr[1].id, Some("5".into()));
                        assert_eq!(arr[2].id, Some("12".into()));
                    }
                    None => {
                        println!(
                            "api_document_collection_from_json_file : Expected three Resources \
                                      in 'included' in a vector"
                        );
                        assert!(false);
                    }
                }

                match res.links {
                    Some(links) => {
                        assert_eq!(links.len(), 3);
                    }
                    None => {
                        println!("api_document_collection_from_json_file : expected links");
                        assert!(false);
                    }
                }
            }
        },
        Err(err) => {
            println!("api_document_collection_from_json_file : Error: {:?}", err);
            assert!(false);
        }
    }
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_resource_001() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/resource_001.json");
    let data: Result<Resource, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_resource_002() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/resource_002.json");
    let data: Result<Resource, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_resource_003() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/resource_003.json");
    let data: Result<Resource, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_resource_004() {
    let _ = env_logger::try_init();
    let s = ::read_json_file("data/resource_004.json");
    let data: Result<Resource, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_compound_document() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/compound_document.json");
    let data: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_links_001() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/links_001.json");
    let data: Result<Links, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_links_002() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/links_002.json");
    let data: Result<Links, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

// TODO - naming of this test and the test file should be more clear
#[test]
fn can_deserialize_jsonapi_example_jsonapi_info() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/jsonapi_info_001.json");
    let data: Result<JsonApiInfo, serde_json::Error> = serde_json::from_str(&s);
    assert!(data.is_ok());
}

#[test]
fn can_get_attribute() {
    let _ = env_logger::try_init();
    let s = crate::read_json_file("data/resource_all_attributes.json");
    let data: Result<Resource, serde_json::Error> = serde_json::from_str(&s);
    match data {
        Err(_) => assert!(false),
        Ok(res) => {
            match res.get_attribute("likes") {
                None => assert!(false),
                Some(val) => match val.as_i64() {
                    None => assert!(false),
                    Some(num) => {
                        let x: i64 = 250;
                        assert_eq!(num, x);
                    }
                },
            }

            match res.get_attribute("title") {
                None => assert!(false),
                Some(val) => match val.as_str() {
                    None => assert!(false),
                    Some(s) => {
                        assert_eq!(s, "Rails is Omakase");
                    }
                },
            }

            match res.get_attribute("published") {
                None => assert!(false),
                Some(val) => match val.as_bool() {
                    None => assert!(false),
                    Some(b) => {
                        assert_eq!(b, true);
                    }
                },
            }

            match res.get_attribute("tags") {
                None => assert!(false),
                Some(val) => match val.as_array() {
                    None => assert!(false),
                    Some(arr) => {
                        assert_eq!(arr[0], "rails");
                        assert_eq!(arr[1], "news");
                    }
                },
            }
        }
    }
}

#[test]
fn can_diff_resource() {
    let _ = env_logger::try_init();
    let s1 = crate::read_json_file("data/resource_post_001.json");
    let s2 = crate::read_json_file("data/resource_post_002.json");

    let data1: Result<Resource, serde_json::Error> = serde_json::from_str(&s1);
    let data2: Result<Resource, serde_json::Error> = serde_json::from_str(&s2);

    match data1 {
        Err(_) => assert!(false),
        Ok(res1) => {
            // So far so good
            match data2 {
                Err(_) => assert!(false),
                Ok(res2) => match res1.diff(res2) {
                    Err(_) => {
                        assert!(false);
                    }
                    Ok(patchset) => {
                        println!("can_diff_resource: PatchSet is {:?}", patchset);
                        assert_eq!(patchset.patches.len(), 5);
                    }
                },
            }
        }
    }
}

#[test]
fn it_omits_empty_document_and_primary_data_keys() {
    let _ = env_logger::try_init();
    let resource = Resource {
        _type: "test".into(),
        id: Some("123".into()),
        attributes: ResourceAttributes::new(),
        ..Default::default()
    };
    let doc = JsonApiDocument::Data(DocumentData {
        data: Some(PrimaryData::Single(Box::new(resource))),
        ..Default::default()
    });

    assert_eq!(
        serde_json::to_string(&doc).unwrap(),
        r#"{"data":{"type":"test","id":"123","attributes":{}}}"#
    );
}

#[test]
fn it_does_not_omit_an_empty_primary_data() {
    let doc = JsonApiDocument::Data(DocumentData {
        data: Some(PrimaryData::None),
        ..Default::default()
    });

    assert_eq!(serde_json::to_string(&doc).unwrap(), r#"{"data":null}"#);
}

#[test]
fn it_omits_empty_error_keys() {
    let error = JsonApiError {
        id: Some("error_id".to_string()),
        ..Default::default()
    };
    let doc = JsonApiDocument::Error(DocumentError {
        errors: vec![error],
        ..Default::default()
    });
    assert_eq!(
        serde_json::to_string(&doc).unwrap(),
        r#"{"errors":[{"id":"error_id"}]}"#
    );
}

#[test]
fn it_allows_for_optional_attributes() {
    let _ = env_logger::try_init();
    let serialized = r#"{
            "data" : {
                "id" :"1", "type" : "post", "relationships" : {}, "links" : {}
            }
        }"#;
    let data: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(serialized);
    assert_eq!(data.is_ok(), true);
}

#[test]
fn it_validates_partialeq_when_compariing_documents() {
    let _ = env_logger::try_init();
    let document1 = r#"
        {
            "data": {
              "type": "posts",
              "id": "1",
              "attributes": {
                "title": "Rails is Omakase"
              },
              "relationships": {
                "author": {
                  "links": {
                    "self": "/posts/1/relationships/author",
                    "related": "/posts/1/author"
                  },
                  "data": {
                      "type": "people",
                      "id": "9"
                  }
                },
                "tags": {
                  "links": {
                    "self": "/posts/1/relationships/tags",
                    "related": "/posts/1/tags"
                  },
                  "data": {
                      "type": "tags",
                      "id": "99"
                  }
                }
              },
              "links": {
                "self": "http://example.com/posts/1"
              }
            }
        }"#;

    let document2 = r#"
        {
            "data": {
              "relationships": {
                "tags": {
                  "data": {
                      "type": "tags",
                      "id": "99"
                  },
                  "links": {
                    "self": "/posts/1/relationships/tags",
                    "related": "/posts/1/tags"
                  }
                },
                "author": {
                  "links": {
                    "self": "/posts/1/relationships/author",
                    "related": "/posts/1/author"
                  },
                  "data": {
                      "type": "people",
                      "id": "9"
                  }
                }
              },
              "links": {
                "self": "http://example.com/posts/1"
              },
              "attributes": {
                "title": "Rails is Omakase"
              },
              "type": "posts",
              "id": "1"
            }
        }"#;
    let doc1: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(document1);
    let doc2: Result<JsonApiDocument, serde_json::Error> = serde_json::from_str(document2);
    assert_eq!(doc1.is_ok(), true);
    assert_eq!(doc2.is_ok(), true);
    assert!(doc1.unwrap() == doc2.unwrap());
}
