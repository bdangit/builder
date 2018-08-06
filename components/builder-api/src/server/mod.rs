// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
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

mod handlers;

use std::path::PathBuf;

use github_api_client::GitHubClient;
use hab_net::privilege::FeatureFlags;
use http_gateway;
use http_gateway::app::prelude::*;
use iron;
use mount::Mount;
use oauth_client::client::OAuth2Client;
use persistent;
use segment_api_client::SegmentClient;
use staticfile::Static;

use backend::{s3, s3::S3Cli};
use upstream::{UpstreamCli, UpstreamClient, UpstreamMgr};

use self::handlers::*;
use super::depot;
use super::github;
use config::Config;

struct ApiSrv;
impl HttpGateway for ApiSrv {
    const APP_NAME: &'static str = "builder-api";

    type Config = Config;

    fn add_middleware(config: Arc<Self::Config>, chain: &mut iron::Chain) {
        chain.link(persistent::Read::<Self::Config>::both(config.clone()));

        chain.link(persistent::Read::<OAuthCli>::both(OAuth2Client::new(
            config.oauth.clone(),
        )));

        chain.link(persistent::Read::<GitHubCli>::both(GitHubClient::new(
            config.github.clone(),
        )));

        chain.link(persistent::Read::<SegmentCli>::both(SegmentClient::new(
            config.segment.clone(),
        )));

        chain.link(persistent::Read::<S3Cli>::both(s3::S3Handler::new(
            config.s3.to_owned(),
        )));

        chain.link(persistent::Read::<UpstreamCli>::both(
            UpstreamClient::default(),
        ));

        chain.link_before(XRouteClient);
        chain.link_after(Cors);
    }

    fn mount(config: Arc<Self::Config>, chain: iron::Chain) -> Mount {
        let mut mount = Mount::new();

        if let Some(ref path) = config.ui.root {
            debug!("Mounting UI at filepath {}", path);
            mount.mount("/", Static::new(path));
        }
        mount.mount("/v1", chain);

        let mut depot_chain_v1 = iron::Chain::new(depot::server::router(config.clone()));
        Self::add_middleware(config.clone(), &mut depot_chain_v1);
        mount.mount("/v1", depot_chain_v1);

        // TBD: Deprecate legacy depot API path
        let mut depot_chain = iron::Chain::new(depot::server::router(config.clone()));
        Self::add_middleware(config, &mut depot_chain);
        mount.mount("/v1/depot", depot_chain);
        mount
    }

    fn router(config: Arc<Self::Config>) -> Router {
        let basic = Authenticated::new(config.key_dir.clone());
        let admin = Authenticated::new(PathBuf::new()).require(FeatureFlags::ADMIN);

        let mut r = Router::new();

        if config.jobsrv_enabled {
            r.post(
                "/jobs/group/:id/promote/:channel",
                XHandler::new(job_group_promote).before(basic.clone()),
                "job_group_promote",
            );
            r.post(
                "/jobs/group/:id/demote/:channel",
                XHandler::new(job_group_demote).before(basic.clone()),
                "job_group_demote",
            );
            r.post(
                "/jobs/group/:id/cancel",
                XHandler::new(job_group_cancel).before(basic.clone()),
                "job_group_cancel",
            );
            r.get("/rdeps/:origin/:name", rdeps_show, "rdeps");
            r.get(
                "/jobs/:id",
                XHandler::new(job_show).before(basic.clone()),
                "job",
            );
            r.get(
                "/jobs/:id/log",
                XHandler::new(job_log).before(basic.clone()),
                "job_log",
            );
            r.post(
                "/projects",
                XHandler::new(project_create).before(basic.clone()),
                "projects",
            );
            r.get(
                "/projects/:origin/:name",
                XHandler::new(project_show).before(basic.clone()),
                "project",
            );
            r.get(
                "/projects/:origin",
                XHandler::new(project_list).before(basic.clone()),
                "project_list",
            );
            r.get(
                "/projects/:origin/:name/jobs",
                XHandler::new(project_jobs).before(basic.clone()),
                "project_jobs",
            );
            r.put(
                "/projects/:origin/:name",
                XHandler::new(project_update).before(basic.clone()),
                "edit_project",
            );
            r.delete(
                "/projects/:origin/:name",
                XHandler::new(project_delete).before(basic.clone()),
                "delete_project",
            );
            r.patch(
                "/projects/:origin/:name/:visibility",
                XHandler::new(project_privacy_toggle).before(basic.clone()),
                "project_privacy_toggle",
            );
            r.get(
                "/projects/:origin/:name/integrations/:integration/default",
                XHandler::new(get_project_integration).before(basic.clone()),
                "project_integration_get",
            );
            r.put(
                "/projects/:origin/:name/integrations/:integration/default",
                XHandler::new(create_project_integration).before(basic.clone()),
                "project_integration_put",
            );
            r.delete(
                "/projects/:origin/:name/integrations/:integration/default",
                XHandler::new(delete_project_integration).before(basic.clone()),
                "project_integration_delete",
            );
            r.get(
                "/ext/installations/:install_id/repos/:repo_id/contents/:path",
                XHandler::new(github::repo_file_content).before(basic.clone()),
                "ext_repo_content",
            );
        }

        r.get("/status", status, "status");
        r.get("/authenticate/:code", authenticate, "authenticate");
        r.post("/notify", notify, "notify");
        r.patch(
            "/profile",
            XHandler::new(update_profile).before(basic.clone()),
            "update_profile",
        );
        r.get(
            "/profile",
            XHandler::new(get_profile).before(basic.clone()),
            "get_profile",
        );

        r.get(
            "/profile/access-tokens",
            XHandler::new(get_access_tokens).before(basic.clone()),
            "get_access_tokens",
        );
        r.post(
            "/profile/access-tokens",
            XHandler::new(generate_access_token).before(basic.clone()),
            "generate_access_token",
        );
        r.delete(
            "/profile/access-tokens/:id",
            XHandler::new(revoke_access_token).before(basic.clone()),
            "revoke_access_token",
        );

        r.get(
            "/user/invitations",
            XHandler::new(list_account_invitations).before(basic.clone()),
            "user_invitations",
        );
        r.get(
            "/user/origins",
            XHandler::new(list_user_origins).before(basic.clone()),
            "user_origins",
        );
        r.post(
            "/ext/integrations/:registry_type/credentials/validate",
            XHandler::new(validate_registry_credentials).before(basic.clone()),
            "ext_credentials_registry",
        );

        r.post(
            "/admin/search",
            XHandler::new(search).before(admin.clone()),
            "admin_search",
        );
        r.get(
            "/admin/accounts/:id",
            XHandler::new(account_show).before(admin.clone()),
            "admin_account",
        );

        r
    }
}

pub fn run(config: Config) -> AppResult<()> {
    UpstreamMgr::start(&config, s3::S3Handler::new(config.s3.to_owned())).unwrap();
    http_gateway::start::<ApiSrv>(config)
}
