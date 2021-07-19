// // Copyright 2020 IOTA Stiftung
// // SPDX-License-Identifier: Apache-2.0

// use iota_client::NodeInfoWrapper;
// use iota_wallet::Error;
// use neon::prelude::*;

// pub struct NodeInfoTask {
//     pub account_id: String,
//     pub url: Option<String>,
//     pub jwt: Option<String>,
//     pub auth: Option<(String, String)>,
// }

// impl Task for NodeInfoTask {
//     type Output = NodeInfoWrapper;
//     type Error = Error;
//     type JsEvent = JsValue;

//     fn perform(&self) -> Result<Self::Output, Self::Error> {
//         crate::block_on(crate::convert_async_panics(|| async {
//             let auth = self.auth.as_ref().map(|a| (a.0.as_str(), a.1.as_str()));
//             crate::get_account(&self.account_id)
//                 .await
//                 .get_node_info(self.url.as_deref(), self.jwt.as_deref(), auth)
//                 .await
//         }))
//     }

//     fn complete(self, mut cx: TaskContext, value: Result<Self::Output, Self::Error>) -> JsResult<Self::JsEvent> {
//         match value {
//             Ok(val) => Ok(neon_serde::to_value(&mut cx, &val)?),
//             Err(e) => cx.throw_error(e.to_string()),
//         }
//     }
// }
