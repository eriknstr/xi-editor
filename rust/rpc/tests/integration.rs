// Copyright 2017 Google Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate serde_json;
extern crate xi_rpc;

use std::time::Duration;

use serde_json::Value;
use xi_rpc::{Handler, RpcLoop, RpcCtx, RpcCall, RemoteError};
use xi_rpc::test_utils::{test_channel, make_reader};

/// Handler that responds to requests with whatever params they sent.
pub struct EchoHandler;

#[allow(unused)]
impl Handler for EchoHandler {
    type Notification = RpcCall;
    type Request = RpcCall;
    fn handle_notification(&mut self, ctx: RpcCtx, rpc: Self::Notification) {}
    fn handle_request(&mut self, ctx: RpcCtx, rpc: Self::Request)
                      -> Result<Value, RemoteError> {
        Ok(rpc.params)
    }
}

#[test]
fn test_recv_notif() {
    // we should not reply to a well formed notification
    let mut handler = EchoHandler;
    let (tx, mut rx) = test_channel();
    let mut rpc_looper = RpcLoop::new(tx);
    let r = make_reader(r#"{"method": "hullo", "params": {"words": "plz"}}"#);
    rpc_looper.mainloop(|| r, &mut handler);
    let resp = rx.next_timeout(Duration::from_millis(100));
    assert!(resp.is_none());
}

#[test]
fn test_recv_resp() {
    // we should reply to a well formed request
    let mut handler = EchoHandler;
    let (tx, mut rx) = test_channel();
    let mut rpc_looper = RpcLoop::new(tx);
    let r = make_reader(r#"{"id": 1, "method": "hullo", "params": {"words": "plz"}}"#);
    rpc_looper.mainloop(|| r, &mut handler);
    let resp = rx.expect_response().unwrap();
    assert_eq!(resp["words"], json!("plz"));
    // do it again
    let r = make_reader(r#"{"id": 0, "method": "hullo", "params": {"words": "yay"}}"#);
    rpc_looper.mainloop(|| r, &mut handler);
    let resp = rx.expect_response().unwrap();
    assert_eq!(resp["words"], json!("yay"));
}

#[test]
fn test_recv_error() {
    // a malformed request containing an ID should receive an error
    let mut handler = EchoHandler;
    let (tx, mut rx) = test_channel();
    let mut rpc_looper = RpcLoop::new(tx);
    let r = make_reader(
        r#"{"id": 0, "method": "hullo","args": {"args": "should", "be": "params"}}"#);
    rpc_looper.mainloop(|| r, &mut handler);
    let resp = rx.expect_response();
    assert!(resp.is_err(), "{:?}", resp);
}
