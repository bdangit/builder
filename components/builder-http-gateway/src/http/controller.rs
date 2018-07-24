// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
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

pub use hab_net::{ErrCode, NetError, NetOk, NetResult};
pub use iron::headers::{ContentType, UserAgent};
pub use iron::prelude::*;
pub use iron::{headers, status};
use protobuf;
use protocol::Routable;

pub use super::headers::*;
use super::middleware::XRouteClient;
pub use super::middleware::*;
pub use super::net_err_to_http;
pub use super::rendering::{render_json, render_net_error};
pub use conn::RouteBroker;

pub fn route_message<M, R>(req: &mut Request, msg: &M) -> NetResult<R>
where
    M: Routable,
    R: protobuf::Message,
{
    req.extensions
        .get_mut::<XRouteClient>()
        .expect("no XRouteClient extension in request")
        .route::<M, R>(msg)
}
