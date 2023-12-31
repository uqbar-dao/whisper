cargo_component_bindings::generate!();

use bindings::component::uq_process::types::*;
use bindings::{
    get_payload, send_requests, send_request, send_response, send_and_await_response, print_to_terminal, receive, Guest,
};
use serde::{Deserialize};
use serde_json::json;
use std::collections::HashMap;

#[allow(dead_code)]
mod process_lib;

struct Component;

const WHISPER_PAGE: &str = include_str!("index.html");
const WHISPER_JS: &str = include_str!("index.js");
const WHISPER_JS2: &str = include_str!("index2.js");
const WHISPER_VIZ: &str = include_str!("viz.js");
const WHISPER_CSS: &str = include_str!("index.css");

#[derive(Deserialize, Debug)]
struct AudioForm {
    audio: String,
}

fn send_http_response(status: u16, headers: HashMap<String, String>, payload_bytes: Vec<u8>) {
    send_response(
        &Response {
            inherit: false,
            ipc: serde_json::json!({
                "status": status,
                "headers": headers,
            })
            .to_string()
            .as_bytes()
            .to_vec(),
            metadata: None,
        },
        Some(&Payload {
            mime: Some("application/octet-stream".to_string()),
            bytes: payload_bytes,
        }),
    )
}

fn http_bind(bindings_address: Address, path: &str) -> (Address, Request, Option<Context>, Option<Payload>) {
    (
        bindings_address,
        Request {
            inherit: false,
            expects_response: None,
            ipc: json!({
                "BindPath": {
                    "path": path,
                    "authenticated": false, // TODO
                    "local_only": false
                }
            })
            .to_string()
            .as_bytes()
            .to_vec(),
            metadata: None,
        },
        None,
        None,
    )
}

impl Guest for Component {
    fn init(our: Address) {
        print_to_terminal(0, "whisper app: start");

        // 1. http bindings
        let bindings_address = Address {
            node: our.node.clone(),
            process: ProcessId::from_str("http_server:sys:uqbar").unwrap(),
        };

        send_requests(&[
            http_bind(bindings_address.clone(), "/"),
            http_bind(bindings_address.clone(), "/audio"),
            http_bind(bindings_address.clone(), "/viz.js"),
            http_bind(bindings_address.clone(), "/index.js"),
            http_bind(bindings_address.clone(), "/index2.js"),
            http_bind(bindings_address.clone(), "/index.css"),
        ]);

        loop {
            let Ok((source, message)) = receive() else {
                print_to_terminal(0, "whisper app: got network error");
                continue;
            };
            print_to_terminal(0, "whisper app: got message");
            let Message::Request(request) = message else {
                print_to_terminal(0, "whisper app: got unexpected Response");
                continue;
            };
            print_to_terminal(0, "whisper app: got request");

            if source.process.to_string() == "http_server:sys:uqbar" {
                print_to_terminal(0, "whisper app: got http request");
                let Ok(json) = serde_json::from_slice::<serde_json::Value>(&request.ipc) else {
                    print_to_terminal(0, "whisper app: got invalid json");
                    continue;
                };
                print_to_terminal(0, "whisper app: got http request");

                let mut default_headers = HashMap::new();
                default_headers.insert("Content-Type".to_string(), "text/html".to_string());

                let path = json["path"].as_str().unwrap_or("");
                let method = json["method"].as_str().unwrap_or("");

                match path {
                    "/" => {
                        print_to_terminal(0, "whisper app: sending homepage");
                        send_http_response(
                            200,
                            {
                                let mut heds = default_headers.clone();
                                // NOTE: you need these headers to enable cross-origin isolation which lets us
                                // use some frontend features, mainly SharedArrayBuffer for audio processing
                                heds.insert("Cross-Origin-Embedder-Policy".to_string(), "require-corp".to_string());
                                heds.insert("Cross-Origin-Opener-Policy".to_string(), "same-origin".to_string());
                                heds
                            },
                            // "audio homepage".as_bytes().to_vec(),
                            WHISPER_PAGE
                                .to_string()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    "/index.js" => {
                        print_to_terminal(0, "whisper app: sending homepage");
                        send_http_response(
                            200,
                            {
                                let mut heds = default_headers.clone();
                                heds.insert("Content-Type".to_string(), "application/javascript".to_string());
                                heds
                            },
                            // "audio homepage".as_bytes().to_vec(),
                            WHISPER_JS
                                .to_string()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    "/index2.js" => {
                        print_to_terminal(0, "whisper app: sending homepage");
                        send_http_response(
                            200,
                            {
                                let mut heds = default_headers.clone();
                                heds.insert("Content-Type".to_string(), "application/javascript".to_string());
                                heds
                            },
                            // "audio homepage".as_bytes().to_vec(),
                            WHISPER_JS2
                                .to_string()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    "/index.css" => {
                        print_to_terminal(0, "whisper app: sending homepage");
                        send_http_response(
                            200,
                            {
                                let mut heds = default_headers.clone();
                                heds.insert("Content-Type".to_string(), "text/css".to_string());
                                heds
                            },
                            // "audio homepage".as_bytes().to_vec(),
                            WHISPER_CSS
                                .to_string()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    "/viz.js" => {
                        print_to_terminal(0, "whisper app: sending homepage");
                        send_http_response(
                            200,
                            {
                                let mut heds = default_headers.clone();
                                heds.insert("Content-Type".to_string(), "application/javascript".to_string());
                                heds
                            },
                            // "audio homepage".as_bytes().to_vec(),
                            WHISPER_VIZ
                                .to_string()
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    "/audio" => {
                        print_to_terminal(0, "whisper app: got audio post");
                        let Some(form) = get_payload() else {
                            print_to_terminal(0, "whisper app: got invalid payload");
                            continue;
                        };
                        match serde_urlencoded::from_bytes::<AudioForm>(&form.bytes) {
                            Ok(parsed_form) => {
                                let Ok(audio_bytes) = base64::decode(parsed_form.audio.clone()) else {
                                    // print_to_terminal(0, "whisper app: got invalid base64");
                                    print_to_terminal(0, "app:whisper: got invalid base64");
                                    continue;
                                };

                                let res = send_and_await_response(
                                    &Address {
                                        node: our.node.clone(),
                                        process: ProcessId::from_str("nn:whisper:drew.uq").unwrap(),
                                    },
                                    &Request {
                                        inherit: false,
                                        expects_response: Some(30), // TODO evaluate
                                        ipc: vec![],
                                        metadata: None,
                                    },
                                    Some(&Payload {
                                        mime: Some("application/octet-stream".to_string()),
                                        bytes: audio_bytes.clone(),
                                    }),
                                );

                                let text = match res {
                                    Ok((src, msg)) => {
                                        let Message::Response(res) = msg else { panic!(); };
                                        res.0.ipc
                                    },
                                    Err(_e) => { "error".to_string().as_bytes().to_vec() }
                                };

                                print_to_terminal(0, &format!("whisper app: output: {:?}", text));
        
                                send_http_response(
                                    200,
                                    default_headers.clone(),
                                    text,
                                );
                            }
                            Err(e) => print_to_terminal(0, &format!("whisper app: got invalid form: {:?}", e))
                        }
                    }
                    _ => { todo!() }
                }
            }
        }
    }
}
