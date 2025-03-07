// Copyright 2023 Zinc Labs Inc.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use http_auth_basic::Credentials;
use tonic::{Request, Status};

use crate::common::{
    infra::{
        cluster::get_internal_grpc_token,
        config::{CONFIG, ROOT_USER, USERS},
    },
    utils::auth::{get_hash, is_root_user},
};

pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let metadata = req.metadata();
    if !metadata.contains_key(&CONFIG.grpc.org_header_key)
        && !metadata.contains_key("authorization")
    {
        return Err(Status::unauthenticated("No valid auth token"));
    }

    let token = req
        .metadata()
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    if token.eq(get_internal_grpc_token().as_str()) {
        Ok(req)
    } else {
        let org_id = metadata.get(&CONFIG.grpc.org_header_key);
        if org_id.is_none() {
            return Err(Status::invalid_argument(format!(
                "Please specify organization id with header key '{}' ",
                &CONFIG.grpc.org_header_key
            )));
        }

        let credentials = match Credentials::from_header(token) {
            Ok(c) => c,
            Err(err) => {
                log::info!("Err authenticating {}", err);
                return Err(Status::unauthenticated("No valid auth token"));
            }
        };

        let user_id = credentials.user_id;
        let user = if is_root_user(&user_id) {
            ROOT_USER.get("root").unwrap()
        } else if let Some(user) = USERS.get(&format!(
            "{}/{}",
            org_id.unwrap().to_str().unwrap(),
            &user_id
        )) {
            user
        } else {
            return Err(Status::unauthenticated("No valid auth token"));
        };

        if user.token.eq(&credentials.password) {
            return Ok(req);
        }
        let in_pass = get_hash(&credentials.password, &user.salt);
        if user_id.eq(&user.email)
            && (credentials.password.eq(&user.password) || in_pass.eq(&user.password))
        {
            Ok(req)
        } else {
            Err(Status::unauthenticated("No valid auth token"))
        }
    }
}

#[cfg(test)]
mod tests {
    use tonic::metadata::MetadataValue;

    use super::*;
    use crate::common::{infra::config::INSTANCE_ID, meta::user::User};

    #[actix_web::test]
    async fn test_check_no_auth() {
        INSTANCE_ID.insert("instance_id".to_owned(), "instance".to_string());
        ROOT_USER.insert(
            "root".to_string(),
            User {
                email: "root@example.com".to_string(),
                password: "Complexpass#123".to_string(),
                role: crate::common::meta::user::UserRole::Root,
                salt: "Complexpass#123".to_string(),
                first_name: "root".to_owned(),
                last_name: "".to_owned(),
                token: "token".to_string(),
                rum_token: Some("rum_token".to_string()),
                org: "dummy".to_owned(),
                is_ldap: false,
            },
        );

        let mut request = tonic::Request::new(());

        let token: MetadataValue<_> = "basic cm9vdEBleGFtcGxlLmNvbTp0b2tlbg==".parse().unwrap();
        let meta: &mut tonic::metadata::MetadataMap = request.metadata_mut();
        meta.insert("authorization2", token.clone());

        let res = check_auth(request);
        assert!(res.is_err())
    }

    #[actix_web::test]
    async fn test_check_auth() {
        INSTANCE_ID.insert("instance_id".to_owned(), "instance".to_string());
        ROOT_USER.insert(
            "root".to_string(),
            User {
                email: "root@example.com".to_string(),
                password: "Complexpass#123".to_string(),
                role: crate::common::meta::user::UserRole::Root,
                salt: "Complexpass#123".to_string(),
                first_name: "root".to_owned(),
                last_name: "".to_owned(),
                token: "token".to_string(),
                rum_token: Some("rum_token".to_string()),
                org: "dummy".to_owned(),
                is_ldap: false,
            },
        );

        let mut request = tonic::Request::new(());
        let token: MetadataValue<_> = "instance".parse().unwrap();
        let meta: &mut tonic::metadata::MetadataMap = request.metadata_mut();
        meta.insert("authorization", token.clone());

        assert!(check_auth(request).is_ok())
    }

    #[actix_web::test]
    async fn test_check_err_auth() {
        INSTANCE_ID.insert("instance_id".to_owned(), "instance".to_string());
        ROOT_USER.insert(
            "root".to_string(),
            User {
                email: "root@example.com".to_string(),
                password: "Complexpass#123".to_string(),
                role: crate::common::meta::user::UserRole::Root,
                salt: "Complexpass#123".to_string(),
                first_name: "root".to_owned(),
                last_name: "".to_owned(),
                token: "token".to_string(),
                rum_token: Some("rum_token".to_string()),
                org: "dummy".to_owned(),
                is_ldap: false,
            },
        );
        let mut request = tonic::Request::new(());

        let token: MetadataValue<_> = "basic cm9vdEBleGFtcGxlLmNvbTp0b2tlbjg4OA=="
            .parse()
            .unwrap();
        let meta: &mut tonic::metadata::MetadataMap = request.metadata_mut();
        meta.insert("authorization", token.clone());

        let res = check_auth(request);
        assert!(res.is_err())
    }
}
